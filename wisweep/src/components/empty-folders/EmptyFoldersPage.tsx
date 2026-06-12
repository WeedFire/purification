import { useState, useEffect, useMemo } from 'react';
import { useAppStore } from '../../stores';
import { invoke } from '@tauri-apps/api/core';
import {
  FolderOpen, CheckSquare, Square, Trash2, CheckCircle,
  ChevronDown, ChevronRight, Folder, ExternalLink, AlertTriangle,
} from 'lucide-react';
import './EmptyFoldersPage.css';

const PAGE_SIZE = 200;

export function EmptyFoldersPage() {
  // ========== 所有 hooks 必须在最顶部 ==========
  const { scanResult, isLoadingResult, hasScanned, setActiveTab, removeEmptyFolders, openFileLocation } = useAppStore();
  const [selectedFolders, setSelectedFolders] = useState<Set<string>>(new Set());
  const [deleting, setDeleting] = useState(false);
  const [done, setDone] = useState(false);
  const [deletedCount, setDeletedCount] = useState(0);
  const [failedCount, setFailedCount] = useState(0);
  const [needAdmin, setNeedAdmin] = useState(false);
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());
  const [visiblePages, setVisiblePages] = useState<Record<string, number>>({});

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout>;
    if (done) {
      timer = setTimeout(() => { setDone(false); setNeedAdmin(false); }, 5000);
    }
    return () => { if (timer) clearTimeout(timer); };
  }, [done]);

  // 按目录分组
  const dirGroups = useMemo(() => {
    if (!scanResult?.empty_folders) return [];
    const groups: Record<string, string[]> = {};
    for (const f of scanResult.empty_folders) {
      const sep = f.path.includes('\\') ? '\\' : '/';
      const parent = f.path.substring(0, f.path.lastIndexOf(sep) + 1) || '/';
      if (!groups[parent]) groups[parent] = [];
      groups[parent].push(f.path);
    }
    return Object.entries(groups).sort(([a], [b]) => a.localeCompare(b));
  }, [scanResult]);

  // ========== early returns 放在最后 ==========
  const shouldShowLoading = isLoadingResult || (hasScanned && !scanResult);

  if (shouldShowLoading) {
    return (
      <div className="empty-folders-page loading">
        <div className="loading-state">
          <div className="loading-spinner"></div>
          <h3>正在加载扫描结果...</h3>
          <p>数据较多，请稍候</p>
        </div>
      </div>
    );
  }

  if (!scanResult || !scanResult.empty_folders || scanResult.empty_folders.length === 0) {
    return (
      <div className="empty-folders-page empty">
        <div className="empty-state">
          <FolderOpen size={64} />
          <h3>暂无空文件夹</h3>
          <p>请先扫描路径，系统会自动检测空文件夹</p>
          <button className="btn btn-primary" onClick={() => setActiveTab('scan')}>
            开始扫描
          </button>
        </div>
      </div>
    );
  }

  const toggleFolder = (path: string) => {
    setSelectedFolders(prev => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const selectAllInDir = (dirPath: string) => {
    const dirEntry = dirGroups.find(([d]) => d === dirPath);
    if (!dirEntry) return;
    const paths = dirEntry[1];
    setSelectedFolders(prev => {
      const next = new Set(prev);
      const allSelected = paths.every(p => prev.has(p));
      if (allSelected) {
        paths.forEach(p => next.delete(p));
      } else {
        paths.forEach(p => next.add(p));
      }
      return next;
    });
  };

  const selectAll = () => {
    const allPaths = scanResult.empty_folders.map(f => f.path);
    setSelectedFolders(new Set(allPaths));
  };

  const deselectAll = () => setSelectedFolders(new Set());

  const toggleDir = (dir: string) => {
    setExpandedDirs(prev => {
      const next = new Set(prev);
      if (next.has(dir)) next.delete(dir);
      else next.add(dir);
      return next;
    });
  };

  const handleDelete = async () => {
    if (selectedFolders.size === 0) return;
    const pathsToDelete = Array.from(selectedFolders);
    setDeleting(true);
    setNeedAdmin(false);
    try {
      const failed = await invoke<string[]>('delete_empty_folders', { dirPaths: pathsToDelete });
      removeEmptyFolders(pathsToDelete);
      const successCount = pathsToDelete.length - failed.length;
      setDeletedCount(successCount);
      setFailedCount(failed.length);
      if (failed.length > 0) {
        setNeedAdmin(true);
      }
      setDone(true);
      setSelectedFolders(new Set());
    } catch (e) {
      alert(`删除失败: ${e}`);
    } finally {
      setDeleting(false);
    }
  };

  const getDirVisible = (dirPath: string, total: number) => {
    const pages = visiblePages[dirPath] || 1;
    return Math.min(pages * PAGE_SIZE, total);
  };

  const dirName = (fullPath: string) => {
    const cleaned = fullPath.replace(/[/\\]$/, '');
    return cleaned.split(/[/\\]/).pop() || cleaned || '(根目录)';
  };

  return (
    <div className="empty-folders-page">
      <div className="page-header">
        <h2>空文件夹</h2>
        <span className="folder-count">
          发现 {scanResult.empty_folders.length} 个空文件夹
        </span>
      </div>

      <div className="action-bar">
        <div className="selection-actions">
          <button className="btn btn-sm" onClick={selectAll}>全选</button>
          <button className="btn btn-sm" onClick={deselectAll}>取消全选</button>
          <span className="selected-count">
            已选 {selectedFolders.size} 个
          </span>
        </div>
        <button
          className="btn btn-danger"
          onClick={handleDelete}
          disabled={selectedFolders.size === 0 || deleting}
        >
          <Trash2 size={18} />
          {deleting ? '删除中...' : `删除选中 (${selectedFolders.size})`}
        </button>
      </div>

      {done && (
        <div className={`toast-success${needAdmin ? ' toast-warn' : ''}`}>
          {needAdmin ? (
            <>
              <AlertTriangle size={18} />
              删除完成：成功 {deletedCount} 个，失败 {failedCount} 个。
              请以<b>管理员身份</b>运行本程序后重试！
            </>
          ) : (
            <>
              <CheckCircle size={18} />
              删除完成，共删除 {deletedCount} 个空文件夹
            </>
          )}
        </div>
      )}

      <div className="file-list">
        {dirGroups.map(([dirPath, paths]) => {
          const visible = getDirVisible(dirPath, paths.length);
          return (
            <div key={dirPath} className="dir-group">
              <div className="dir-header" onClick={() => toggleDir(dirPath)}>
                {expandedDirs.has(dirPath) ? <ChevronDown size={20} /> : <ChevronRight size={20} />}
                <Folder size={16} className="dir-icon" />
                <span className="dir-name">{dirName(dirPath)}</span>
                <span className="dir-count">{paths.length} 个</span>
                <button className="btn btn-xs" onClick={(e) => { e.stopPropagation(); selectAllInDir(dirPath); }}>
                  {paths.every(p => selectedFolders.has(p)) ? '取消全选' : '全选此目录'}
                </button>
              </div>
              {expandedDirs.has(dirPath) && (
                <div className="dir-files">
                  {paths.slice(0, visible).map(path => (
                    <div
                      key={path}
                      className={`folder-item ${selectedFolders.has(path) ? 'selected' : ''}`}
                    >
                      <button className="checkbox-btn" onClick={() => toggleFolder(path)}>
                        {selectedFolders.has(path) ? <CheckSquare size={18} /> : <Square size={18} />}
                      </button>
                      <FolderOpen size={18} className="folder-icon" />
                      <div className="folder-info">
                        <div className="folder-name">{path.split(/[/\\]/).pop() || path}</div>
                        <div className="folder-path">{path}</div>
                      </div>
                      <button
                        className="btn btn-icon"
                        onClick={(e) => { e.stopPropagation(); openFileLocation(path); }}
                        title="打开文件夹位置"
                      >
                        <ExternalLink size={16} />
                      </button>
                    </div>
                  ))}
                  {visible < paths.length && (
                    <button className="load-more-btn"
                      onClick={(e) => { e.stopPropagation(); setVisiblePages(prev => ({ ...prev, [dirPath]: (prev[dirPath] || 1) + 1 })); }}>
                      显示更多（剩余 {paths.length - visible} 项）
                    </button>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
