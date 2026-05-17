# 智净大师 — 架构设计文档

> 版本: 0.1.0 | 最后更新: 2026-05-16

---

## 1. 系统总览

智净大师是一款基于 **Tauri v2** 的跨平台桌面文件清理工具，核心设计理念是 **"透明可控"**——系统只负责识别推荐清理的文件，最终删除决策权完全交给用户。

```
┌─────────────────────────────────────────────────────────┐
│                    用户界面层 (UI Layer)                  │
│  React 19 + TypeScript + Vite 7                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ 扫描页面  │ │ 结果页面  │ │ 空文件夹  │ │ 设置页面  │  │
│  │ ScanPage  │ │ResultPage│ │   页面    │ │Settings  │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│       └────────────┴──────┬─────┴────────────┘         │
│                     │ Zustand Store │                   │
│              Tauri IPC (invoke / events)                │
├─────────────────────────────────────────────────────────┤
│                    业务逻辑层 (Service Layer)             │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────┐ │
│  │ 扫描任务调度器   │ │  结果聚合器     │ │ 清理执行器 │ │
│  │ Task Scheduler  │ │ Result Aggregator│ │ Cleaner   │ │
│  └────────┬────────┘ └────────┬────────┘ └─────┬─────┘ │
├───────────┼───────────────────┼─────────────────┼───────┤
│           ▼                   ▼                  ▼       │
│                核心引擎层 (Core Engine)                   │
│  ┌──────────┐ ┌────────────┐ ┌──────────┐ ┌──────────┐ │
│  │文件遍历器 │ │ 分类引擎   │ │空文件夹  │ │重复文件  │ │
│  │ jwalk    │ │ Rule-based │ │ 检测器   │ │ 检测器   │ │
│  └──────────┘ └────────────┘ └──────────┘ └──────────┘ │
├─────────────────────────────────────────────────────────┤
│                 基础设施层 (Infrastructure)               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ SQLite   │ │文件系统  │ │ 进程占用  │ │ 平台工具  │  │
│  │ 数据库   │ │抽象层    │ │ 检测     │ │ win/mac/linux│
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 2. 技术栈

| 层次 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 桌面框架 | **Tauri** | ^2.0 | 跨平台原生窗口、系统API调用 |
| 前端框架 | **React** | ^19.1 | UI 组件化 |
| 前端语言 | **TypeScript** | ~5.8 | 类型安全 |
| 构建工具 | **Vite** | ^7.0 | 前端打包开发服务器 |
| 状态管理 | **Zustand** | ^5.0 | 轻量全局状态 |
| 图标库 | **lucide-react** | ^1.16 | 界面图标 |
| 后端语言 | **Rust** | edition 2021 | 高性能、内存安全 |
| 文件遍历 | **jwalk** | ^0.8 | 并行目录遍历 |
| 序列化 | **serde** | ^1.0 | JSON序列化/反序列化 |
| 并发 | **tokio** + **crossbeam** | - | 异步运行时 + 通道 |
| 数据库 | **rusqlite** (SQLite) | ^0.32 | 本地持久化 |
| 包管理 | **pnpm** | - | 依赖管理 |

---

## 3. 数据流

### 3.1 扫描流程

```
用户输入路径
     │
     ▼
┌─────────────────────┐
│ start_scan command  │  [lib.rs]
│ 解析路径 → Vec<PathBuf>
└─────────┬───────────┘
          │ tokio::task::spawn_blocking
          ▼
┌─────────────────────┐
│ FileScanner::scan() │  [scanner.rs]
│ for each path:
│   walk_directory()  │
│    ├─ 目录 → jwalk  │
│    └─ 文件 → process_single_file()
│                      │
│ 产出 Vec<FileInfo>   │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ classifier::classify│  [engine.rs]
│ 逐文件匹配规则      │
│ 生成 categories     │
│ 未匹配 → FileCategory::Other
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ 过滤候选文件         │  [lib.rs]
│ filter: !categories.is_empty()
│     && !is_protected
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ EmptyFolderDetector │  [empty_folder_detector.rs]
│ 后序遍历检测空目录   │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ ScanResult → emit    │
│ "scan-progress" event│
│ 前端收到结果展示     │
└─────────────────────┘
```

### 3.2 清理流程

```
用户勾选 → 点击"清理选中"
     │
     ▼
