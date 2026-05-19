import { useAppStore } from '../../stores';
import {
  Scan,
  FileText,
  FolderOpen,
  History,
  Settings,
} from 'lucide-react';
import './Sidebar.css';

export function Sidebar() {
  const { activeTab, setActiveTab, scanResult } = useAppStore();
  
  const navItems = [
    { id: 'scan' as const, label: '开始扫描', icon: Scan },
    { id: 'result' as const, label: '扫描结果', icon: FileText, badge: scanResult?.candidates.length },
    { id: 'empty-folders' as const, label: '空文件夹', icon: FolderOpen, badge: scanResult?.empty_folders.length },
    { id: 'history' as const, label: '清理历史', icon: History },
    { id: 'settings' as const, label: '设置', icon: Settings },
  ];
  
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h1 className="app-title">智净大师</h1>
        <p className="app-subtitle">智能文件清理系统</p>
      </div>
      
      <nav className="sidebar-nav">
        {navItems.map(({ id, label, icon: Icon, badge }) => (
          <button
            key={id}
            className={`nav-item ${activeTab === id ? 'active' : ''}`}
            onClick={() => setActiveTab(id)}
          >
            <Icon size={20} />
            <span>{label}</span>
            {badge !== undefined && badge > 0 && (
              <span className="badge">{badge}</span>
            )}
          </button>
        ))}
      </nav>
      
      <div className="sidebar-footer">
        <p>版本 1.0.0</p>
      </div>
    </aside>
  );
}
