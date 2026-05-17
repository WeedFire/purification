import { useEffect } from 'react';
import { useAppStore } from '../../stores';
import { formatSize } from '../../utils/format';
import { Loader2, CheckCircle, XCircle } from 'lucide-react';
import './CleanupProgress.css';

interface CleanupProgressProps {
  onComplete: () => void;
  onClose: () => void;
}

export function CleanupProgress({ onComplete }: CleanupProgressProps) {
  const { cleanupProgress, cleanupResult } = useAppStore();

  useEffect(() => {
    if (cleanupResult) {
      const timer = setTimeout(onComplete, 3000);
      return () => clearTimeout(timer);
    }
  }, [cleanupResult, onComplete]);

  if (cleanupResult) {
    return (
      <div className="dialog-overlay">
        <div className="cleanup-progress-dialog done">
          <div className="progress-icon success">
            <CheckCircle size={48} />
          </div>
          <h2>清理完成</h2>
          <div className="result-stats">
            <div className="result-stat">
              <span className="stat-value">{cleanupResult.success_count}</span>
              <span className="stat-label">成功清理 (个)</span>
            </div>
            <div className="result-stat">
              <span className="stat-value">{formatSize(cleanupResult.released_size)}</span>
              <span className="stat-label">释放空间</span>
            </div>
            {cleanupResult.failed_count > 0 && (
              <div className="result-stat failed">
                <span className="stat-value">{cleanupResult.failed_count}</span>
                <span className="stat-label">失败</span>
              </div>
            )}
          </div>
          {cleanupResult.failed_items.length > 0 && (
            <div className="failed-list">
              <h4>失败项</h4>
              {cleanupResult.failed_items.map((item) => (
                <div key={item.path} className="failed-item">
                  <XCircle size={14} />
                  <span className="failed-path">
                    {item.path.split(/[/\\]/).pop()}
                  </span>
                  <span className="failed-reason">{item.reason}</span>
                </div>
              ))}
            </div>
          )}
          <div className="auto-close-hint">
            3 秒后自动关闭...
          </div>
        </div>
      </div>
    );
  }

  if (cleanupProgress && cleanupProgress.is_cleaning) {
    const percent = cleanupProgress.total_count > 0
      ? Math.round((cleanupProgress.processed_count / cleanupProgress.total_count) * 100)
      : 0;

    return (
      <div className="dialog-overlay">
        <div className="cleanup-progress-dialog">
          <div className="progress-icon spinning">
            <Loader2 size={48} />
          </div>
          <h2>正在清理...</h2>
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${percent}%` }} />
          </div>
          <div className="progress-text">
            {cleanupProgress.processed_count} / {cleanupProgress.total_count}
          </div>
          <div className="current-file">
            当前: {cleanupProgress.current_file.split(/[/\\]/).pop()}
          </div>
          <div className="released-info">
            已释放: {formatSize(cleanupProgress.released_size)}
          </div>
        </div>
      </div>
    );
  }

  return null;
}