┌─────────────────────┐
│ cleanup_files cmd   │  [lib.rs]
│ 验证文件是否存在    │
│ 创建 FileCleaner    │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ FileCleaner::cleanup│  [cleaner.rs]
│ 根据模式:           │
│  ├─ 回收站 (默认)   │
│  ├─ 永久删除        │
│  └─ 安全擦除        │
│ 进度 → 事件         │
│ 结果 → SQLite       │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ 前端展示清理报告     │
│ 成功/失败文件列表    │
└─────────────────────┘
```

---

## 4. 组件树

```
App
├── Sidebar                    [导航] 
│   ├── NavItem: 开始扫描
│   ├── NavItem: 扫描结果 (badge)
│   ├── NavItem: 空文件夹 (badge)
│   ├── NavItem: 清理历史
│   └── NavItem: 设置
├── ScanPage                   [路径选择 + 配置]
│   ├── PathInput (文本框 + 浏览按钮)
│   ├── FavoritePaths (收藏列表)
│   ├── ScanConfig (复选框配置面板)
│   ├── DiskInfo (磁盘空间概览)
│   └── ProgressBar (扫描进度)
├── ResultPage                 [扫描结果展示] *
│   ├── CategoryFilter (分类筛选按钮组)
│   ├── ActionBar (全选/清理按钮)
│   ├── CategoryGroup[]
│   │   ├── CategoryHeader (折叠)
│   │   └── FileRow[]
│   │       ├── Checkbox
│   │       ├── FileName + Path
│   │       ├── FileSize + Date
│   │       └── Categories + OpenLocation
├── CleanupPage (规划中)       [空文件夹展示]
├── HistoryPage (规划中)       [清理历史]
└── SettingsPage (规划中)      [规则管理 + 主题]
```

> `*` = 已实现，其余为占位/规划中

---

## 5. 数据库模型 (SQLite)

```sql
-- 扫描历史表
CREATE TABLE scan_history (
    id          TEXT PRIMARY KEY,           -- UUID
    paths       TEXT NOT NULL,              -- JSON 路径数组
    start_time  INTEGER NOT NULL,           -- 毫秒时间戳
    end_time    INTEGER,
    total_files INTEGER DEFAULT 0,
    total_dirs  INTEGER DEFAULT 0,
    candidates  INTEGER DEFAULT 0,
    releasable_size INTEGER DEFAULT 0,
    config_json TEXT                        -- 扫描配置快照
);

-- 路径收藏表
CREATE TABLE favorite_paths (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL UNIQUE,
    alias       TEXT,
    created_at  INTEGER NOT NULL
);

-- 自定义规则表
CREATE TABLE custom_rules (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    pattern_type    TEXT NOT NULL,           -- 'glob' | 'regex' | 'path_contains'
    patterns        TEXT NOT NULL,           -- JSON 数组
    category        TEXT NOT NULL,           -- FileCategory 枚举值
    base_weight     INTEGER DEFAULT 50,
    time_threshold_days INTEGER,
    time_weight_bonus    INTEGER DEFAULT 0,
    size_threshold       INTEGER,
    size_weight_bonus    INTEGER DEFAULT 0,
    is_protection   INTEGER DEFAULT 0,
    enabled         INTEGER DEFAULT 1,
    description     TEXT,
    created_at      INTEGER NOT NULL
);

