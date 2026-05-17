import { useState } from 'react';
import { useAppStore } from '../../stores';
import { CleanupMode } from '../../types';
import { formatSize } from '../../utils/format';
import { Trash2, X, Shield, AlertTriangle, RotateCcw } from 'lucide-react';
import './CleanupDialog.css';

interface CleanupDialogProps {
  onClose: () => void;
}

export function CleanupDialog({ onClose }: CleanupDialogProps) {
  const {
    scanResult,
    selectedFiles,
    cleanupFiles,
    isCleaning,
  } = useAppStore();
  
  const [mode, setMode] = useState<CleanupMode>('recycle_bin');
  const [showWarning, setShowWarning] = useState(false);

  if (!scanResult) {
    onClose();
    return null;
  }

  const selectedCandidates = scanResult.candidates.filter(
    f => selectedFiles.has(f.path)
  );

  const totalSize = selectedCandidates.reduce((sum, f) => sum + f.size, 0);

  // 按分类统计
  const categoryStats = new Map<string, { count: number; size: number }>();
  for (const file of selectedCandidates) {
    const cat = file.categories[0] || 'other';
    const stat = categoryStats.get(cat) || { count: 0, size: 0 };
    stat.count += 1;
    stat.size += file.size;
    categoryStats.set(cat, stat);
  }

  const handleConfirm = async () => {
    if (mode === 'secure_wipe') {
      if (!showWarning) {
        setShowWarning(true);
        return;
      }
    }
    
    try {
      await cleanupFiles(
        selectedCandidates.map(f => f.path),
        mode
      );
      onClose();
    } catch (error) {
      alert(`清理失败: ${error}`);
    }
  };

  const modeIcons: Record<CleanupMode, React.ReactNode> = {
    recycle_bin: <RotateCcw size={18} />,
    permanent: <Trash2 size={18} />,
    secure_wipe: <Shield size={18} />,
  };

  const modeLabels: Record<CleanupMode, string> = {
    recycle_bin: '移至回收站（可恢复）',
    permanent: '永久删除（不可恢复）',
    secure_wipe: '安全擦除（覆写后删除）',
  };

  const modeDescriptions: Record<CleanupMode, string> = {
    recycle_bin: '文件移至系统回收站，可随时还原',
    permanent: '直接删除文件，但文件系统层面仍有恢复可能',
    secure_wipe: '用随机数据覆写文件后删除，无法恢复',
  };

  const categoryColorMap: Record<string, string> = {
    system_temp: '#FF6B6B',
    app_cache: '#4ECDC4',
    log: '#95E1D3',
    build_artifact: '#F38181',
    old_backup: '#AA96DA',
    large_file: '#FCBAD3',
    download_residue: '#A8D8EA',
    other: '#B2BEC3',
  };

  const categoryLabelMap: Record<string, string> = {
    system_temp: '系统临时文件',
    app_cache: '应用程序缓存',
    log: '日志文件',
    build_artifact: '构建产物',
    old_backup: '旧版备份文件',
    large_file: '大文件',
    download_residue: '下载残留',
    other: '其它',
  };

  if (isCleaning) {
    return null; // 由 CleanupProgress 处理
  }

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="cleanup-dialog" onClick={e => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>确认清理</h2>
          <p>请仔细核对即将删除的文件</p>
          <button className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="dialog-body">
          {/* 汇总统计 */}
          <div className="summary-section">
            <div className="summary-stat">
              <span className="stat-number">{selectedCandidates.length}</span>
              <span className="stat-label">个文件</span>
            </div>
            <div className="summary-stat primary">
              <span className="stat-number">{formatSize(totalSize)}</span>
              <span className="stat-label">可释放空间</span>
            </div>
          </div>

          {/* 分类汇总 */}
          <div className="category-summary">
            {Array.from(categoryStats.entries()).map(([cat, stat]) => (
              <div key={cat} className="category-row">
                <span
                  className="category-dot"
                  style={{ backgroundColor: categoryColorMap[cat] || '#B2BEC3' }}
                />
                <span className="category-label">
                  {categoryLabelMap[cat] || cat}
                </span>
                <span className="category-count">{stat.count} 个</span>
                <span className="category-size">{formatSize(stat.size)}</span>
              </div>
            ))}
          </div>

          {/* 删除模式选择 */}
          <div className="mode-section">
            <h3>删除模式</h3>
            <div className="mode-options">
              {(['recycle_bin', 'permanent', 'secure_wipe'] as CleanupMode[]).map((m) => (
                <button
                  key={m}
                  className={`mode-option ${mode === m ? 'active' : ''} ${m === 'secure_wipe' && showWarning ? 'warning' : ''}`}
                  onClick={() => { setMode(m); setShowWarning(false); }}
                >
                  {modeIcons[m]}
                  <div className="mode-text">
                    <span className="mode-label">{modeLabels[m]}</span>
                    <span className="mode-desc">{modeDescriptions[m]}</span>
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* 安全擦除警告 */}
          {mode === 'secure_wipe' && showWarning && (
            <div className="warning-banner">
              <AlertTriangle size={20} />
              <div>
                <strong>这是不可逆操作！</strong>
                <p>文件将被覆写 3 次后删除，即使使用数据恢复工具也无法恢复。</p>
              </div>
            </div>
          )}

          {/* 操作提示 */}
          <div className="action-hint">
            <AlertTriangle size={16} />
            <span>此操作默认不勾选，已由您人工确认勾选</span>
          </div>
        </div>

        <div className="dialog-footer">
          <button className="btn btn-secondary" onClick={onClose}>
            取消
          </button>
          <button
            className={`btn ${mode === 'secure_wipe' && !showWarning ? 'btn-warning' : 'btn-danger'}`}
            onClick={handleConfirm}
          >
            {mode === 'secure_wipe' && !showWarning ? (
              <>
                <Shield size={18} />
                我了解风险，继续
              </>
            ) : (
              <>
                <Trash2 size={18} />
                确认删除 ({selectedCandidates.length} 项)
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
