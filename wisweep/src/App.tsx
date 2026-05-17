import { useState, useCallback, useEffect } from 'react';
import { useAppStore } from './stores';
import { Sidebar } from './components/common';
import { ScanPage, ResultPage, CleanupDialog, CleanupProgress } from './components/scan';
import { EmptyFoldersPage } from './components/empty-folders';
import { HistoryPage } from './components/history';
import { SettingsPage } from './components/settings';
import './App.css';

function App() {
  const { 
    activeTab, 
    loadFavoritePaths, 
    loadRules,
    selectedFiles,
    isCleaning,
    cleanupResult,
    setCleanupResult,
  } = useAppStore();
  
  const [showCleanupDialog, setShowCleanupDialog] = useState(false);
  
  useEffect(() => {
    loadFavoritePaths();
    loadRules();
  }, []);
  
  const handleStartCleanup = useCallback(() => {
    if (selectedFiles.size === 0) return;
    setShowCleanupDialog(true);
  }, [selectedFiles]);
  
  const handleCleanupComplete = useCallback(() => {
    setShowCleanupDialog(false);
    // 从扫描结果中移除已清理的文件
    if (cleanupResult) {
      const cleanedPaths = cleanupResult.success_items.map(i => i.path);
      useAppStore.getState().removeFromScanResult(cleanedPaths);
    }
    setCleanupResult(null);
  }, [setCleanupResult, cleanupResult]);
  
  const handleCleanupClose = useCallback(() => {
    setShowCleanupDialog(false);
  }, []);
  
  const renderContent = () => {
    switch (activeTab) {
      case 'scan':
        return <ScanPage />;
      case 'result':
        return (
          <ResultPage 
            onStartCleanup={handleStartCleanup} 
          />
        );
      case 'empty-folders':
        return <EmptyFoldersPage />;
      case 'history':
        return <HistoryPage />;
      case 'settings':
        return <SettingsPage />;
      default:
        return <ScanPage />;
    }
  };
  
  return (
    <div className="app">
      <Sidebar />
      <main className="main-content">
        {renderContent()}
      </main>
      
      {showCleanupDialog && !isCleaning && !cleanupResult && (
        <CleanupDialog onClose={handleCleanupClose} />
      )}
      
      {/* 清理进度/结果 */}
      {(isCleaning || cleanupResult) && (
        <CleanupProgress 
          onComplete={handleCleanupComplete}
          onClose={handleCleanupClose}
        />
      )}
    </div>
  );
}

export default App;
