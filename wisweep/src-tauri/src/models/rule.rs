use serde::{Deserialize, Serialize};
use std::path::Path;

use super::FileCategory;

/// 匹配模式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchPatternType {
    /// Glob 模式
    Glob,
    /// 正则表达式
    Regex,
    /// 路径包含
    PathContains,
}

/// 分类规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 匹配模式类型
    pub pattern_type: MatchPatternType,
    /// 匹配模式列表
    pub patterns: Vec<String>,
    /// 分类标签
    pub category: FileCategory,
    /// 基础权重（0-100）
    pub base_weight: u8,
    /// 时间阈值（天），超过此时间增加权重
    pub time_threshold_days: Option<u32>,
    /// 时间阈值增加的权重
    pub time_weight_bonus: u8,
    /// 大小阈值（字节），超过此大小增加权重
    pub size_threshold: Option<u64>,
    /// 大小阈值增加的权重
    pub size_weight_bonus: u8,
    /// 是否为保护规则
    pub is_protection: bool,
    /// 是否启用
    pub enabled: bool,
    /// 规则描述
    pub description: Option<String>,
}

impl ClassificationRule {
    /// 创建临时文件规则
    pub fn temp_files() -> Vec<Self> {
        vec![Self {
            id: "TMP_001".to_string(),
            name: "临时文件".to_string(),
            pattern_type: MatchPatternType::Glob,
            patterns: vec!["*.tmp".to_string(), "*._mp".to_string(), "~*.*".to_string()],
            category: FileCategory::SystemTemp,
            base_weight: 90,
            time_threshold_days: Some(7),
            time_weight_bonus: 5,
            size_threshold: None,
            size_weight_bonus: 0,
            is_protection: false,
            enabled: true,
            description: Some("系统临时文件，通常可以安全删除".to_string()),
        }]
    }

    /// 创建日志文件规则
    pub fn log_files() -> Vec<Self> {
        vec![Self {
            id: "LOG_001".to_string(),
            name: "日志文件".to_string(),
            pattern_type: MatchPatternType::Glob,
            patterns: vec!["*.log".to_string(), "*.log.*".to_string()],
            category: FileCategory::Log,
            base_weight: 60,
            time_threshold_days: None,
            time_weight_bonus: 0,
            size_threshold: Some(100 * 1024 * 1024), // 100MB
            size_weight_bonus: 30,
            is_protection: false,
            enabled: true,
            description: Some("应用程序日志文件".to_string()),
        }]
    }

