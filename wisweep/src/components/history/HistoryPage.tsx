import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatSize, formatDate, formatDuration } from '../../utils/format';
import { History, FileText, Trash2, Scan, Trash, X } from 'lucide-react';
import './HistoryPage.css';

interface ScanHistoryItem {
  id: string;
  paths: string[];
  start_time: number;
  end_time: number | null;
  duration_ms: number;
  total_files: number;
  total_dirs: number;
  candidate_count: number;
  candidate_size: number;
  empty_folder_count: number;
  cleaned_count: number;
  cleaned_size: number;
  config: string | null;
}

export function HistoryPage() {
  const [history, setHistory] = useState<ScanHistoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    loadHistory();
  }, []);

  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => setToast(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [toast]);

  const loadHistory = async () => {
    try {
      const items = await invoke<ScanHistoryItem[]>('get_scan_history');
      setHistory(items);
    } catch (e) {
      console.error('Failed to load history:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke('delete_scan_history', { scanId: id });
      setHistory(prev => prev.filter(item => item.id !== id));
      setToast('已删除该条记录');
    } catch (e) {
      alert(`删除失败: ${e}`);
    }
  };

  const handleClearAll = async () => {
    if (!confirm('确定要清空所有历史记录吗？此操作不可撤销。')) return;
    try {
      await invoke('clear_scan_history');
      setHistory([]);
      setToast('已清空所有历史记录');
    } catch (e) {
      alert(`清空失败: ${e}`);
    }
  };

  if (loading) {
    return (
      <div className="history-page empty">
        <div className="empty-state">
          <p>加载中...</p>
        </div>
      </div>
    );
  }

  if (history.length === 0) {
    return (
      <div className="history-page empty">
        <div className="empty-state">
          <History size={64} />
          <h3>暂无扫描历史</h3>
          <p>开始扫描后，历史记录将显示在这里</p>
        </div>
      </div>
    );
  }

  return (
    <div className="history-page">
      <div className="page-header">
        <h2>清理历史</h2>
        <div className="header-actions">
          <span className="history-count">共 {history.length} 条记录</span>
          <button className="btn btn-sm btn-danger-text" onClick={handleClearAll}>
            <Trash size={16} />
            清空全部
          </button>
        </div>
      </div>

      {toast && (
        <div className="toast-info">
          <X size={14} />
          {toast}
        </div>
      )}

      <div className="history-list">
        {history.map((item) => (
          <div key={item.id} className="history-card">
            <div className="history-header">
              <div className="history-paths">
                {item.paths.map((p, i) => (
                  <span key={i} className="history-path-item">{p}</span>
                ))}
              </div>
              <div className="history-card-actions">
                <span className="history-date">{formatDate(item.start_time)}</span>
                <button
                  className="btn btn-icon btn-delete-icon"
                  onClick={() => handleDelete(item.id)}
                  title="删除此记录"
                >
                  <X size={14} />
                </button>
              </div>
            </div>

            <div className="history-stats">
              <div className="history-stat">
                <Scan size={16} />
                <span>{formatDuration(item.duration_ms)}</span>
              </div>
              <div className="history-stat">
                <FileText size={16} />
                <span>{item.total_files} 文件</span>
              </div>
              <div className="history-stat highlight">
                <Trash2 size={16} />
                <span>{item.candidate_count} 候选 · {formatSize(item.candidate_size)}</span>
              </div>
              {item.cleaned_count > 0 && (
                <div className="history-stat success">
                  <Trash2 size={16} />
                  <span>已清理 {item.cleaned_count} 项 · {formatSize(item.cleaned_size)}</span>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
