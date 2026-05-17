use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;

use crate::models::{ClassificationRule, FileCategory, FileInfo};

use super::rules;

/// 文件分类引擎
pub struct FileClassifier {
    rules: Arc<RwLock<Vec<ClassificationRule>>>,
}

impl FileClassifier {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(ClassificationRule::built_in_rules())),
        }
    }

    /// 添加自定义规则
    pub fn add_rule(&self, rule: ClassificationRule) {
        self.rules.write().push(rule);
    }

    /// 移除规则
    pub fn remove_rule(&self, rule_id: &str) -> bool {
        let mut rules = self.rules.write();
        if let Some(pos) = rules.iter().position(|r| r.id == rule_id) {
            rules.remove(pos);
            true
        } else {
            false
        }
    }

    /// 获取所有规则
    pub fn get_rules(&self) -> Vec<ClassificationRule> {
        self.rules.read().clone()
    }

    /// 对文件进行分类
    pub fn classify(&self, file_info: &mut FileInfo) {
        let rules = self.rules.read();
        let path = &file_info.path;

        let mut max_weight = 0u8;
        let mut is_protected = false;
        let mut protection_reason = None;

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if rule.matches(path) {
                if rule.is_protection {
                    is_protected = true;
                    protection_reason = Some(format!("匹配保护规则: {}", rule.name));
                    if !file_info.categories.contains(&rule.category) {
                        file_info.categories.push(rule.category);
                    }
                } else {
                    let mut weight = rule.base_weight;

                    if let Some(threshold_days) = rule.time_threshold_days {
                        if let Some(modified_time) = file_info.modified_time {
                            let now = chrono::Utc::now().timestamp_millis();
                            let age_days = ((now - modified_time) / (1000 * 60 * 60 * 24)) as u32;
                            if age_days >= threshold_days {
                                weight = weight.saturating_add(rule.time_weight_bonus);
                            }
                        }
                    }

                    if let Some(size_threshold) = rule.size_threshold {
                        if file_info.size >= size_threshold {
                            weight = weight.saturating_add(rule.size_weight_bonus);
                        }
                    }

                    if weight > max_weight {
                        max_weight = weight;
                    }

                    if !file_info.categories.contains(&rule.category) {
                        file_info.categories.push(rule.category);
                    }
                }
            }
        }

        // 使用高级检测函数辅助分类（增强精准度）
        if !is_protected {
            // 浏览缓存检测
            if !file_info.categories.contains(&FileCategory::AppCache)
                && rules::is_browser_cache(path)
            {
                file_info.categories.push(FileCategory::AppCache);
                if max_weight < 80 { max_weight = 80; }
            }

            // 构建产物检测
            if !file_info.categories.contains(&FileCategory::BuildArtifact)
                && rules::is_build_cache(path)
            {
                file_info.categories.push(FileCategory::BuildArtifact);
                if max_weight < 95 { max_weight = 95; }
            }

            // 包管理器缓存检测
            if !file_info.categories.contains(&FileCategory::AppCache)
                && rules::is_package_cache(path)
            {
                file_info.categories.push(FileCategory::AppCache);
                if max_weight < 85 { max_weight = 85; }
            }

            // IDE 缓存检测
            if !file_info.categories.contains(&FileCategory::AppCache)
                && rules::is_ide_cache(path)
            {
                file_info.categories.push(FileCategory::AppCache);
                if max_weight < 75 { max_weight = 75; }
            }

            // 系统垃圾文件检测
            if rules::is_system_junk(path) {
                if !file_info.categories.contains(&FileCategory::SystemTemp) {
                    file_info.categories.push(FileCategory::SystemTemp);
                }
                if max_weight < 90 { max_weight = 90; }
            }

            // 下载残留检测
            if !file_info.categories.contains(&FileCategory::DownloadResidue)
                && rules::is_download_residue(path)
            {
                file_info.categories.push(FileCategory::DownloadResidue);
                if max_weight < 90 { max_weight = 90; }
            }
        }

        file_info.weight = max_weight;
        file_info.is_protected = is_protected;
        file_info.protection_reason = protection_reason;

        if !is_protected && file_info.categories.is_empty() {
            file_info.categories.push(FileCategory::Other);
        }

        if !file_info.is_protected && !file_info.categories.is_empty() {
            file_info.recommendation_reason = Some(generate_recommendation_reason(file_info));
        }
    }

    /// 批量分类
    pub fn classify_batch(&self, files: &mut [FileInfo]) {
        for file in files.iter_mut() {
            self.classify(file);
        }
    }
}

impl Default for FileClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// 生成推荐理由
fn generate_recommendation_reason(file: &FileInfo) -> String {
    let mut reasons = Vec::new();

    for category in &file.categories {
        match category {
            FileCategory::SystemTemp => {
                reasons.push("系统临时文件".to_string());
            }
            FileCategory::AppCache => {
                reasons.push("应用程序缓存".to_string());
            }
            FileCategory::Log => {
                reasons.push("日志文件".to_string());
            }
            FileCategory::BuildArtifact => {
                reasons.push("构建产物/依赖目录".to_string());
            }
            FileCategory::OldBackup => {
                reasons.push("旧版备份文件".to_string());
            }
            FileCategory::LargeFile => {
                reasons.push(format!("大文件 ({})", file.formatted_size()));
            }
            FileCategory::DownloadResidue => {
                reasons.push("未完成的下载文件".to_string());
            }
            _ => {}
        }
    }

    if reasons.is_empty() {
        "可清理文件".to_string()
    } else {
        reasons.join(", ")
    }
}
