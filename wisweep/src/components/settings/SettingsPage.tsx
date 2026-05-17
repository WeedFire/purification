import { useAppStore } from '../../stores';
import { FileText, Shield } from 'lucide-react';
import { AboutSection } from './AboutSection';
import './SettingsPage.css';

export function SettingsPage() {
  const { rules, cleanupMode, setCleanupMode } = useAppStore();

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
