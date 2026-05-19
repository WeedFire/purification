use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// 数据库管理器
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// 初始化数据库表
    fn initialize(&self) -> Result<(), anyhow::Error> {
        self.conn.execute_batch(
            r#"
            -- 扫描历史表
            CREATE TABLE IF NOT EXISTS scan_history (
                id TEXT PRIMARY KEY,
                paths TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                duration_ms INTEGER DEFAULT 0,
                total_files INTEGER DEFAULT 0,
                total_dirs INTEGER DEFAULT 0,
                candidate_count INTEGER DEFAULT 0,
                candidate_size INTEGER DEFAULT 0,
                empty_folder_count INTEGER DEFAULT 0,
                cleaned_count INTEGER DEFAULT 0,
                cleaned_size INTEGER DEFAULT 0,
                config TEXT
            );

            -- 路径收藏表
            CREATE TABLE IF NOT EXISTS favorite_paths (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                alias TEXT,
                created_at INTEGER NOT NULL
            );

            -- 自定义规则表
            CREATE TABLE IF NOT EXISTS custom_rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                pattern_type TEXT NOT NULL,
                patterns TEXT NOT NULL,
                category TEXT NOT NULL,
                base_weight INTEGER DEFAULT 50,
                time_threshold_days INTEGER,
                time_weight_bonus INTEGER DEFAULT 0,
                size_threshold INTEGER,
                size_weight_bonus INTEGER DEFAULT 0,
                is_protection INTEGER DEFAULT 0,
                enabled INTEGER DEFAULT 1,
                description TEXT,
                created_at INTEGER NOT NULL
            );

            -- 清理记录表
            CREATE TABLE IF NOT EXISTS cleanup_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scan_id TEXT NOT NULL,
                cleanup_time INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                category TEXT,
                delete_mode TEXT NOT NULL,
                result TEXT NOT NULL,
                error_message TEXT,
                FOREIGN KEY (scan_id) REFERENCES scan_history(id)
            );

            -- 创建索引
            CREATE INDEX IF NOT EXISTS idx_scan_history_start_time ON scan_history(start_time);
            CREATE INDEX IF NOT EXISTS idx_favorite_paths_path ON favorite_paths(path);
            CREATE INDEX IF NOT EXISTS idx_cleanup_log_scan_id ON cleanup_log(scan_id);

            -- 系统保护路径表（用户可自定义）
            CREATE TABLE IF NOT EXISTS protected_paths (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                description TEXT,
                is_system INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL
            );

            -- 插入默认的 Windows 系统保护路径（仅当表为空时）
            INSERT OR IGNORE INTO protected_paths (path, description, is_system, created_at)
            SELECT '\Windows\', 'Windows 系统目录', 1, 0 WHERE NOT EXISTS (SELECT 1 FROM protected_paths);
            INSERT OR IGNORE INTO protected_paths (path, description, is_system, created_at)
            SELECT '\Program Files\', '程序安装目录', 1, 0 WHERE NOT EXISTS (SELECT 1 FROM protected_paths);
            INSERT OR IGNORE INTO protected_paths (path, description, is_system, created_at)
            SELECT '\Program Files (x86)\', '32位程序安装目录', 1, 0 WHERE NOT EXISTS (SELECT 1 FROM protected_paths);
            INSERT OR IGNORE INTO protected_paths (path, description, is_system, created_at)
            SELECT '\ProgramData\', '程序数据目录', 1, 0 WHERE NOT EXISTS (SELECT 1 FROM protected_paths);
            "#,
        )?;
        Ok(())
    }

    /// 保存扫描历史
    pub fn save_scan_history(&self, history: &ScanHistoryRecord) -> Result<(), anyhow::Error> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO scan_history 
            (id, paths, start_time, end_time, duration_ms, total_files, total_dirs, 
             candidate_count, candidate_size, empty_folder_count, cleaned_count, cleaned_size, config)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                history.id.to_string(),
                serde_json::to_string(&history.paths)?,
                history.start_time,
                history.end_time,
                history.duration_ms,
                history.total_files,
                history.total_dirs,
                history.candidate_count,
                history.candidate_size,
                history.empty_folder_count,
                history.cleaned_count,
                history.cleaned_size,
                history.config,
            ],
        )?;
        Ok(())
    }

    /// 获取扫描历史列表
    pub fn get_scan_history(&self, limit: usize) -> Result<Vec<ScanHistoryRecord>, anyhow::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM scan_history ORDER BY start_time DESC LIMIT ?1")?;

        let records = stmt
            .query_map(params![limit as i64], |row| {
                Ok(ScanHistoryRecord {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                    paths: serde_json::from_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                    start_time: row.get(2)?,
                    end_time: row.get(3)?,
                    duration_ms: row.get(4)?,
                    total_files: row.get(5)?,
                    total_dirs: row.get(6)?,
                    candidate_count: row.get(7)?,
                    candidate_size: row.get(8)?,
                    empty_folder_count: row.get(9)?,
                    cleaned_count: row.get(10)?,
                    cleaned_size: row.get(11)?,
                    config: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// 添加收藏路径
    pub fn add_favorite_path(&self, path: &str, alias: Option<&str>) -> Result<(), anyhow::Error> {
        self.conn.execute(
            "INSERT OR IGNORE INTO favorite_paths (path, alias, created_at) VALUES (?1, ?2, ?3)",
            params![path, alias, chrono::Utc::now().timestamp_millis()],
        )?;
        Ok(())
    }

    /// 移除收藏路径
    pub fn remove_favorite_path(&self, path: &str) -> Result<(), anyhow::Error> {
        self.conn
            .execute("DELETE FROM favorite_paths WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// 获取收藏路径列表
    pub fn get_favorite_paths(&self) -> Result<Vec<FavoritePath>, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, alias, created_at FROM favorite_paths ORDER BY created_at DESC",
        )?;

        let paths = stmt
            .query_map([], |row| {
                Ok(FavoritePath {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    alias: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(paths)
    }

    /// 保存清理记录
    pub fn save_cleanup_log(&self, log: &CleanupLogRecord) -> Result<(), anyhow::Error> {
        self.conn.execute(
            r#"
            INSERT INTO cleanup_log 
            (scan_id, cleanup_time, file_path, file_size, category, delete_mode, result, error_message)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                log.scan_id.to_string(),
                log.cleanup_time,
                log.file_path,
                log.file_size,
                log.category,
                log.delete_mode,
                log.result,
                log.error_message,
            ],
        )?;
        Ok(())
    }

    /// 获取指定扫描的清理日志
    pub fn get_cleanup_logs(&self, scan_id: &Uuid) -> Result<Vec<CleanupLogRecord>, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, scan_id, cleanup_time, file_path, file_size, category, delete_mode, result, error_message
             FROM cleanup_log WHERE scan_id = ?1 ORDER BY cleanup_time DESC"
        )?;

        let logs = stmt
            .query_map(params![scan_id.to_string()], |row| {
                Ok(CleanupLogRecord {
                    scan_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                    cleanup_time: row.get(2)?,
                    file_path: row.get(3)?,
                    file_size: row.get(4)?,
                    category: row.get(5)?,
                    delete_mode: row.get(6)?,
                    result: row.get(7)?,
                    error_message: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// 删除单条扫描历史及关联的清理日志
    pub fn delete_scan_history(&self, scan_id: &Uuid) -> Result<(), anyhow::Error> {
        self.conn.execute(
            "DELETE FROM cleanup_log WHERE scan_id = ?1",
            params![scan_id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM scan_history WHERE id = ?1",
            params![scan_id.to_string()],
        )?;
        Ok(())
    }

    /// 清空所有扫描历史和清理日志
    pub fn clear_all_history(&self) -> Result<(), anyhow::Error> {
        self.conn.execute_batch(
            "DELETE FROM cleanup_log; DELETE FROM scan_history;"
        )?;
        Ok(())
    }

    /// 获取所有保护路径
    pub fn get_protected_paths(&self) -> Result<Vec<ProtectedPath>, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, description, is_system, created_at FROM protected_paths ORDER BY is_system DESC, id ASC"
        )?;

        let paths = stmt
            .query_map([], |row| {
                Ok(ProtectedPath {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    description: row.get(2)?,
                    is_system: row.get::<_, i32>(3)? == 1,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(paths)
    }

    /// 添加保护路径
    pub fn add_protected_path(&self, path: &str, description: Option<&str>) -> Result<(), anyhow::Error> {
        self.conn.execute(
            "INSERT OR IGNORE INTO protected_paths (path, description, is_system, created_at) VALUES (?1, ?2, 0, ?3)",
            params![path, description, chrono::Utc::now().timestamp_millis()],
        )?;
        Ok(())
    }

    /// 移除保护路径
    pub fn remove_protected_path(&self, id: i64) -> Result<(), anyhow::Error> {
        self.conn.execute(
            "DELETE FROM protected_paths WHERE id = ?1 AND is_system = 0",
            params![id],
        )?;
        Ok(())
    }

    /// 重置为默认系统保护路径（删除用户自定义的）
    pub fn reset_protected_paths(&self) -> Result<(), anyhow::Error> {
        self.conn.execute(
            "DELETE FROM protected_paths WHERE is_system = 0",
            [],
        )?;
        Ok(())
    }
}

/// 扫描历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanHistoryRecord {
    pub id: Uuid,
    pub paths: Vec<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_ms: u64,
    pub total_files: u64,
    pub total_dirs: u64,
    pub candidate_count: u64,
    pub candidate_size: u64,
    pub empty_folder_count: u64,
    pub cleaned_count: u64,
    pub cleaned_size: u64,
    pub config: Option<String>,
}

/// 收藏路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoritePath {
    pub id: i64,
    pub path: String,
    pub alias: Option<String>,
    pub created_at: i64,
}

/// 清理日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupLogRecord {
    pub scan_id: Uuid,
    pub cleanup_time: i64,
    pub file_path: String,
    pub file_size: u64,
    pub category: Option<String>,
    pub delete_mode: String,
    pub result: String,
    pub error_message: Option<String>,
}

/// 保护路径记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedPath {
    pub id: i64,
    pub path: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: i64,
}
