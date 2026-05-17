use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// 清理模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupMode {
    /// 移至回收站（可恢复）
    RecycleBin,
    /// 永久删除
    Permanent,
    /// 安全擦除（不可恢复）
    SecureWipe,
}

impl Default for CleanupMode {
    fn default() -> Self {
        Self::RecycleBin
    }
}

impl CleanupMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            CleanupMode::RecycleBin => "移至回收站",
            CleanupMode::Permanent => "永久删除",
            CleanupMode::SecureWipe => "安全擦除",
        }
    }
}

/// 清理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    /// 关联的扫描ID
    pub scan_id: Uuid,
    /// 清理时间
    pub cleanup_time: i64,
    /// 清理模式
    pub mode: CleanupMode,
    /// 成功清理的文件列表
    pub success_items: Vec<CleanupItem>,
    /// 失败的文件列表
    pub failed_items: Vec<FailedCleanupItem>,
    /// 成功清理的文件数
    pub success_count: u64,
    /// 失败的文件数
    pub failed_count: u64,
    /// 成功释放的空间（字节）
    pub released_size: u64,
}

impl CleanupResult {
    pub fn new(scan_id: Uuid, mode: CleanupMode) -> Self {
        Self {
            scan_id,
            cleanup_time: chrono::Utc::now().timestamp_millis(),
            mode,
            success_items: Vec::new(),
            failed_items: Vec::new(),
            success_count: 0,
            failed_count: 0,
            released_size: 0,
        }
    }
}

/// 清理项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupItem {
    /// 文件路径
    pub path: PathBuf,
    /// 文件大小
    pub size: u64,
    /// 清理时间
    pub cleanup_time: i64,
}

/// 失败的清理项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedCleanupItem {
    /// 文件路径
    pub path: PathBuf,
    /// 文件大小
    pub size: u64,
    /// 失败原因
    pub reason: String,
}

/// 清理进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupProgress {
    /// 当前处理的文件
    pub current_file: String,
    /// 已处理数量
    pub processed_count: u64,
    /// 总数量
    pub total_count: u64,
    /// 已释放空间
    pub released_size: u64,
    /// 是否正在清理
    pub is_cleaning: bool,
}

impl Default for CleanupProgress {
    fn default() -> Self {
        Self {
            current_file: String::new(),
            processed_count: 0,
            total_count: 0,
            released_size: 0,
            is_cleaning: false,
        }
    }
}
