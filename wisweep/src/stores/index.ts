import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  ScanConfig,
  ScanProgress,
  ScanResult,
  CleanupMode,
  CleanupResult,
  CleanupProgress,
  FavoritePath,
  ClassificationRule,
  DiskSpaceInfo,
} from '../types';

interface AppState {
  // 扫描状态
  scanProgress: ScanProgress | null;
  scanResult: ScanResult | null;
  isScanning: boolean;
  isLoadingResult: boolean;
  
  // 清理状态
  cleanupProgress: CleanupProgress | null;
  cleanupResult: CleanupResult | null;
  isCleaning: boolean;
  
  // 配置
  scanConfig: ScanConfig;
  cleanupMode: CleanupMode;
  
  // 数据
  favoritePaths: FavoritePath[];
  rules: ClassificationRule[];
  
  // UI 状态
  activeTab: 'scan' | 'result' | 'empty-folders' | 'history' | 'settings';
  selectedFiles: Set<string>;
  
  // Actions
  setActiveTab: (tab: AppState['activeTab']) => void;
  setScanConfig: (config: Partial<ScanConfig>) => void;
  setCleanupMode: (mode: CleanupMode) => void;
  toggleFileSelection: (path: string) => void;
  selectAllFiles: () => void;
  deselectAllFiles: () => void;
  selectAllInDirectory: (dirPath: string) => void;
  selectFilesInCategory: (category: string) => void;
  
  // 扫描操作
  startScan: (paths: string[]) => Promise<string>;
  pauseScan: () => Promise<void>;
  resumeScan: () => Promise<void>;
  cancelScan: () => Promise<void>;
  
  // 清理操作
  cleanupFiles: (filePaths: string[], mode: CleanupMode) => Promise<CleanupResult>;
  cancelCleanup: () => Promise<void>;
  setCleanupResult: (result: CleanupResult | null) => void;
  removeFromScanResult: (paths: string[]) => void;
  removeEmptyFolders: (paths: string[]) => void;
  
  // 文件操作
  openFileLocation: (path: string) => Promise<void>;
  openFolder: (path: string) => Promise<void>;
  
  // 收藏路径
  loadFavoritePaths: () => Promise<void>;
  addFavoritePath: (path: string, alias?: string) => Promise<void>;
  removeFavoritePath: (path: string) => Promise<void>;
  
  // 规则
  loadRules: () => Promise<void>;
  
  // 磁盘空间
  getDiskSpace: (path: string) => Promise<DiskSpaceInfo>;
}

const defaultScanConfig: ScanConfig = {
  paths: [],
  recursive: true,
  max_depth: null,
  include_hidden: true,
  include_system: false,
  min_file_size: 1024,
  exclude_patterns: ['*.git/*', '*.svn/*', '*.hg/*'],
  large_file_threshold: 100 * 1024 * 1024,
  temp_file_age_days: 7,
  scan_empty_folders: true,
  detect_duplicates: false,
};

