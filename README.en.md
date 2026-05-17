# Wisweep — Smart File Cleanup for Any Path

> Don't let the tool decide for you. See what you're deleting, choose what to keep.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Tauri](https://img.shields.io/badge/Tauri-v2-blueviolet)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![React](https://img.shields.io/badge/React-19-blue)

---

## Overview

Traditional PC cleaner tools are locked to drive C:. Your real disk space hogs—`node_modules`, design cache, old downloads—live on drives D:, E:, or external disks.

**Wisweep** lets you scan **any path** you choose. It identifies cleanable files using an intelligent rule-based engine, presents them in a clear categorized list, and **defaults everything unchecked**. You decide what to delete, one by one.

### Key Features

- **Scan any path** — Local disks, external drives, USB, any directory
- **Smart classification** — Temp files, caches, logs, build artifacts, etc.
- **Empty folder detection** — Recursive empty directory analysis with merge suggestions
- **Transparent confirmation** — All items unchecked by default, double-confirmation before cleanup
- **Multiple cleanup modes** — Recycle bin (recoverable), permanent delete, secure wipe
- **File location** — One-click open in file manager with file highlighted

---

## Quick Start

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | >= 18 | [nodejs.org](https://nodejs.org/) |
| pnpm | >= 8 | `npm install -g pnpm` |
| Rust | stable | [rustup.rs](https://rustup.rs/) |

### One-Click Start

```powershell
# Windows PowerShell
.\scripts\dev.ps1
```

### Manual Start

```bash
# 1. Install dependencies
pnpm install

# 2. Start Tauri development
pnpm dev:tauri
```

---

## Scripts

| Script | Purpose | Command |
|--------|---------|---------|
| `scripts/dev.ps1` | Start development | `.\scripts\dev.ps1` |
| `scripts/build.ps1` | Build project | `.\scripts\build.ps1 --release` |
| `scripts/package.ps1` | Package installers | `.\scripts\package.ps1` |
| `scripts/ci-build.ps1` | CI/CD build | `.\scripts\ci-build.ps1` |

Or use pnpm shortcuts:

```bash
pnpm scripts:dev          # Start dev
pnpm scripts:build        # Build
pnpm scripts:package      # Package for release
pnpm build:tauri          # Tauri build (Debug)
pnpm build:release        # Tauri build (Release)
```

---

## Build & Package

```bash
# Development build
pnpm build:tauri

# Release build (MSI + NSIS installers)
pnpm build:release

# Or use packaging script (cleans old artifacts first)
pnpm scripts:package
```

Output: `src-tauri/target/release/bundle/`

---

## Architecture

```
UI Layer (React 19 + TypeScript)
    │ Tauri IPC
    ▼
Service Layer ──→ Core Engine ──→ Infrastructure
(Rust)            (Rust)          (Rust + OS APIs)
```

See [specs/architecture.md](specs/architecture.md) for details (Chinese).

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Desktop | Tauri v2 | Cross-platform native window |
| Frontend | React 19 + TypeScript | UI components |
| State | Zustand 5 | Global state management |
| Bundler | Vite 7 | Dev server & build |
| Backend | Rust (edition 2021) | High-performance engine |
| File walk | jwalk 0.8 | Parallel directory traversal |
| Database | SQLite (rusqlite) | Local persistence |
| Icons | lucide-react | UI icon set |

---

## User Guide

1. **Select path** — Type/paste path(s), click Browse, or pick from favorites
2. **Configure** — Toggle recursive scan, hidden files, min file size
3. **Scan** — Progress shows real-time stats
4. **Review** — Results grouped by category, expand to see files
5. **Select** — Check items to clean (none checked by default)
6. **Confirm** — Review summary in confirmation dialog
7. **Clean** — Execute with chosen mode

### Cleanup Modes

| Mode | Description | Recoverable |
|------|-------------|-------------|
| Recycle Bin | Move to system trash (default) | ✅ |
| Permanent | Delete permanently | ❌ |
| Secure Wipe | Overwrite then delete | ❌ (irrecoverable) |

---

## License

MIT

---

## Related Docs

- [Architecture Design](specs/architecture.md) (Chinese)
- [Full Design Doc](../设计文档.md) (Chinese)
