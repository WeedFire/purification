use parking_lot::RwLock;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// 扫描进度追踪器
#[derive(Debug)]
pub struct ScanProgressTracker {
    current_path: RwLock<String>,
    scanned_count: AtomicU64,
    candidate_count: AtomicU64,
    empty_folder_count: AtomicU64,
    dir_count: AtomicU64,
    is_scanning: AtomicBool,
    is_paused: AtomicBool,
}

impl ScanProgressTracker {
    pub fn new() -> Self {
        Self {
            current_path: RwLock::new(String::new()),
            scanned_count: AtomicU64::new(0),
            candidate_count: AtomicU64::new(0),
            empty_folder_count: AtomicU64::new(0),
            dir_count: AtomicU64::new(0),
            is_scanning: AtomicBool::new(false),
            is_paused: AtomicBool::new(false),
        }
    }

    pub fn reset(&self) {
        *self.current_path.write() = String::new();
        self.scanned_count.store(0, Ordering::SeqCst);
        self.candidate_count.store(0, Ordering::SeqCst);
        self.empty_folder_count.store(0, Ordering::SeqCst);
        self.dir_count.store(0, Ordering::SeqCst);
        self.is_scanning.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn set_current_path(&self, path: String) {
        *self.current_path.write() = path;
    }

    pub fn increment_scanned(&self) {
        self.scanned_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_candidates(&self) {
        self.candidate_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_empty_folders(&self) {
        self.empty_folder_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_dirs(&self) {
        self.dir_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn set_scanning(&self, value: bool) {
        self.is_scanning.store(value, Ordering::SeqCst);
    }

    pub fn set_paused(&self, value: bool) {
        self.is_paused.store(value, Ordering::SeqCst);
    }

    pub fn get_progress(&self) -> ScanProgressSnapshot {
        ScanProgressSnapshot {
            current_path: self.current_path.read().clone(),
            scanned_count: self.scanned_count.load(Ordering::SeqCst),
            candidate_count: self.candidate_count.load(Ordering::SeqCst),
            empty_folder_count: self.empty_folder_count.load(Ordering::SeqCst),
            dir_count: self.dir_count.load(Ordering::SeqCst),
            is_scanning: self.is_scanning.load(Ordering::SeqCst),
            is_paused: self.is_paused.load(Ordering::SeqCst),
        }
    }
}

impl Default for ScanProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 扫描进度快照
#[derive(Debug, Clone, Serialize)]
pub struct ScanProgressSnapshot {
    pub current_path: String,
    pub scanned_count: u64,
    pub candidate_count: u64,
    pub empty_folder_count: u64,
    pub dir_count: u64,
    pub is_scanning: bool,
    pub is_paused: bool,
}
