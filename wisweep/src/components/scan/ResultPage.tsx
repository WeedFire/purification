import { useState, useMemo, useCallback } from 'react';
import { useAppStore } from '../../stores';
import { CategoryLabels, CategoryColors, type FileCategory } from '../../types';
import { formatSize, formatDate } from '../../utils/format';
import { 
  FolderOpen, 
  ExternalLink, 
  Trash2, 
  CheckSquare, 
  Square,
  ChevronDown,
  ChevronRight,
  Folder,
  Search,
  X,
} from 'lucide-react';
import './ResultPage.css';

const PAGE_SIZE = 200;

interface ResultPageProps {
  onStartCleanup: () => void;
}

export function ResultPage({ onStartCleanup }: ResultPageProps) {
  const {
    scanResult,
    selectedFiles,
    toggleFileSelection,
    deselectAllFiles,
    selectAllInDirectory,
    openFileLocation,
    setActiveTab,
  } = useAppStore();

  // 搜索
  const [searchQuery, setSearchQuery] = useState('');
  
  // 展开的分类
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  // 每个分类已加载的页数
  const [loadedPages, setLoadedPages] = useState<Record<string, number>>({});
  
  // 按分类分组
  const groupedFiles = useMemo(() => {
    const result: Record<string, FileInfoBasic[]> = {};
    if (!scanResult) return result;
    for (const file of scanResult.candidates) {
      const category = file.categories[0] || 'other';
      if (!result[category]) result[category] = [];
      result[category].push(file);
    }
    return result;
  }, [scanResult]);

  // 当前选中的分类
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  // 当前可见文件列表（含搜索过滤）
  const currentFiles = useMemo(() => {
    if (!scanResult) return [];
    let files: FileInfoBasic[];
    if (selectedCategory === 'all') {
      files = scanResult.candidates;
    } else {
      files = groupedFiles[selectedCategory] || [];
    }
    if (searchQuery.trim()) {
      const q = searchQuery.trim().toLowerCase();
      files = files.filter(f =>
        f.path.toLowerCase().includes(q) ||
        (f.path.split(/[/\\]/).pop() || '').toLowerCase().includes(q)
      );
    }
    return files;
  }, [scanResult, selectedCategory, groupedFiles, searchQuery]);

  // 全选：搜索模式下只选中当前过滤结果
  const handleSelectAll = useCallback(() => {
    const store = useAppStore.getState();
    const sr = store.scanResult;
    if (!sr) return;
    const q = searchQuery.trim();
    const targetPaths = q
      ? currentFiles.map(f => f.path)
      : sr.candidates.map(f => f.path);
    const newSet = new Set(store.selectedFiles);
    for (const p of targetPaths) newSet.add(p);
    useAppStore.setState({ selectedFiles: newSet });
  }, [searchQuery, currentFiles]);

  // 全选当前分类：搜索模式下只选中过滤后的
  const handleSelectCategory = useCallback(() => {
    const store = useAppStore.getState();
    const targetPaths = currentFiles.map(f => f.path);
    const newSet = new Set(store.selectedFiles);
    for (const p of targetPaths) newSet.add(p);
    useAppStore.setState({ selectedFiles: newSet });
  }, [currentFiles]);

  const toggleCategoryExpand = useCallback((category: string) => {
    setExpandedCategories(prev => {
      const next = new Set(prev);
      if (next.has(category)) next.delete(category);
      else next.add(category);
      return next;
    });
  }, []);

  // 加载更多
  const loadMore = useCallback((category: string) => {
    setLoadedPages(prev => ({
      ...prev,
      [category]: (prev[category] || 1) + 1,
    }));
  }, []);

  // 获取某分类可见的文件切片
  const getVisibleFiles = useCallback((files: typeof currentFiles, category: string) => {
    const pages = loadedPages[category] || 1;
    return files.slice(0, pages * PAGE_SIZE);
  }, [loadedPages]);

  if (!scanResult) {
    return (
      <div className="result-page empty">
        <div className="empty-state">
          <FolderOpen size={64} />
          <h3>暂无扫描结果</h3>
          <p>请先选择路径进行扫描</p>
          <button className="btn btn-primary" onClick={() => setActiveTab('scan')}>
            开始扫描
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="result-page">
      <div className="page-header">
        <h2>扫描结果</h2>
        <div className="result-summary">
          <span>发现 <strong>{scanResult.candidates.length}</strong> 个可清理文件</span>
          <span>共 <strong>{formatSize(scanResult.total_releasable_size)}</strong></span>
        </div>
      </div>
      
      {/* 搜索框 */}
      <div className="search-bar">
        <Search size={16} className="search-icon" />
        <input
          type="text"
          className="search-input"
          placeholder="搜索文件名或路径..."
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
        />
        {searchQuery && (
          <button className="search-clear" onClick={() => setSearchQuery('')}>
            <X size={16} />
          </button>
        )}
        {searchQuery && (
          <span className="search-count">
            找到 {currentFiles.length} 个文件
          </span>
        )}
      </div>
      
      {/* 分类筛选 */}
      <div className="category-filter">
        <button
          className={`category-btn ${selectedCategory === 'all' ? 'active' : ''}`}
          onClick={() => setSelectedCategory('all')}
        >
          全部 ({scanResult.candidates.length})
        </button>
        {Object.entries(scanResult.category_stats).map(([cat, stats]) => (
          <button
            key={cat}
            className={`category-btn ${selectedCategory === cat ? 'active' : ''}`}
            onClick={() => { setSelectedCategory(cat); toggleCategoryExpand(cat); }}
            style={{ borderColor: CategoryColors[cat as FileCategory] }}
          >
            {CategoryLabels[cat as FileCategory]} ({stats.count})
          </button>
        ))}
      </div>
      
      {/* 操作栏 */}
      <div className="action-bar">
        <div className="selection-actions">
          <button className="btn btn-sm" onClick={handleSelectAll}>全选</button>
          <button className="btn btn-sm" onClick={deselectAllFiles}>取消全选</button>
          {selectedCategory !== 'all' && (
            <button className="btn btn-sm" onClick={handleSelectCategory}>
              全选当前分类
            </button>
          )}
          <span className="selected-count">
            已选择 {selectedFiles.size} 个文件
          </span>
        </div>
        
        <button 
          className="btn btn-danger"
          onClick={onStartCleanup}
          disabled={selectedFiles.size === 0}
        >
          <Trash2 size={18} />
          清理选中 ({formatSize(
            scanResult.candidates
              .filter((f: { path: string }) => selectedFiles.has(f.path))
              .reduce((sum: number, f: { size: number }) => sum + f.size, 0)
          )})
        </button>
      </div>
      
      {/* 文件列表 */}
      <div className="file-list">
        {searchQuery.trim() ? (
          // 搜索模式：扁平显示过滤后的结果（按目录分组），带分页
          <div className="search-results">
            <FilesByDirectory
              files={getVisibleFiles(currentFiles, '_search_')}
              selectedFiles={selectedFiles}
              onToggle={toggleFileSelection}
              onSelectDir={selectAllInDirectory}
              onOpenLocation={openFileLocation}
            />
            {currentFiles.length > getVisibleFiles(currentFiles, '_search_').length && (
              <button className="load-more-btn" onClick={() => loadMore('_search_')}>
                显示更多（剩余 {currentFiles.length - getVisibleFiles(currentFiles, '_search_').length} 项）
              </button>
            )}
          </div>
        ) : selectedCategory === 'all' ? (
          // 全部类别：按分类分组展示
          Object.entries(groupedFiles).map(([category, files]) => (
            <div key={category} className="category-group">
              <div 
                className="category-header"
                onClick={() => toggleCategoryExpand(category)}
              >
                {expandedCategories.has(category) ? (
                  <ChevronDown size={20} />
                ) : (
                  <ChevronRight size={20} />
                )}
                <span 
                  className="category-badge"
                  style={{ backgroundColor: CategoryColors[category as FileCategory] }}
                />
                <span className="category-name">{CategoryLabels[category as FileCategory]}</span>
                <span className="category-count">{files.length} 个文件</span>
                <span className="category-size">
                  {formatSize(files.reduce((sum: number, f: { size: number }) => sum + f.size, 0))}
                </span>
              </div>
              
              {expandedCategories.has(category) && (
                <div className="category-files">
                  <FilesByDirectory
                    files={getVisibleFiles(files, category)}
                    selectedFiles={selectedFiles}
                    onToggle={toggleFileSelection}
                    onSelectDir={selectAllInDirectory}
                    onOpenLocation={openFileLocation}
                  />
                  {files.length > getVisibleFiles(files, category).length && (
                    <button className="load-more-btn" onClick={() => loadMore(category)}>
                      显示更多（剩余 {files.length - getVisibleFiles(files, category).length} 项）
                    </button>
                  )}
                </div>
              )}
            </div>
          ))
        ) : selectedCategory && groupedFiles[selectedCategory] ? (
          // 单一分类：按目录分组展示
          <div className="single-category-files">
            <FilesByDirectory
              files={getVisibleFiles(groupedFiles[selectedCategory], selectedCategory)}
              selectedFiles={selectedFiles}
              onToggle={toggleFileSelection}
              onSelectDir={selectAllInDirectory}
              onOpenLocation={openFileLocation}
            />
            {groupedFiles[selectedCategory].length > getVisibleFiles(groupedFiles[selectedCategory], selectedCategory).length && (
              <button className="load-more-btn" onClick={() => loadMore(selectedCategory)}>
                显示更多（剩余 {groupedFiles[selectedCategory].length - getVisibleFiles(groupedFiles[selectedCategory], selectedCategory).length} 项）
              </button>
            )}
          </div>
        ) : null}
      </div>
    </div>
  );
}

// ─── 按目录分组的文件列表 ───

interface FileInfoBasic {
  path: string;
  size: number;
  modified_time: number | null;
  recommendation_reason: string | null;
  categories: FileCategory[];
}

function FilesByDirectory({ files, selectedFiles, onToggle, onSelectDir, onOpenLocation }: {
  files: FileInfoBasic[];
  selectedFiles: Set<string>;
  onToggle: (path: string) => void;
  onSelectDir: (dirPath: string) => void;
  onOpenLocation: (path: string) => void;
}) {
  // 按目录分组
  const dirGroups = useMemo(() => {
    const groups: Record<string, FileInfoBasic[]> = {};
    for (const file of files) {
      const sep = file.path.includes('\\') ? '\\' : '/';
      const lastSep = file.path.lastIndexOf(sep);
      const dir = lastSep >= 0 ? file.path.substring(0, lastSep + 1) : '/';
      if (!groups[dir]) groups[dir] = [];
      groups[dir].push(file);
    }
    return groups;
  }, [files]);

  const getDirName = (fullPath: string): string => {
    const cleaned = fullPath.replace(/[/\\]$/, '');
    const parts = cleaned.split(/[/\\]/);
    return parts[parts.length - 1] || cleaned || '(根目录)';
  };

  // 是否所有文件都已选中
  const isAllSelectedInDir = (dirFiles: FileInfoBasic[]): boolean =>
    dirFiles.every((f: FileInfoBasic) => selectedFiles.has(f.path));

  return (
    <>
      {Object.entries(dirGroups).map(([dirPath, dirFiles]) => (
        <div key={dirPath} className="dir-group">
          <div className="dir-header">
            <Folder size={16} className="dir-icon" />
            <span className="dir-name">{getDirName(dirPath)}</span>
            <span className="dir-count">{dirFiles.length} 个文件</span>
            <button
              className="btn btn-xs"
              onClick={() => onSelectDir(dirPath)}
            >
              {isAllSelectedInDir(dirFiles) ? '取消全选' : '全选此目录'}
            </button>
          </div>
          <div className="dir-files">
            {dirFiles.map((file: FileInfoBasic) => (
              <FileRow
                key={file.path}
                file={file}
                isSelected={selectedFiles.has(file.path)}
                onToggle={() => onToggle(file.path)}
                onOpenLocation={() => onOpenLocation(file.path)}
              />
            ))}
          </div>
        </div>
      ))}
    </>
  );
}

// ─── 文件行组件 ───

interface FileRowProps {
  file: FileInfoBasic;
  isSelected: boolean;
  onToggle: () => void;
  onOpenLocation: () => void;
}

function FileRow({ file, isSelected, onToggle, onOpenLocation }: FileRowProps) {
  const fileName = file.path.split(/[/\\]/).pop() || file.path;
  const dirPath = file.path.substring(0, file.path.length - fileName.length);
  
  return (
    <div className={`file-row ${isSelected ? 'selected' : ''}`}>
      <button className="checkbox-btn" onClick={onToggle}>
        {isSelected ? <CheckSquare size={18} /> : <Square size={18} />}
      </button>
      
      <div className="file-info">
        <div className="file-name">{fileName}</div>
        <div className="file-path">{dirPath}</div>
        {file.recommendation_reason && (
          <div className="file-reason">{file.recommendation_reason}</div>
        )}
      </div>
      
      <div className="file-meta">
        <span className="file-size">{formatSize(file.size)}</span>
        {file.modified_time && (
          <span className="file-time">{formatDate(file.modified_time)}</span>
        )}
      </div>
      
      <div className="file-categories">
        {file.categories.map((cat) => (
          <span 
            key={cat}
            className="category-tag"
            style={{ backgroundColor: CategoryColors[cat] }}
          >
            {CategoryLabels[cat]}
          </span>
        ))}
      </div>
      
      <button className="btn btn-icon" onClick={onOpenLocation} title="打开文件位置">
        <ExternalLink size={16} />
      </button>
    </div>
  );
}
