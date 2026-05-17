use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// 扫描路径列表
    pub paths: Vec<PathBuf>,
    /// 是否递归扫描子目录
    pub recursive: bool,
    /// 最大递归深度（0 表示无限制）
    pub max_depth: Option<usize>,
    /// 是否包含隐藏文件
    pub include_hidden: bool,
    /// 是否包含系统文件
    pub include_system: bool,
    /// 最小文件大小过滤（字节）
    pub min_file_size: u64,
    /// 排除路径模式（glob）
    pub exclude_patterns: Vec<String>,
    /// 大文件阈值（字节）
    pub large_file_threshold: u64,
    /// 临时文件时间阈值（天）
    pub temp_file_age_days: u32,
    /// 是否扫描空文件夹
    pub scan_empty_folders: bool,
    /// 是否检测重复文件
    pub detect_duplicates: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
            max_depth: None,
            include_hidden: true,
            include_system: false,
            min_file_size: 1024, // 1KB
            exclude_patterns: vec![
                "*.git/*".to_string(),
                "*.svn/*".to_string(),
                "*.hg/*".to_string(),
            ],
            large_file_threshold: 100 * 1024 * 1024, // 100MB
            temp_file_age_days: 7,
            scan_empty_folders: true,
            detect_duplicates: false,
        }
    }
}

impl ScanConfig {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            paths,
            ..Default::default()
        }
    }
}
