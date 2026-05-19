import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../../stores';
import { FileText, Shield, FolderPlus, Trash2, RotateCcw } from 'lucide-react';
import { AboutSection } from './AboutSection';
import './SettingsPage.css';

interface ProtectedPath {
  id: number;
  path: string;
  description: string | null;
  is_system: boolean;
  created_at: number;
}

export function SettingsPage() {
  const { rules, cleanupMode, setCleanupMode } = useAppStore();
  const [protectedPaths, setProtectedPaths] = useState<ProtectedPath[]>([]);
  const [newPath, setNewPath] = useState('');
  const [newDesc, setNewDesc] = useState('');
  const [loading, setLoading] = useState(false);

  // 加载保护路径
  const loadProtectedPaths = async () => {
    try {
      const paths = await invoke<ProtectedPath[]>('get_protected_paths');
      setProtectedPaths(paths);
    } catch (e) {
      console.error('加载保护路径失败:', e);
    }
  };

  useEffect(() => {
    loadProtectedPaths();
  }, []);

  // 添加保护路径
  const handleAddPath = async () => {
    if (!newPath.trim()) return;
    setLoading(true);
    try {
      await invoke('add_protected_path', {
        path: newPath.trim(),
        description: newDesc.trim() || null,
      });
      setNewPath('');
      setNewDesc('');
      await loadProtectedPaths();
    } catch (e) {
      console.error('添加保护路径失败:', e);
    } finally {
      setLoading(false);
    }
  };

  // 移除保护路径
  const handleRemovePath = async (id: number) => {
    setLoading(true);
    try {
      await invoke('remove_protected_path', { id });
      await loadProtectedPaths();
    } catch (e) {
      console.error('移除保护路径失败:', e);
    } finally {
      setLoading(false);
    }
  };

  // 重置为默认
  const handleReset = async () => {
    if (!confirm('确定要重置为默认系统保护路径吗？')) return;
    setLoading(true);
    try {
      await invoke('reset_protected_paths');
      await loadProtectedPaths();
    } catch (e) {
      console.error('重置失败:', e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="settings-page">
      <div className="page-header">
        <h2>设置</h2>
      </div>

      <div className="settings-content">
        {/* 清理模式 */}
        <section className="settings-section">
          <h3>
            <Shield size={20} />
            默认清理模式
          </h3>
          <div className="settings-options">
            {(['recycle_bin', 'permanent', 'secure_wipe'] as const).map((mode) => (
              <label key={mode} className="settings-radio">
                <input
                  type="radio"
                  name="cleanupMode"
                  checked={cleanupMode === mode}
                  onChange={() => setCleanupMode(mode)}
                />
                <div className="radio-content">
                  <span className="radio-label">
                    {mode === 'recycle_bin' ? '移至回收站（推荐）' :
                     mode === 'permanent' ? '永久删除' : '安全擦除'}
                  </span>
                  <span className="radio-desc">
                    {mode === 'recycle_bin' ? '文件移至系统回收站，可随时还原' :
                     mode === 'permanent' ? '直接删除文件，不可恢复' : '覆写文件后删除，无法恢复'}
                  </span>
                </div>
              </label>
            ))}
          </div>
        </section>

        {/* 系统保护路径 */}
        <section className="settings-section">
          <h3>
            <Shield size={20} />
            系统保护路径
          </h3>
          <div className="admin-tip">
            <Shield size={16} />
            <span>提示：删除系统文件或受保护目录时，需要 <strong>以管理员身份运行</strong> 本程序才能成功删除</span>
          </div>
          <p className="section-desc">保护路径下的文件不会被清理。用户可添加自定义路径或移除不需要的系统路径</p>
          
          {/* 添加新路径 */}
          <div className="protected-path-add">
            <input
              type="text"
              placeholder="输入路径（如 \Program Files\MyApp\）"
              value={newPath}
              onChange={(e) => setNewPath(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddPath()}
            />
            <input
              type="text"
              placeholder="描述（可选）"
              value={newDesc}
              onChange={(e) => setNewDesc(e.target.value)}
            />
            <button onClick={handleAddPath} disabled={loading || !newPath.trim()}>
              <FolderPlus size={16} />
              添加
            </button>
          </div>

          {/* 路径列表 */}
          <div className="protected-paths-list">
            {protectedPaths.map((p) => (
              <div key={p.id} className="protected-path-item">
                <div className="path-info">
                  <code>{p.path}</code>
                  {p.description && <span className="path-desc">{p.description}</span>}
                  {p.is_system && <span className="path-tag system">系统</span>}
                </div>
                {!p.is_system && (
                  <button
                    className="btn-icon btn-danger"
                    onClick={() => handleRemovePath(p.id)}
                    disabled={loading}
                    title="移除"
                  >
                    <Trash2 size={16} />
                  </button>
                )}
              </div>
            ))}
          </div>

          <button className="btn-secondary" onClick={handleReset} disabled={loading}>
            <RotateCcw size={16} />
            重置为默认
          </button>
        </section>

        {/* 分类规则 */}
        <section className="settings-section">
          <h3>
            <FileText size={20} />
            分类规则
          </h3>
          <p className="section-desc">系统内置规则，用于自动识别可清理的文件类型</p>
          <div className="rules-list">
            {rules.map((rule) => (
              <div key={rule.id} className="rule-card">
                <div className="rule-header">
                  <span className="rule-id">{rule.id}</span>
                  <span className="rule-name">{rule.name}</span>
                  {rule.is_protection ? (
                    <span className="rule-tag protect">保护</span>
                  ) : (
                    <span className="rule-tag weight">权重 {rule.base_weight}</span>
                  )}
                </div>
                {rule.description && (
                  <p className="rule-desc">{rule.description}</p>
                )}
                {rule.patterns.length > 0 && (
                  <div className="rule-patterns">
                    {rule.patterns.map((p, i) => (
                      <code key={i}>{p}</code>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </section>

        {/* 关于 */}
        <AboutSection />
      </div>
    </div>
  );
}
