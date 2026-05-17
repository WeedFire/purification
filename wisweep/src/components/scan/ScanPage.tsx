import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '../../stores';
import { FolderOpen, Play, Pause, Square } from 'lucide-react';
import { formatSize } from '../../utils/format';
import './ScanPage.css';

export function ScanPage() {
  const {
    scanConfig,
    setScanConfig,
    scanProgress,
    isScanning,
    startScan,
    pauseScan,
    resumeScan,
    cancelScan,
    getDiskSpace,
    favoritePaths,
  } = useAppStore();
  
  const [pathInput, setPathInput] = useState('');
  const [diskInfo, setDiskInfo] = useState<{ path: string; info: Awaited<ReturnType<typeof getDiskSpace>> } | null>(null);
  
  const handleBrowse = async () => {
    const selected = await open({
      directory: true,
      multiple: true,
      title: '选择要扫描的文件夹',
    });
    
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      setPathInput(paths.join(';'));
      setScanConfig({ paths });
      
      // 获取第一个路径的磁盘信息
      if (paths.length > 0) {
        try {
          const info = await getDiskSpace(paths[0]);
          setDiskInfo({ path: paths[0], info });
        } catch (e) {
          console.error('Failed to get disk space:', e);
        }
      }
    }
  };
  
  const handleStartScan = async () => {
    const paths = pathInput.split(';').filter(p => p.trim());
    if (paths.length === 0) {
      alert('请选择要扫描的路径');
      return;
    }
    
    try {
      await startScan(paths);
    } catch (error) {
      alert(`扫描失败: ${error}`);
    }
  };
  
  const handlePauseResume = async () => {
    if (scanProgress?.is_paused) {
      await resumeScan();
    } else {
      await pauseScan();
    }
  };
  
  return (
    <div className="scan-page">
      <div className="page-header">
        <h2>开始扫描</h2>
        <p>选择要扫描的文件夹，智能识别可清理的文件</p>
      </div>
      
      {/* 路径选择 */}
      <div className="path-section">
        <div className="path-input-group">
          <input
            type="text"
            value={pathInput}
            onChange={(e) => setPathInput(e.target.value)}
            placeholder="输入或粘贴路径（多个路径用分号分隔）"
            className="path-input"
          />
          <button className="btn btn-secondary" onClick={handleBrowse}>
            <FolderOpen size={18} />
            浏览
          </button>
        </div>
        
        {/* 收藏路径 */}
        {favoritePaths.length > 0 && (
          <div className="favorite-paths">
            <span className="label">收藏路径:</span>
            {favoritePaths.map((fp) => (
              <button
                key={fp.id}
                className="favorite-path-btn"
                onClick={() => setPathInput(fp.path)}
              >
                {fp.alias || fp.path}
              </button>
            ))}
          </div>
        )}
      </div>
      
      {/* 扫描配置 */}
      <div className="config-section">
        <h3>扫描配置</h3>
        
        <div className="config-grid">
          <label className="config-item">
            <input
              type="checkbox"
              checked={scanConfig.recursive}
              onChange={(e) => setScanConfig({ recursive: e.target.checked })}
            />
            <span>递归扫描子目录</span>
          </label>
          
          <label className="config-item">
            <input
              type="checkbox"
              checked={scanConfig.include_hidden}
              onChange={(e) => setScanConfig({ include_hidden: e.target.checked })}
            />
            <span>包含隐藏文件</span>
          </label>
          
          <label className="config-item">
            <input
              type="checkbox"
              checked={scanConfig.scan_empty_folders}
              onChange={(e) => setScanConfig({ scan_empty_folders: e.target.checked })}
            />
            <span>扫描空文件夹</span>
          </label>
          
          <div className="config-item">
            <label>最小文件大小:</label>
            <input
              type="number"
              value={scanConfig.min_file_size}
              onChange={(e) => setScanConfig({ min_file_size: parseInt(e.target.value) || 1024 })}
              className="config-input"
            />
            <span>字节</span>
          </div>
        </div>
      </div>
      
      {/* 磁盘空间信息 */}
      {diskInfo && (
        <div className="disk-info-section">
          <h3>磁盘空间</h3>
          <div className="disk-info">
            <div className="disk-bar">
              <div
                className="disk-used"
                style={{ width: `${(diskInfo.info.used_space / diskInfo.info.total_space) * 100}%` }}
              />
            </div>
            <div className="disk-stats">
              <span>总容量: {formatSize(diskInfo.info.total_space)}</span>
              <span>已用: {formatSize(diskInfo.info.used_space)}</span>
              <span>可用: {formatSize(diskInfo.info.available_space)}</span>
            </div>
          </div>
        </div>
      )}
      
      {/* 扫描进度 */}
      {isScanning && scanProgress && (
        <div className="progress-section">
          <h3>扫描进度</h3>
          <div className="progress-info">
            <div className="progress-item">
              <span className="label">当前路径:</span>
              <span className="value">{scanProgress.current_path}</span>
            </div>
            <div className="progress-stats">
              <div className="stat">
                <span className="stat-value">{scanProgress.scanned_count.toLocaleString()}</span>
                <span className="stat-label">已扫描文件</span>
              </div>
              <div className="stat">
                <span className="stat-value">{scanProgress.candidate_count.toLocaleString()}</span>
                <span className="stat-label">候选文件</span>
              </div>
              <div className="stat">
                <span className="stat-value">{scanProgress.empty_folder_count.toLocaleString()}</span>
                <span className="stat-label">空文件夹</span>
              </div>
            </div>
          </div>
          
          <div className="progress-actions">
            <button
              className={`btn ${scanProgress.is_paused ? 'btn-primary' : 'btn-secondary'}`}
              onClick={handlePauseResume}
            >
              {scanProgress.is_paused ? <Play size={18} /> : <Pause size={18} />}
              {scanProgress.is_paused ? '继续' : '暂停'}
            </button>
            <button className="btn btn-danger" onClick={cancelScan}>
              <Square size={18} />
              取消
            </button>
          </div>
        </div>
      )}
      
      {/* 开始扫描按钮 */}
      {!isScanning && (
        <div className="action-section">
          <button
            className="btn btn-primary btn-large"
            onClick={handleStartScan}
            disabled={!pathInput.trim()}
          >
            <Play size={20} />
            开始扫描
          </button>
        </div>
      )}
    </div>
  );
}
