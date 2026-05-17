use crossbeam::channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::progress::ScanProgressTracker;
use crate::models::{DiskSpaceInfo, FileInfo, ScanConfig, ScanProgress};

/// 文件扫描器
pub struct FileScanner {
    config: ScanConfig,
    progress: Arc<ScanProgressTracker>,
    pause_flag: Arc<AtomicBool>,
    cancel_flag: Arc<AtomicBool>,
}

impl FileScanner {
    pub fn new(config: ScanConfig) -> Self {
        Self {
            config,
            progress: Arc::new(ScanProgressTracker::new()),
            pause_flag: Arc::new(AtomicBool::new(false)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取进度追踪器
    pub fn progress_tracker(&self) -> Arc<ScanProgressTracker> {
        Arc::clone(&self.progress)
    }

    /// 暂停扫描
    pub fn pause(&self) {
        self.pause_flag.store(true, Ordering::SeqCst);
    }

    /// 继续扫描
    pub fn resume(&self) {
        self.pause_flag.store(false, Ordering::SeqCst);
    }

    /// 取消扫描
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    /// 检查是否应该暂停
    fn check_pause(&self) {
        while self.pause_flag.load(Ordering::SeqCst) && !self.cancel_flag.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// 检查是否已取消
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// 扫描文件
    pub fn scan(&self) -> Result<Vec<FileInfo>, anyhow::Error> {
        self.progress.reset();
        self.progress.set_scanning(true);
        self.cancel_flag.store(false, Ordering::SeqCst);
        self.pause_flag.store(false, Ordering::SeqCst);

        let mut results = Vec::new();
        let start_time = Instant::now();

        for scan_path in &self.config.paths {
            if self.is_cancelled() {
                break;
            }

            if !scan_path.exists() {
                continue;
            }

            let path_str = scan_path.to_string_lossy().to_string();
            self.progress.set_current_path(path_str);

            // 使用 jwalk 进行并行遍历
            let walk_result = self.walk_directory(scan_path, &mut results);
            if let Err(e) = walk_result {
                eprintln!("Error walking directory {:?}: {}", scan_path, e);
            }
        }

        self.progress.set_scanning(false);
        let duration = start_time.elapsed();
        println!(
            "Scan completed in {:.2}s, found {} files",
            duration.as_secs_f64(),
            results.len()
        );

        Ok(results)
    }

    /// 遍历目录
    fn walk_directory(
        &self,
        root: &Path,
        results: &mut Vec<FileInfo>,
    ) -> Result<(), anyhow::Error> {
        // 如果路径是文件，直接处理单个文件
        if root.is_file() {
            return self.process_single_file(root, results);
        }

        let max_depth = self.config.max_depth.unwrap_or(20);

        // 使用 jwalk 进行并行遍历
        for entry_result in jwalk::WalkDir::new(root)
            .max_depth(max_depth)
            .follow_links(false)
        {
            self.check_pause();

            if self.is_cancelled() {
                break;
            }

            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // 检查排除模式
            if self.should_exclude(&path) {
                continue;
            }

            // 获取文件元数据
            let metadata = match std::fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // 更新进度
            self.progress.increment_scanned();

            // 跳过目录（空文件夹单独处理）
            if metadata.is_dir() {
                self.progress.increment_dirs();
                continue;
            }

            // 处理单个文件
            if let Err(_) = self.build_file_info(&path, &metadata, results) {
                continue;
            }
        }

        Ok(())
    }

    /// 处理单个文件（当扫描路径直接指向一个文件时）
    fn process_single_file(
        &self,
        file_path: &Path,
        results: &mut Vec<FileInfo>,
    ) -> Result<(), anyhow::Error> {
        // 检查排除模式
        if self.should_exclude(file_path) {
            return Ok(());
        }

        // 获取文件元数据
        let metadata = match std::fs::metadata(file_path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error reading file metadata {:?}: {}", file_path, e);
                return Ok(());
            }
        };

        self.progress.increment_scanned();

        // 检查隐藏文件
        if !self.config.include_hidden && is_hidden(file_path) {
            return Ok(());
        }

        let _ = self.build_file_info(file_path, &metadata, results);

        Ok(())
    }

    /// 构建文件信息并将其推入结果列表
    fn build_file_info(
        &self,
        path: &Path,
        metadata: &std::fs::Metadata,
        results: &mut Vec<FileInfo>,
    ) -> Result<(), ()> {
        // 检查文件大小过滤
        if metadata.len() < self.config.min_file_size {
            return Err(());
        }

        // 创建文件信息
        let mut file_info = FileInfo::new(path.to_path_buf());
        file_info.size = metadata.len();

        // 获取时间信息
        if let Ok(created) = metadata.created() {
            if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                file_info.created_time = Some(duration.as_millis() as i64);
            }
        }

        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                file_info.modified_time = Some(duration.as_millis() as i64);
            }
        }

        if let Ok(accessed) = metadata.accessed() {
            if let Ok(duration) = accessed.duration_since(std::time::UNIX_EPOCH) {
                file_info.accessed_time = Some(duration.as_millis() as i64);
            }
        }

        // 检查大文件
        if metadata.len() >= self.config.large_file_threshold {
            file_info
                .categories
                .push(crate::models::FileCategory::LargeFile);
        }

        results.push(file_info);
        self.progress.increment_candidates();

        Ok(())
    }

    /// 检查路径是否应该被排除
    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.exclude_patterns {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                if glob.matches(&path_str) {
                    return true;
                }
            }
        }

        false
    }
}

/// 检查文件是否为隐藏文件
fn is_hidden(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            return metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                return name_str.starts_with('.');
            }
        }
    }

    false
}

/// 获取磁盘空间信息
pub fn get_disk_space(path: &Path) -> Option<DiskSpaceInfo> {
    let path_str = path.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            // Windows 下使用 WinAPI 获取磁盘信息
            use std::ptr;
            use winapi::shared::ntdef::ULARGE_INTEGER;
            use winapi::um::fileapi::GetDiskFreeSpaceExW;

            let path_wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

            let mut free_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
            let mut total_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
            let mut available_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };

            unsafe {
                if GetDiskFreeSpaceExW(
                    path_wide.as_ptr(),
                    &mut free_bytes,
                    &mut total_bytes,
                    &mut available_bytes,
                ) != 0
                {
                    return Some(DiskSpaceInfo {
                        path: path_str,
                        total_space: *total_bytes.QuadPart(),
                        used_space: *total_bytes.QuadPart() - *free_bytes.QuadPart(),
                        available_space: *available_bytes.QuadPart(),
                    });
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(stat) = nix::sys::statvfs::statvfs(path) {
            let block_size = stat.block_size() as u64;
            return Some(DiskSpaceInfo {
                path: path_str,
                total_space: stat.blocks() * block_size,
                used_space: (stat.blocks() - stat.blocks_available()) * block_size,
                available_space: stat.blocks_available() * block_size,
            });
        }
    }

    None
}
