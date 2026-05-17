import { useState } from 'react';
import { Info, ExternalLink, Loader2 } from 'lucide-react';
import './AboutSection.css';

const APP_VERSION = '0.1.0';

export function AboutSection() {
  const [checking, setChecking] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<{ available: boolean; version?: string; url?: string } | null>(null);

  const checkUpdate = async () => {
    setChecking(true);
    setUpdateInfo(null);
    try {
      const resp = await fetch('https://api.github.com/repos/WeedFire/purification/releases/latest', {
        signal: AbortSignal.timeout(5000),
      });
      if (resp.ok) {
        const data = await resp.json();
        const latestVer = (data.tag_name || '').replace(/^v/, '');
        if (latestVer > APP_VERSION) {
          setUpdateInfo({ available: true, version: latestVer, url: data.html_url });
        } else {
          setUpdateInfo({ available: false });
        }
      } else {
        setUpdateInfo({ available: false });
      }
    } catch {
      setUpdateInfo({ available: false });
    } finally {
      setChecking(false);
    }
  };

  return (
    <section className="settings-section about-section">
      <h3>
        <Info size={20} />
        关于
      </h3>
      <div className="about-content">
        <div className="about-logo">智净大师</div>
        <div className="about-version">版本 {APP_VERSION}</div>
        <p className="about-desc">
          任意路径智能文件清理系统。不替用户做决定，把清理的决策权完整交还给用户。
        </p>

        <div className="about-update">
          <button
            className="btn btn-sm"
            onClick={checkUpdate}
            disabled={checking}
          >
            {checking ? (
              <><Loader2 size={14} className="spin" /> 检查中...</>
            ) : (
              '检查更新'
            )}
          </button>

          {updateInfo && (
            <div className={`update-result ${updateInfo.available ? 'available' : 'latest'}`}>
              {updateInfo.available ? (
                <span>
                  发现新版本 v{updateInfo.version}！
                  <a href={updateInfo.url} target="_blank" rel="noopener noreferrer">
                    前往下载 <ExternalLink size={12} />
                  </a>
                </span>
              ) : (
                <span>已是最新版本</span>
              )}
            </div>
          )}
        </div>

        <div className="about-links">
          <span>技术栈: Tauri v2 + React 19 + Rust</span>
        </div>
      </div>
    </section>
  );
}
