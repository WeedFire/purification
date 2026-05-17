// 文件分类
export type FileCategory =
  | 'system_temp'
  | 'app_cache'
  | 'log'
  | 'build_artifact'
  | 'old_backup'
  | 'large_file'
  | 'download_residue'
  | 'empty_folder'
  | 'system_protected'
  | 'other';

export const CategoryLabels: Record<FileCategory, string> = {
  system_temp: '系统临时文件',
  app_cache: '应用程序缓存',
  log: '日志文件',
  build_artifact: '构建产物',
  old_backup: '旧版备份文件',
  large_file: '大文件',
  download_residue: '下载残留',
  empty_folder: '空文件夹',
  system_protected: '系统保护文件',
  other: '其它',
};

export const CategoryColors: Record<FileCategory, string> = {
  system_temp: '#FF6B6B',
  app_cache: '#4ECDC4',
  log: '#95E1D3',
  build_artifact: '#F38181',
  old_backup: '#AA96DA',
  large_file: '#FCBAD3',
  download_residue: '#A8D8EA',
  empty_folder: '#DFE6E9',
  system_protected: '#636E72',
  other: '#B2BEC3',
};

// 文件信息
export interface FileInfo {
  path: string;
  size: number;
  is_dir: boolean;
  created_time: number | null;
  modified_time: number | null;
  accessed_time: number | null;
  extension: string | null;
  categories: FileCategory[];
  weight: number;
  is_protected: boolean;
  protection_reason: string | null;
  is_in_use: boolean;
  recommendation_reason: string | null;
}

// 空文件夹信息
export interface EmptyFolderInfo {
  path: string;
  depth: number;
  can_merge: boolean;
  empty_children_count: number;
}

// 扫描配置
export interface ScanConfig {
  paths: string[];
  recursive: boolean;
  max_depth: number | null;
  include_hidden: boolean;
  include_system: boolean;
  min_file_size: number;
  exclude_patterns: string[];
  large_file_threshold: number;
  temp_file_age_days: number;
  scan_empty_folders: boolean;
  detect_duplicates: boolean;
}

// 扫描进度
export interface ScanProgress {
  current_path: string;
  scanned_count: number;
  candidate_count: number;
  empty_folder_count: number;
  dir_count: number;
  is_scanning: boolean;
  is_paused: boolean;
}

// 扫描结果
export interface ScanResult {
  scan_id: string;
  paths: string[];
  start_time: number;
  end_time: number | null;
  duration_ms: number;
  total_files: number;
  total_dirs: number;
  candidates: FileInfo[];
  empty_folders: EmptyFolderInfo[];
  category_stats: Record<string, CategoryStats>;
  total_releasable_size: number;
  cleaned_count: number;
  cleaned_size: number;
  config: string;
}

export interface CategoryStats {
  count: number;
  total_size: number;
}

// 磁盘空间信息
export interface DiskSpaceInfo {
  path: string;
  total_space: number;
  used_space: number;
  available_space: number;
}

// 清理模式
export type CleanupMode = 'recycle_bin' | 'permanent' | 'secure_wipe';

export const CleanupModeLabels: Record<CleanupMode, string> = {
  recycle_bin: '移至回收站',
  permanent: '永久删除',
  secure_wipe: '安全擦除',
};

// 清理结果
export interface CleanupResult {
  scan_id: string;
  cleanup_time: number;
  mode: CleanupMode;
  success_items: CleanupItem[];
  failed_items: FailedCleanupItem[];
  success_count: number;
  failed_count: number;
  released_size: number;
}

export interface CleanupItem {
  path: string;
  size: number;
  cleanup_time: number;
}

export interface FailedCleanupItem {
  path: string;
  size: number;
  reason: string;
}

// 清理进度
export interface CleanupProgress {
  current_file: string;
  processed_count: number;
  total_count: number;
  released_size: number;
  is_cleaning: boolean;
}

// 分类规则
export interface ClassificationRule {
  id: string;
  name: string;
  pattern_type: 'glob' | 'regex' | 'path_contains';
  patterns: string[];
  category: FileCategory;
  base_weight: number;
  time_threshold_days: number | null;
  time_weight_bonus: number;
  size_threshold: number | null;
  size_weight_bonus: number;
  is_protection: boolean;
  enabled: boolean;
  description: string | null;
}

// 收藏路径
export interface FavoritePath {
  id: number;
  path: string;
  alias: string | null;
  created_at: number;
}
