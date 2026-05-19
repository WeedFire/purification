// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod models;
pub mod scanner;
pub mod classifier;
pub mod cleaner;
pub mod database;
pub mod utils;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use tauri::{Manager, State, Emitter};
use uuid::Uuid;

use models::*;
use scanner::{FileScanner, EmptyFolderDetector, ScanProgressSnapshot};
use classifier::FileClassifier;
use cleaner::FileCleaner;
use database::{Database, ScanHistoryRecord, CleanupLogRecord};
use utils::{open_file_in_explorer, open_directory, get_app_data_dir};

/// 应用状态
pub struct AppState {
    pub scanner: RwLock<Option<Arc<FileScanner>>>,
    pub classifier: Arc<FileClassifier>,
    pub cleaner: RwLock<Option<Arc<FileCleaner>>>,
    pub database: Mutex<Option<Database>>,
    pub current_scan_result: Arc<RwLock<Option<ScanResult>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            scanner: RwLock::new(None),
            classifier: Arc::new(FileClassifier::new()),
            cleaner: RwLock::new(None),
            database: Mutex::new(None),
            current_scan_result: Arc::new(RwLock::new(None)),
        }
    }
}

// ==================== 扫描相关命令 ====================

#[tauri::command]
async fn start_scan(
    paths: Vec<String>,
    config: Option<ScanConfig>,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let scan_config = config.unwrap_or_else(|| ScanConfig::new(path_bufs.clone()));
    
    // 创建扫描器
    let scanner = Arc::new(FileScanner::new(scan_config));
    let progress_tracker = scanner.progress_tracker();
    
    // 保存扫描器到状态
    *state.scanner.write() = Some(Arc::clone(&scanner));
    
    // 设置扫描状态
    progress_tracker.set_scanning(true);
    
    // 获取分类器和状态引用
    let classifier = Arc::clone(&state.classifier);
    let state_result = Arc::clone(&state.current_scan_result);
    
    // 克隆路径用于结果
    let paths_clone = paths.clone();
    let path_bufs_clone = path_bufs.clone();
    
    // 启动进度更新任务
    let app_handle_progress = app.clone();
    let progress_tracker_clone = Arc::clone(&progress_tracker);
    tokio::spawn(async move {
        let mut last_count = 0u64;
        loop {
            let progress = progress_tracker_clone.get_progress();
            
            // 发送进度事件
            app_handle_progress.emit("scan-progress", progress.clone()).ok();
            
            // 检查是否完成
            if !progress.is_scanning && !progress.is_paused {
                break;
            }
            
            // 如果进度有变化，立即发送
            if progress.scanned_count != last_count {
                last_count = progress.scanned_count;
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    });
    
    // 在后台线程执行扫描
    let app_handle_result = app.clone();
    let db_path = get_app_data_dir().map(|d| d.join("wisweep.db")).ok();
    tokio::task::spawn_blocking(move || {
        let mut result = ScanResult::new(paths_clone);
        
        match scanner.scan() {
            Ok(mut files) => {
                classifier.classify_batch(&mut files);
                let candidates: Vec<FileInfo> = files.into_iter()
                    .filter(|f| !f.categories.is_empty() && !f.is_protected)
                    .collect();
                result.candidates = candidates;
                result.total_files = scanner.progress_tracker().get_progress().scanned_count;
                result.total_dirs = scanner.progress_tracker().get_progress().dir_count;
            }
            Err(e) => {
                eprintln!("扫描失败: {}", e);
                progress_tracker.set_scanning(false);
                let _ = app_handle_result.emit("scan-progress", progress_tracker.get_progress());
                return;
            }
        }
        
        let empty_detector = EmptyFolderDetector::new();
        for path in &path_bufs_clone {
            if let Ok(empty_folders) = empty_detector.detect(path) {
                result.empty_folders.extend(empty_folders);
            }
        }
        
        result.finish();
        
        // 保存结果到全局状态
        *state_result.write() = Some(result);
        
        // 在 blocking 线程中保存扫描历史（新开 DB 连接）
        if let Some(ref db_path) = db_path {
            if let Ok(db) = Database::new(db_path) {
                if let Some(scan_result) = state_result.read().as_ref() {
                    let history = ScanHistoryRecord {
                        id: scan_result.scan_id,
                        paths: path_bufs_clone.iter().map(|p| p.to_string_lossy().to_string()).collect(),
                        start_time: scan_result.start_time,
                        end_time: scan_result.end_time,
                        duration_ms: scan_result.duration_ms,
                        total_files: scan_result.total_files,
                        total_dirs: scan_result.total_dirs,
                        candidate_count: scan_result.candidates.len() as u64,
                        candidate_size: scan_result.total_releasable_size,
                        empty_folder_count: scan_result.empty_folders.len() as u64,
                        cleaned_count: 0,
                        cleaned_size: 0,
                        config: None,
                    };
                    let _ = db.save_scan_history(&history);
                }
            }
        }
        
        progress_tracker.set_scanning(false);
        let final_progress = progress_tracker.get_progress();
        app_handle_result.emit("scan-progress", final_progress).ok();
    });
    
    Ok("scanning".to_string())
}

#[tauri::command]
fn pause_scan(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(scanner) = state.scanner.read().as_ref() {
        scanner.pause();
        Ok(())
    } else {
        Err("没有正在进行的扫描".to_string())
    }
}

#[tauri::command]
fn resume_scan(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(scanner) = state.scanner.read().as_ref() {
        scanner.resume();
        Ok(())
    } else {
        Err("没有暂停的扫描".to_string())
    }
}

#[tauri::command]
fn cancel_scan(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(scanner) = state.scanner.read().as_ref() {
        scanner.cancel();
        Ok(())
    } else {
        Err("没有正在进行的扫描".to_string())
    }
}

#[tauri::command]
fn get_scan_progress(state: State<'_, AppState>) -> Result<ScanProgressSnapshot, String> {
    if let Some(scanner) = state.scanner.read().as_ref() {
        Ok(scanner.progress_tracker().get_progress())
    } else {
        Err("没有扫描任务".to_string())
    }
}

#[tauri::command]
fn get_scan_result(state: State<'_, AppState>) -> Result<Option<ScanResult>, String> {
    Ok(state.current_scan_result.read().clone())
}

// ==================== 清理相关命令 ====================

#[tauri::command]
async fn cleanup_files(
    file_paths: Vec<String>,
    mode: CleanupMode,
    scan_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<CleanupResult, String> {
    let scan_uuid = Uuid::parse_str(&scan_id).map_err(|e| format!("无效的扫描ID: {}", e))?;
    
    // 获取当前扫描结果中的文件信息
    let scan_result = state.current_scan_result.read().clone();
    let files_to_clean: Vec<FileInfo> = if let Some(result) = scan_result {
        result.candidates.into_iter()
            .filter(|f| file_paths.contains(&f.path.to_string_lossy().to_string()))
            .collect()
    } else {
        return Err("没有扫描结果".to_string());
    };
    
    let cleaner = Arc::new(FileCleaner::new());
    let progress_tracker = cleaner.progress_tracker();
    
    *state.cleaner.write() = Some(Arc::clone(&cleaner));
    
    // 启动进度更新任务
    let app_handle = app.clone();
    tokio::spawn(async move {
        loop {
            let progress = progress_tracker.get_progress();
            
            if !progress.is_cleaning {
                break;
            }
            
            app_handle.emit("cleanup-progress", progress.clone()).ok();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
    
    let result = cleaner.cleanup(&files_to_clean, mode, scan_uuid);
    
    // 保存清理记录到数据库
        if let Some(db) = state.database.lock().as_ref() {
            for item in &result.success_items {
                let log = database::CleanupLogRecord {
                    scan_id: scan_uuid,
                    cleanup_time: item.cleanup_time,
                    file_path: item.path.to_string_lossy().to_string(),
                    file_size: item.size,
                    category: None,
                    delete_mode: mode.display_name().to_string(),
                    result: "success".to_string(),
                    error_message: None,
                };
                db.save_cleanup_log(&log).ok();
            }
            // 更新扫描历史中的清理计数
            if let Some(_scan_result) = state.current_scan_result.read().as_ref() {
                if let Ok(history_list) = db.get_scan_history(1) {
                    if let Some(mut history) = history_list.into_iter().next() {
                        history.cleaned_count = result.success_count;
                        history.cleaned_size = result.released_size;
                        db.save_scan_history(&history).ok();
                    }
                }
            }
        }
    
    Ok(result)
}

#[tauri::command]
fn cancel_cleanup(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(cleaner) = state.cleaner.read().as_ref() {
        cleaner.cancel();
        Ok(())
    } else {
        Err("没有正在进行的清理".to_string())
    }
}

// ==================== 空文件夹命令 ====================

/// 判断目录是否为空（无文件、无子目录）
fn is_dir_empty(dir: &Path) -> bool {
    if let Ok(mut entries) = std::fs::read_dir(dir) {
        entries.next().is_none()
    } else {
        false
    }
}

/// 递归向上删除变成空目录的父目录
fn remove_empty_parent_dirs(start_path: &Path) {
    let mut current = start_path.parent();
    while let Some(parent) = current {
        // 如果父目录不存在或不是目录，停止
        if !parent.exists() || !parent.is_dir() {
            break;
        }
        // 如果父目录不为空，停止
        if !is_dir_empty(parent) {
            break;
        }
        // 删除这个空父目录
        if std::fs::remove_dir(parent).is_ok() {
            current = parent.parent();
        } else {
            // 删除失败，停止
            break;
        }
    }
}

#[tauri::command]
fn delete_empty_folders(dir_paths: Vec<String>) -> Result<Vec<String>, String> {
    let mut failed = Vec::new();
    for path_str in &dir_paths {
        let path = PathBuf::from(path_str);
        if path.exists() && path.is_dir() {
            // 先删除指定的空目录
            if let Err(e) = std::fs::remove_dir(&path) {
                failed.push(format!("{}: {}", path_str, e));
            } else {
                // 成功后向上遍历，删除变成空的父目录
                remove_empty_parent_dirs(&path);
            }
        }
    }
    if failed.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(failed)
    }
}

// ==================== 文件操作命令 ====================

#[tauri::command]
fn open_file_location(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    open_file_in_explorer(&path).map_err(|e| format!("打开文件位置失败: {}", e))
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    open_directory(&path).map_err(|e| format!("打开文件夹失败: {}", e))
}

// ==================== 磁盘空间命令 ====================

#[tauri::command]
fn get_disk_space(path: String) -> Result<DiskSpaceInfo, String> {
    let path = PathBuf::from(&path);
    scanner::get_disk_space(&path).ok_or_else(|| "无法获取磁盘空间信息".to_string())
}

// ==================== 扫描历史命令 ====================

#[tauri::command]
fn get_scan_history(state: State<'_, AppState>) -> Result<Vec<ScanHistoryRecord>, String> {
    if let Some(db) = state.database.lock().as_ref() {
        db.get_scan_history(50)
            .map_err(|e| format!("获取历史失败: {}", e))
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
fn get_cleanup_logs(scan_id: String, state: State<'_, AppState>) -> Result<Vec<CleanupLogRecord>, String> {
    let uuid = Uuid::parse_str(&scan_id).map_err(|e| format!("无效ID: {}", e))?;
    if let Some(db) = state.database.lock().as_ref() {
        db.get_cleanup_logs(&uuid)
            .map_err(|e| format!("获取清理日志失败: {}", e))
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
fn delete_scan_history(scan_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scan_id).map_err(|e| format!("无效ID: {}", e))?;
    if let Some(db) = state.database.lock().as_ref() {
        db.delete_scan_history(&uuid)
            .map_err(|e| format!("删除历史失败: {}", e))
    } else {
        Ok(())
    }
}

#[tauri::command]
fn clear_scan_history(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(db) = state.database.lock().as_ref() {
        db.clear_all_history()
            .map_err(|e| format!("清空历史失败: {}", e))
    } else {
        Ok(())
    }
}

// ==================== 收藏路径命令 ====================

#[tauri::command]
fn add_favorite_path(path: String, alias: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(db) = state.database.lock().as_ref() {
        db.add_favorite_path(&path, alias.as_deref())
            .map_err(|e| format!("添加收藏失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn remove_favorite_path(path: String, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(db) = state.database.lock().as_ref() {
        db.remove_favorite_path(&path)
            .map_err(|e| format!("移除收藏失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn get_favorite_paths(state: State<'_, AppState>) -> Result<Vec<database::FavoritePath>, String> {
    if let Some(db) = state.database.lock().as_ref() {
        db.get_favorite_paths()
            .map_err(|e| format!("获取收藏列表失败: {}", e))
    } else {
        Ok(Vec::new())
    }
}

// ==================== 规则管理命令 ====================

#[tauri::command]
fn get_rules(state: State<'_, AppState>) -> Vec<ClassificationRule> {
    state.classifier.get_rules()
}

#[tauri::command]
fn add_rule(rule: ClassificationRule, state: State<'_, AppState>) -> Result<(), String> {
    state.classifier.add_rule(rule);
    Ok(())
}

#[tauri::command]
fn remove_rule(rule_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.classifier.remove_rule(&rule_id))
}

// ==================== 保护路径管理命令 ====================

#[tauri::command]
fn get_protected_paths(state: State<'_, AppState>) -> Result<Vec<database::ProtectedPath>, String> {
    if let Some(ref db) = *state.database.lock() {
        db.get_protected_paths()
            .map_err(|e| format!("获取保护路径失败: {}", e))
    } else {
        Err("数据库未初始化".to_string())
    }
}

#[tauri::command]
fn add_protected_path(path: String, description: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(ref db) = *state.database.lock() {
        db.add_protected_path(&path, description.as_deref())
            .map_err(|e| format!("添加保护路径失败: {}", e))?;
        
        // 重新加载保护路径到分类器
        if let Ok(paths) = db.get_protected_paths() {
            let path_strings: Vec<String> = paths.into_iter().map(|p| p.path).collect();
            state.classifier.update_protected_paths(path_strings);
        }
        Ok(())
    } else {
        Err("数据库未初始化".to_string())
    }
}

#[tauri::command]
fn remove_protected_path(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(ref db) = *state.database.lock() {
        db.remove_protected_path(id)
            .map_err(|e| format!("移除保护路径失败: {}", e))?;
        
        // 重新加载保护路径到分类器
        if let Ok(paths) = db.get_protected_paths() {
            let path_strings: Vec<String> = paths.into_iter().map(|p| p.path).collect();
            state.classifier.update_protected_paths(path_strings);
        }
        Ok(())
    } else {
        Err("数据库未初始化".to_string())
    }
}

#[tauri::command]
fn reset_protected_paths(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(ref db) = *state.database.lock() {
        db.reset_protected_paths()
            .map_err(|e| format!("重置保护路径失败: {}", e))?;
        
        // 重新加载保护路径到分类器
        if let Ok(paths) = db.get_protected_paths() {
            let path_strings: Vec<String> = paths.into_iter().map(|p| p.path).collect();
            state.classifier.update_protected_paths(path_strings);
        }
        Ok(())
    } else {
        Err("数据库未初始化".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化数据库
    let db_path = get_app_data_dir()
        .map(|dir| dir.join("wisweep.db"))
        .expect("Failed to get app data directory");
    
    // 确保目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    let database = Database::new(&db_path).expect("Failed to initialize database");
    
    // 从数据库加载保护路径
    let protected_paths: Vec<String> = database.get_protected_paths()
        .map(|paths| paths.into_iter().map(|p| p.path).collect())
        .unwrap_or_default();
    
    // 创建分类器并加载保护路径
    let classifier = FileClassifier::new();
    if !protected_paths.is_empty() {
        classifier.update_protected_paths(protected_paths);
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            database: Mutex::new(Some(database)),
            classifier: Arc::new(classifier),
            ..Default::default()
        })
        .invoke_handler(tauri::generate_handler![
            start_scan,
            pause_scan,
            resume_scan,
            cancel_scan,
            get_scan_progress,
            get_scan_result,
            cleanup_files,
            cancel_cleanup,
            open_file_location,
            open_folder,
            get_disk_space,
            add_favorite_path,
            remove_favorite_path,
            get_favorite_paths,
            get_rules,
            add_rule,
            remove_rule,
            get_protected_paths,
            add_protected_path,
            remove_protected_path,
            reset_protected_paths,
            get_scan_history,
            get_cleanup_logs,
            delete_scan_history,
            clear_scan_history,
            delete_empty_folders,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