export const useAppStore = create<AppState>((set, get) => ({
  // 初始状态
  scanProgress: null,
  scanResult: null,
  isScanning: false,
  isLoadingResult: false,
  cleanupProgress: null,
  cleanupResult: null,
  isCleaning: false,
  scanConfig: defaultScanConfig,
  cleanupMode: 'recycle_bin',
  favoritePaths: [],
  rules: [],
  activeTab: 'scan',
  selectedFiles: new Set(),
  
  // Actions
  setActiveTab: (tab) => set({ activeTab: tab }),
  
  setScanConfig: (config) => set((state) => ({
    scanConfig: { ...state.scanConfig, ...config },
  })),
  
  setCleanupMode: (mode) => set({ cleanupMode: mode }),
  
  toggleFileSelection: (path) => set((state) => {
    const newSet = new Set(state.selectedFiles);
    if (newSet.has(path)) {
      newSet.delete(path);
    } else {
      newSet.add(path);
    }
    return { selectedFiles: newSet };
  }),
  
  selectAllFiles: () => set((state) => {
    const allPaths = state.scanResult?.candidates.map(f => f.path) || [];
    return { selectedFiles: new Set(allPaths) };
  }),
  
  deselectAllFiles: () => set({ selectedFiles: new Set() }),
  
  selectAllInDirectory: (dirPath) => set((state) => {
    const paths = state.scanResult?.candidates
      .filter(f => f.path.startsWith(dirPath))
      .map(f => f.path) || [];
    const newSet = new Set(state.selectedFiles);
    // 如果目录内所有文件都已选中，则取消全选；否则全选
    const allSelected = paths.every(p => newSet.has(p));
    if (allSelected) {
      paths.forEach(p => newSet.delete(p));
    } else {
      paths.forEach(p => newSet.add(p));
    }
    return { selectedFiles: newSet };
  }),
  
  selectFilesInCategory: (category) => set((state) => {
    const paths = state.scanResult?.candidates
      .filter(f => f.categories[0] === category)
      .map(f => f.path) || [];
    const newSet = new Set(state.selectedFiles);
    const allSelected = paths.every(p => newSet.has(p));
    if (allSelected) {
      paths.forEach(p => newSet.delete(p));
    } else {
      paths.forEach(p => newSet.add(p));
    }
    return { selectedFiles: newSet };
  }),
  
  // 扫描操作
  startScan: async (paths) => {
    set({ isScanning: true, scanProgress: null, scanResult: null, selectedFiles: new Set(), cleanupResult: null });
    
    try {
      const scanId = await invoke<string>('start_scan', {
        paths,
        config: { ...get().scanConfig, paths },
      });
      return scanId;
    } catch (error) {
      set({ isScanning: false });
      throw error;
    }
  },
  
  pauseScan: async () => {
    await invoke('pause_scan');
    set((state) => ({
      scanProgress: state.scanProgress ? { ...state.scanProgress, is_paused: true } : null,
    }));
  },
  
  resumeScan: async () => {
    await invoke('resume_scan');
    set((state) => ({
      scanProgress: state.scanProgress ? { ...state.scanProgress, is_paused: false } : null,
    }));
  },
  
  cancelScan: async () => {
    await invoke('cancel_scan');
    set({ isScanning: false });
  },
  
  // 清理操作
  cleanupFiles: async (filePaths, mode) => {
    const scanId = get().scanResult?.scan_id;
    if (!scanId) throw new Error('没有扫描结果');
    
    set({ isCleaning: true, cleanupProgress: null, cleanupResult: null });
    
    try {
      const result = await invoke<CleanupResult>('cleanup_files', {
        filePaths,
        mode,
        scanId,
      });
      set({ cleanupResult: result, isCleaning: false });
      
      // 清理后刷新扫描结果
      const updatedResult = await invoke<ScanResult | null>('get_scan_result');
      if (updatedResult) {
        set({ scanResult: updatedResult });
      }
      
      return result;
    } catch (error) {
      set({ isCleaning: false });
      throw error;
    }
  },
  
  cancelCleanup: async () => {
    await invoke('cancel_cleanup');
    set({ isCleaning: false });
  },
  
  setCleanupResult: (result) => set({ cleanupResult: result }),
  
  removeFromScanResult: (paths) => set((state) => {
    if (!state.scanResult) return state;
    const pathSet = new Set(paths);
    const newCandidates = state.scanResult.candidates.filter(f => !pathSet.has(f.path));
    const newSelected = new Set(state.selectedFiles);
    for (const p of paths) newSelected.delete(p);
    return {
      scanResult: { ...state.scanResult, candidates: newCandidates },
      selectedFiles: newSelected,
    };
  }),
  
  removeEmptyFolders: (paths) => set((state) => {
    if (!state.scanResult) return state;
    const pathSet = new Set(paths);
    return {
      scanResult: {
        ...state.scanResult,
        empty_folders: state.scanResult.empty_folders.filter(f => !pathSet.has(f.path)),
      },
    };
  }),
  
  // 文件操作
  openFileLocation: async (path) => {
    await invoke('open_file_location', { path });
  },
  
  openFolder: async (path) => {
    await invoke('open_folder', { path });
  },
  
  // 收藏路径
  loadFavoritePaths: async () => {
    const paths = await invoke<FavoritePath[]>('get_favorite_paths');
    set({ favoritePaths: paths });
  },
  
  addFavoritePath: async (path, alias) => {
    await invoke('add_favorite_path', { path, alias });
    await get().loadFavoritePaths();
  },
  
  removeFavoritePath: async (path) => {
    await invoke('remove_favorite_path', { path });
    await get().loadFavoritePaths();
  },
  
  // 规则
  loadRules: async () => {
    const rules = await invoke<ClassificationRule[]>('get_rules');
    set({ rules });
  },
  
  // 磁盘空间
  getDiskSpace: async (path) => {
    return await invoke<DiskSpaceInfo>('get_disk_space', { path });
  },
}));

// 监听扫描进度事件
listen<ScanProgress>('scan-progress', (event) => {
  const progress = event.payload;
  useAppStore.setState({ scanProgress: progress });
  
  if (!progress.is_scanning && !progress.is_paused) {
    useAppStore.setState({ isScanning: false, isLoadingResult: true });
    // 扫描完成后获取结果
    invoke<ScanResult | null>('get_scan_result').then((result) => {
      useAppStore.setState({ 
        scanResult: result,
        isLoadingResult: false,
        // 自动跳转到结果页
        activeTab: 'result',
      });
    }).catch(() => {
      useAppStore.setState({ isLoadingResult: false });
    });
  }
});

// 监听清理进度事件
listen<CleanupProgress>('cleanup-progress', (event) => {
  useAppStore.setState({ cleanupProgress: event.payload });
});