-- 清理记录表
CREATE TABLE cleanup_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id     TEXT NOT NULL REFERENCES scan_history(id),
    cleanup_time INTEGER NOT NULL,
    file_path   TEXT NOT NULL,
    file_size   INTEGER,
    category    TEXT,
    delete_mode TEXT NOT NULL,              -- 'recycle_bin' | 'permanent' | 'secure_wipe'
    result      TEXT NOT NULL,              -- 'success' | 'failed'
    error_msg   TEXT
);
```

---

## 6. 分类引擎规则体系

规则定义在 `ClassificationRule` 结构体中，支持的匹配模式：

| 模式类型 | 说明 | 示例 |
|----------|------|------|
| `Glob` | 通配符匹配 | `*.tmp`, `*.log`, `~*.*` |
| `Regex` | 正则匹配 | 用户自定义规则 |
| `PathContains` | 路径包含关键词 | `node_modules`, `target` |

**内置规则**（共 6 类，17+ 条）：

| 规则组 | 匹配目标 | 基础权重 | 典型文件 |
|--------|----------|----------|----------|
| `TMP_001` | 临时文件 | 90 | `*.tmp`, `~*.*` |
| `LOG_001-002` | 日志文件 | 60-90 | `*.log`, `*.log.*` |
| `BUILD_001` | 构建产物 | 95 | `node_modules/`, `target/` |
| `BAK_001` | 旧备份 | 85 | `*.bak`, `*.old` |
| `DOWN_001` | 下载残留 | 95 | `*.aria2`, `*.crdownload` |
| `SYS_PROTECT` | 系统保护 | — | `C:\Windows\` (不可清理) |

未匹配任何规则的文件自动归入 **`Other`**（"其它"）类别。

---

## 7. 安全设计

```
┌──────────────────────────────────────────────────┐
│                  安全层叠模型                       │
│                                                    │
│  ① 默认不勾选 ← 所有复选框初始为空                  │
│  ② 二次确认   ← 弹窗显示汇总，需手动确认            │
│  ③ 系统路径保护 ← C:\Windows\ 等硬编码不可删        │
│  ④ 进程占用检测 ← 删除前确认文件未被使用            │
│  ⑤ 回收站模式  ← 默认可恢复，提供安全擦除选项       │
│  ⑥ 操作日志   ← 每次清理记录到 SQLite 可追溯       │
└──────────────────────────────────────────────────┘
```

---

## 8. 目录结构

```
wisweep/
├── package.json                  # 前端依赖 + 脚本
├── vite.config.ts                # Vite 配置
├── tsconfig.json                 # TypeScript 配置
├── index.html                    # HTML 入口
├── src/                          # 前端源码
│   ├── main.tsx                  # React 入口
│   ├── App.tsx                   # 路由 + 布局
│   ├── App.css                   # 全局样式 + CSS 变量
│   ├── components/
│   │   ├── common/               # 通用组件 (Sidebar)
│   │   ├── scan/                 # 扫描页面组件
│   │   ├── cleanup/              # 清理相关 (待实现)
│   │   └── settings/             # 设置页面 (待实现)
│   ├── stores/index.ts           # Zustand 全局状态
│   ├── types/index.ts            # TypeScript 类型定义
│   └── utils/format.ts           # 格式化工具
├── src-tauri/                    # Tauri 后端
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 应用配置
│   └── src/
│       ├── main.rs               # 程序入口
│       ├── lib.rs                # 命令注册 + 应用状态
│       ├── scanner/              # 文件扫描模块
│       │   ├── scanner.rs        # 核心扫描器
│       │   ├── empty_folder_detector.rs
│       │   └── progress.rs       # 进度追踪
│       ├── classifier/           # 分类引擎
│       │   ├── engine.rs         # 分类流水线
│       │   └── rules.rs          # 规则工具函数
│       ├── cleaner/              # 清理执行模块
│       │   ├── cleaner.rs        # 文件清理器
│       │   ├── file_lock.rs      # 进程占用检测
│       │   ├── recycle.rs        # 回收站操作
│       │   └── secure_wipe.rs    # 安全擦除
│       ├── database/             # SQLite 持久化
│       │   ├── database.rs       # 数据库层
│       │   ├── favorites.rs      # 收藏路径
│       │   └── history.rs        # 扫描历史
│       ├── models/               # 数据模型
│       │   ├── file_info.rs      # 文件信息
│       │   ├── scan_config.rs    # 扫描配置
│       │   ├── scan_result.rs    # 扫描结果
│       │   ├── cleanup_result.rs # 清理结果
│       │   └── rule.rs           # 分类规则
│       └── utils/                # 工具函数
│           ├── file_ops.rs       # 文件操作
│           └── platform.rs       # 跨平台适配
├── specs/
│   └── architecture.md           # 本文档
└── scripts/
    ├── dev.ps1                   # 开发启动
    ├── build.ps1                 # 构建
    ├── package.ps1               # 发布打包
    └── ci-build.ps1              # CI 构建
```
