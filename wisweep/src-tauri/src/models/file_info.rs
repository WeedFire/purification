use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 文件分类标签
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileCategory {
    /// 系统临时文件
    SystemTemp,
    /// 应用程序缓存
    AppCache,
    /// 日志文件
    Log,
    /// 构建产物
    BuildArtifact,
    /// 旧版备份文件
    OldBackup,
    /// 大文件
    LargeFile,
    /// 下载残留
    DownloadResidue,
    /// 空文件夹
    EmptyFolder,
    /// 系统保护文件
    SystemProtected,
    /// 其它
    Other,
}

impl FileCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            FileCategory::SystemTemp => "系统临时文件",
            FileCategory::AppCache => "应用程序缓存",
            FileCategory::Log => "日志文件",
            FileCategory::BuildArtifact => "构建产物",
            FileCategory::OldBackup => "旧版备份文件",
            FileCategory::LargeFile => "大文件",
            FileCategory::DownloadResidue => "下载残留",
            FileCategory::EmptyFolder => "空文件夹",
            FileCategory::SystemProtected => "系统保护文件",
            FileCategory::Other => "其它",
        }
    }

    /// 返回与前端 FileCategory 类型匹配的 snake_case 键名
    pub fn to_key(&self) -> &'static str {
        match self {
            FileCategory::SystemTemp => "system_temp",
            FileCategory::AppCache => "app_cache",
            FileCategory::Log => "log",
            FileCategory::BuildArtifact => "build_artifact",
            FileCategory::OldBackup => "old_backup",
            FileCategory::LargeFile => "large_file",
            FileCategory::DownloadResidue => "download_residue",
            FileCategory::EmptyFolder => "empty_folder",
            FileCategory::SystemProtected => "system_protected",
            FileCategory::Other => "other",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            FileCategory::SystemTemp => "#FF6B6B",
            FileCategory::AppCache => "#4ECDC4",
            FileCategory::Log => "#95E1D3",
            FileCategory::BuildArtifact => "#F38181",
            FileCategory::OldBackup => "#AA96DA",
            FileCategory::LargeFile => "#FCBAD3",
            FileCategory::DownloadResidue => "#A8D8EA",
            FileCategory::EmptyFolder => "#DFE6E9",
            FileCategory::SystemProtected => "#636E72",
            FileCategory::Other => "#B2BEC3",
        }
    }
}

/// 文件信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 文件大小（字节）
    pub size: u64,
    /// 是否为目录
    pub is_dir: bool,
    /// 创建时间（Unix 时间戳，毫秒）
    pub created_time: Option<i64>,
    /// 修改时间（Unix 时间戳，毫秒）
    pub modified_time: Option<i64>,
    /// 访问时间（Unix 时间戳，毫秒）
    pub accessed_time: Option<i64>,
    /// 文件扩展名
    pub extension: Option<String>,
    /// 分类标签列表
    pub categories: Vec<FileCategory>,
    /// 推荐权重（0-100）
    pub weight: u8,
    /// 是否被保护
    pub is_protected: bool,
    /// 保护原因
    pub protection_reason: Option<String>,
    /// 是否正在被使用
    pub is_in_use: bool,
    /// 推荐清理理由
    pub recommendation_reason: Option<String>,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> Self {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase());

        Self {
            path,
            size: 0,
            is_dir: false,
            created_time: None,
            modified_time: None,
            accessed_time: None,
            extension,
            categories: Vec::new(),
            weight: 0,
            is_protected: false,
            protection_reason: None,
            is_in_use: false,
            recommendation_reason: None,
        }
    }

    /// 格式化文件大小
    pub fn formatted_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.2} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.2} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}

/// 空文件夹信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyFolderInfo {
    /// 文件夹路径
    pub path: PathBuf,
    /// 目录层级深度
    pub depth: usize,
    /// 是否可以合并清理（父文件夹也仅包含空文件夹）
    pub can_merge: bool,
    /// 子空文件夹数量
    pub empty_children_count: usize,
}
