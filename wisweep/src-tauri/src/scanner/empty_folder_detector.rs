use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::models::EmptyFolderInfo;

/// 空文件夹检测器
pub struct EmptyFolderDetector {
    /// 最小深度
    min_depth: usize,
}

impl EmptyFolderDetector {
    pub fn new() -> Self {
        Self { min_depth: 0 }
    }

    /// 检测空文件夹
    pub fn detect(&self, root: &Path) -> Result<Vec<EmptyFolderInfo>, anyhow::Error> {
        let mut empty_folders = Vec::new();
        let mut visited = HashSet::new();

        self.detect_recursive(root, 0, &mut empty_folders, &mut visited)?;

        // 计算可合并的空文件夹
        self.calculate_merge_suggestions(&mut empty_folders);

        Ok(empty_folders)
    }

    /// 递归检测
    fn detect_recursive(
        &self,
        dir: &Path,
        depth: usize,
        results: &mut Vec<EmptyFolderInfo>,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<bool, anyhow::Error> {
        if visited.contains(dir) {
            return Ok(false);
        }
        visited.insert(dir.to_path_buf());

        let mut has_files = false;
        let mut empty_child_count = 0;

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // 递归检查子目录
                    let child_empty = self.detect_recursive(&path, depth + 1, results, visited)?;
                    if child_empty {
                        empty_child_count += 1;
                    } else {
                        has_files = true;
                    }
                } else {
                    has_files = true;
                }
            }
        }

        // 如果目录为空（没有文件，也没有非空子目录）
        if !has_files {
            results.push(EmptyFolderInfo {
                path: dir.to_path_buf(),
                depth,
                can_merge: empty_child_count > 0,
                empty_children_count: empty_child_count,
            });
            return Ok(true);
        }

        Ok(false)
    }

    /// 计算合并建议
    fn calculate_merge_suggestions(&self, folders: &mut [EmptyFolderInfo]) {
        // 按深度排序，从深到浅处理
        folders.sort_by(|a, b| b.depth.cmp(&a.depth));

        // 统计每个父目录的空子目录数量
        let mut parent_empty_count = std::collections::HashMap::new();

        for folder in folders.iter() {
            if let Some(parent) = folder.path.parent() {
                *parent_empty_count.entry(parent.to_path_buf()).or_insert(0) += 1;
            }
        }

        // 更新 can_merge 标志
        for folder in folders.iter_mut() {
            if folder.empty_children_count > 0 {
                // 检查是否所有子目录都是空的
                if let Some(count) = parent_empty_count.get(&folder.path) {
                    folder.can_merge = *count == folder.empty_children_count;
                }
            }
        }
    }
}

impl Default for EmptyFolderDetector {
    fn default() -> Self {
        Self::new()
    }
}
