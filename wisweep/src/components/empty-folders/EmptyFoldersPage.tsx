import { useState, useEffect, useRef } from 'react';
import { useAppStore } from '../../stores';
import { invoke } from '@tauri-apps/api/core';
import { FolderOpen, CheckSquare, Square, Trash2, CheckCircle } from 'lucide-react';
import './EmptyFoldersPage.css';

export function EmptyFoldersPage() {
  const { scanResult, setActiveTab, removeEmptyFolders } = useAppStore();
  const [selectedFolders, setSelectedFolders] = useState<Set<string>>(new Set());
  const [deleting, setDeleting] = useState(false);
  const [done, setDone] = useState(false);
  const [deletedCount, setDeletedCount] = useState(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  if (!scanResult || scanResult.empty_folders.length === 0) {
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

  const emptyFolders = scanResult.empty_folders;

  const toggleFolder = (path: string) => {
    setSelectedFolders(prev => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const selectAll = () => {
    setSelectedFolders(new Set(emptyFolders.map(f => f.path)));
  };

  const deselectAll = () => {
    setSelectedFolders(new Set());
  };

  const handleDelete = async () => {
    if (selectedFolders.size === 0) return;
    const pathsToDelete = Array.from(selectedFolders);
    setDeleting(true);
    try {
      const failed = await invoke<string[]>('delete_empty_folders', {
        dirPaths: pathsToDelete,
      });
      // 从 scanResult 中移除已删除的
      removeEmptyFolders(pathsToDelete);
      // 显示完成提示
      setDeletedCount(pathsToDelete.length - failed.length);
      setDone(true);
      setSelectedFolders(new Set());
      // 3 秒后自动关闭
      timerRef.current = setTimeout(() => {
        setDone(false);
      }, 3000);
    } catch (e) {
      alert(`删除失败: ${e}`);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div className="empty-folders-page">
      <div className="page-header">
        <h2>空文件夹</h2>
        <span className="folder-count">
          发现 {emptyFolders.length} 个空文件夹
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

      {/* 完成提示 */}
      {done && (
        <div className="toast-success">
          <CheckCircle size={18} />
          删除完成，共删除 {deletedCount} 个空文件夹
        </div>
      )}

      <div className="folder-list">
        {emptyFolders.map((folder) => (
          <div
            key={folder.path}
            className={`folder-item ${selectedFolders.has(folder.path) ? 'selected' : ''}`}
            style={{ paddingLeft: `${folder.depth * 24 + 16}px` }}
          >
            <button className="checkbox-btn" onClick={() => toggleFolder(folder.path)}>
              {selectedFolders.has(folder.path) ? (
                <CheckSquare size={18} />
              ) : (
                <Square size={18} />
              )}
            </button>
            <FolderOpen size={18} className="folder-icon" />
            <div className="folder-info">
              <div className="folder-name">
                {folder.path.split(/[/\\]/).pop() || folder.path}
              </div>
              <div className="folder-path">{folder.path}</div>
            </div>
            {folder.can_merge && <span className="merge-badge">可合并</span>}
            {folder.empty_children_count > 0 && (
              <span className="children-count">{folder.empty_children_count} 个子文件夹</span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