    /// 创建构建产物规则
    pub fn build_artifacts() -> Vec<Self> {
        vec![Self {
            id: "BUILD_001".to_string(),
            name: "构建产物".to_string(),
            pattern_type: MatchPatternType::PathContains,
            patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".venv".to_string(),
                "build".to_string(),
                "dist".to_string(),
                "__pycache__".to_string(),
                ".gradle".to_string(),
                "Pods".to_string(),
            ],
            category: FileCategory::BuildArtifact,
            base_weight: 95,
            time_threshold_days: None,
            time_weight_bonus: 0,
            size_threshold: None,
            size_weight_bonus: 0,
            is_protection: false,
            enabled: true,
            description: Some("开发构建产物和依赖目录".to_string()),
        }]
    }

    /// 创建备份文件规则
    pub fn backup_files() -> Vec<Self> {
        vec![Self {
            id: "BAK_001".to_string(),
            name: "备份文件".to_string(),
            pattern_type: MatchPatternType::Glob,
            patterns: vec![
                "*.old".to_string(),
                "*.bak".to_string(),
                "*.backup".to_string(),
            ],
            category: FileCategory::OldBackup,
            base_weight: 85,
            time_threshold_days: Some(30),
            time_weight_bonus: 10,
            size_threshold: None,
            size_weight_bonus: 0,
            is_protection: false,
            enabled: true,
            description: Some("旧版备份文件".to_string()),
        }]
    }

    /// 创建下载残留规则
    pub fn download_residue() -> Vec<Self> {
        vec![Self {
            id: "DOWN_001".to_string(),
            name: "下载残留".to_string(),
            pattern_type: MatchPatternType::Glob,
            patterns: vec![
                "*.aria2".to_string(),
                "*.td".to_string(),
                "*.crdownload".to_string(),
                "*.part".to_string(),
            ],
            category: FileCategory::DownloadResidue,
            base_weight: 95,
            time_threshold_days: Some(1),
            time_weight_bonus: 5,
            size_threshold: None,
            size_weight_bonus: 0,
            is_protection: false,
            enabled: true,
            description: Some("未完成的下载文件".to_string()),
        }]
    }

    /// 创建系统保护规则
    pub fn system_protection() -> Vec<Self> {
        let mut rules = Vec::new();

        #[cfg(windows)]
        {
            rules.push(Self {
                id: "SYS_PROTECT_WIN".to_string(),
                name: "Windows 系统保护".to_string(),
                pattern_type: MatchPatternType::PathContains,
                patterns: vec![
                    "\\Windows\\".to_string(),
                    "\\Program Files\\".to_string(),
                    "\\Program Files (x86)\\".to_string(),
                    "\\ProgramData\\".to_string(),
                ],
                category: FileCategory::SystemProtected,
                base_weight: 0,
                time_threshold_days: None,
                time_weight_bonus: 0,
                size_threshold: None,
                size_weight_bonus: 0,
                is_protection: true,
                enabled: true,
                description: Some("Windows 系统关键路径".to_string()),
            });
        }

        #[cfg(target_os = "macos")]
        {
            rules.push(Self {
                id: "SYS_PROTECT_MAC".to_string(),
                name: "macOS 系统保护".to_string(),
                pattern_type: MatchPatternType::PathContains,
                patterns: vec![
                    "/System/".to_string(),
                    "/Library/".to_string(),
                    "/usr/".to_string(),
                    "/bin/".to_string(),
                    "/sbin/".to_string(),
                ],
                category: FileCategory::SystemProtected,
                base_weight: 0,
                time_threshold_days: None,
                time_weight_bonus: 0,
                size_threshold: None,
                size_weight_bonus: 0,
                is_protection: true,
                enabled: true,
                description: Some("macOS 系统关键路径".to_string()),
            });
        }

        #[cfg(target_os = "linux")]
        {
            rules.push(Self {
                id: "SYS_PROTECT_LINUX".to_string(),
                name: "Linux 系统保护".to_string(),
                pattern_type: MatchPatternType::PathContains,
                patterns: vec![
                    "/bin/".to_string(),
                    "/sbin/".to_string(),
                    "/usr/".to_string(),
                    "/lib/".to_string(),
                    "/lib64/".to_string(),
                    "/boot/".to_string(),
                    "/etc/".to_string(),
                ],
                category: FileCategory::SystemProtected,
                base_weight: 0,
                time_threshold_days: None,
                time_weight_bonus: 0,
                size_threshold: None,
                size_weight_bonus: 0,
                is_protection: true,
                enabled: true,
                description: Some("Linux 系统关键路径".to_string()),
            });
        }

        rules
    }

    /// 获取所有内置规则
    pub fn built_in_rules() -> Vec<Self> {
        let mut rules = Vec::new();
        rules.extend(Self::temp_files());
        rules.extend(Self::log_files());
        rules.extend(Self::build_artifacts());
        rules.extend(Self::backup_files());
        rules.extend(Self::download_residue());
        rules.extend(Self::system_protection());
        rules
    }

    /// 检查路径是否匹配规则
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.patterns {
            let matched = match self.pattern_type {
                MatchPatternType::Glob => {
                    if let Ok(glob) = glob::Pattern::new(pattern) {
                        glob.matches(&path_str)
                    } else {
                        false
                    }
                }
                MatchPatternType::Regex => {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        re.is_match(&path_str)
                    } else {
                        false
                    }
                }
                MatchPatternType::PathContains => path_str.contains(pattern),
            };

            if matched {
                return true;
            }
        }

        false
    }
}
