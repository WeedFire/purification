use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use super::{is_file_locked, move_to_recycle_bin, secure_delete_file};
use crate::models::{
    CleanupItem, CleanupMode, CleanupProgress, CleanupResult, FailedCleanupItem, FileInfo,
};

/// 文件清理器
pub struct FileCleaner {
    progress: Arc<CleanupProgressTracker>,
    cancel_flag: Arc<AtomicBool>,
}

impl FileCleaner {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(CleanupProgressTracker::new()),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取进度追踪器
    pub fn progress_tracker(&self) -> Arc<CleanupProgressTracker> {
        Arc::clone(&self.progress)
    }

    /// 取消清理
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    /// 检查是否已取消
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// 清理文件
    pub fn cleanup(&self, files: &[FileInfo], mode: CleanupMode, scan_id: Uuid) -> CleanupResult {
        self.progress.reset();
        self.progress.set_cleaning(true);
        self.progress.set_total(files.len() as u64);
        self.cancel_flag.store(false, Ordering::SeqCst);

        let mut result = CleanupResult::new(scan_id, mode);

        for file in files {
            if self.is_cancelled() {
                break;
            }

            self.progress
                .set_current_file(file.path.to_string_lossy().to_string());

            // 检查文件是否被锁定
            if is_file_locked(&file.path) {
                result.failed_items.push(FailedCleanupItem {
                    path: file.path.clone(),
                    size: file.size,
                    reason: "文件正在被使用".to_string(),
                });
                result.failed_count += 1;
                self.progress.increment_processed();
                continue;
            }

            // 检查文件是否受保护
            if file.is_protected {
                result.failed_items.push(FailedCleanupItem {
                    path: file.path.clone(),
                    size: file.size,
                    reason: file
                        .protection_reason
                        .clone()
                        .unwrap_or_else(|| "受保护文件".to_string()),
                });
                result.failed_count += 1;
                self.progress.increment_processed();
                continue;
            }

            // 执行清理
            let cleanup_result = match mode {
                CleanupMode::RecycleBin => move_to_recycle_bin(&file.path),
                CleanupMode::Permanent => self.permanent_delete(&file.path),
                CleanupMode::SecureWipe => secure_delete_file(&file.path, 3),
            };

            match cleanup_result {
                Ok(()) => {
                    result.success_items.push(CleanupItem {
                        path: file.path.clone(),
                        size: file.size,
                        cleanup_time: chrono::Utc::now().timestamp_millis(),
                    });
                    result.success_count += 1;
                    result.released_size += file.size;
                    self.progress.add_released_size(file.size);
                }
                Err(e) => {
                    result.failed_items.push(FailedCleanupItem {
                        path: file.path.clone(),
                        size: file.size,
                        reason: e.to_string(),
                    });
                    result.failed_count += 1;
                }
            }

            self.progress.increment_processed();
        }

        self.progress.set_cleaning(false);
        result
    }

    /// 永久删除文件
    fn permanent_delete(&self, path: &PathBuf) -> Result<(), anyhow::Error> {
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
            // 删除后清理空父目录
            Self::remove_empty_parents(path);
        }
        Ok(())
    }

    /// 从文件路径向上递归删除空目录
    fn remove_empty_parents(file_path: &PathBuf) {
        let mut current = file_path.clone();
        loop {
            let parent = match current.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
            // 到达文件系统根目录时停止
            if parent == current {
                break;
            }
            match std::fs::read_dir(&parent) {
                Ok(mut entries) => {
                    if entries.next().is_none() {
                        let _ = std::fs::remove_dir(&parent);
                        current = parent;
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
}

impl Default for FileCleaner {
    fn default() -> Self {
        Self::new()
    }
}

/// 清理进度追踪器
#[derive(Debug)]
pub struct CleanupProgressTracker {
    current_file: RwLock<String>,
    processed_count: AtomicU64,
    total_count: AtomicU64,
    released_size: AtomicU64,
    is_cleaning: AtomicBool,
}

impl CleanupProgressTracker {
    pub fn new() -> Self {
        Self {
            current_file: RwLock::new(String::new()),
            processed_count: AtomicU64::new(0),
            total_count: AtomicU64::new(0),
            released_size: AtomicU64::new(0),
            is_cleaning: AtomicBool::new(false),
        }
    }

    pub fn reset(&self) {
        *self.current_file.write() = String::new();
        self.processed_count.store(0, Ordering::SeqCst);
        self.total_count.store(0, Ordering::SeqCst);
        self.released_size.store(0, Ordering::SeqCst);
        self.is_cleaning.store(false, Ordering::SeqCst);
    }

    pub fn set_current_file(&self, file: String) {
        *self.current_file.write() = file;
    }

    pub fn set_total(&self, total: u64) {
        self.total_count.store(total, Ordering::SeqCst);
    }

    pub fn increment_processed(&self) {
        self.processed_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn add_released_size(&self, size: u64) {
        self.released_size.fetch_add(size, Ordering::SeqCst);
    }

    pub fn set_cleaning(&self, value: bool) {
        self.is_cleaning.store(value, Ordering::SeqCst);
    }

    pub fn get_progress(&self) -> CleanupProgress {
        CleanupProgress {
            current_file: self.current_file.read().clone(),
            processed_count: self.processed_count.load(Ordering::SeqCst),
            total_count: self.total_count.load(Ordering::SeqCst),
            released_size: self.released_size.load(Ordering::SeqCst),
            is_cleaning: self.is_cleaning.load(Ordering::SeqCst),
        }
    }
}

impl Default for CleanupProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}
