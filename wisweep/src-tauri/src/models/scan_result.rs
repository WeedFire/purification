use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{EmptyFolderInfo, FileCategory, FileInfo};

/// 扫描进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    /// 当前扫描路径
    pub current_path: String,
    /// 已扫描文件数
    pub scanned_count: u64,
    /// 已发现候选文件数
    pub candidate_count: u64,
    /// 已发现空文件夹数
    pub empty_folder_count: u64,
    /// 预估剩余时间（秒）
    pub estimated_remaining: Option<u64>,
    /// 是否正在扫描
    pub is_scanning: bool,
    /// 是否已暂停
    pub is_paused: bool,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            current_path: String::new(),
            scanned_count: 0,
            candidate_count: 0,
            empty_folder_count: 0,
            estimated_remaining: None,
            is_scanning: false,
            is_paused: false,
        }
    }
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 扫描ID
    pub scan_id: Uuid,
    /// 扫描路径
    pub paths: Vec<String>,
    /// 扫描开始时间
    pub start_time: i64,
    /// 扫描结束时间
    pub end_time: Option<i64>,
    /// 扫描总耗时（毫秒）
    pub duration_ms: u64,
    /// 扫描的文件总数
    pub total_files: u64,
    /// 扫描的目录总数
    pub total_dirs: u64,
    /// 候选清理文件列表
    pub candidates: Vec<FileInfo>,
    /// 空文件夹列表
    pub empty_folders: Vec<EmptyFolderInfo>,
    /// 按分类统计
    pub category_stats: HashMap<String, CategoryStats>,
    /// 可释放的总空间（字节）
    pub total_releasable_size: u64,
    /// 已清理的文件数
    pub cleaned_count: u64,
    /// 已清理的空间（字节）
    pub cleaned_size: u64,
    /// 扫描配置
    pub config: String,
}

impl ScanResult {
    pub fn new(paths: Vec<String>) -> Self {
        Self {
            scan_id: Uuid::new_v4(),
            paths,
            start_time: chrono::Utc::now().timestamp_millis(),
            end_time: None,
            duration_ms: 0,
            total_files: 0,
            total_dirs: 0,
            candidates: Vec::new(),
            empty_folders: Vec::new(),
            category_stats: HashMap::new(),
            total_releasable_size: 0,
            cleaned_count: 0,
            cleaned_size: 0,
            config: String::new(),
        }
    }

    pub fn finish(&mut self) {
        self.end_time = Some(chrono::Utc::now().timestamp_millis());
        if let Some(end) = self.end_time {
            self.duration_ms = (end - self.start_time) as u64;
        }
        self.calculate_stats();
    }

    fn calculate_stats(&mut self) {
        self.category_stats.clear();
        self.total_releasable_size = 0;

        for file in &self.candidates {
            if file.is_protected {
                continue;
            }

            self.total_releasable_size += file.size;

            for category in &file.categories {
                let key = category.to_key().to_string();
                let stats = self
                    .category_stats
                    .entry(key)
                    .or_insert_with(CategoryStats::default);
                stats.count += 1;
                stats.total_size += file.size;
            }
        }
    }
}

/// 分类统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStats {
    /// 文件数量
    pub count: u64,
    /// 总大小
    pub total_size: u64,
}

/// 磁盘空间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSpaceInfo {
    /// 路径
    pub path: String,
    /// 总容量（字节）
    pub total_space: u64,
    /// 已用空间（字节）
    pub used_space: u64,
    /// 可用空间（字节）
    pub available_space: u64,
}

impl DiskSpaceInfo {
    pub fn usage_percent(&self) -> f64 {
        if self.total_space == 0 {
            0.0
        } else {
            (self.used_space as f64 / self.total_space as f64) * 100.0
        }
    }
}
