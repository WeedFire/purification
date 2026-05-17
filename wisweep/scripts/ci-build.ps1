# 智净大师 (Wisweep) - CI/CD 构建脚本
# ==============================================
# 用于 GitHub Actions / 其它 CI 环境
# 用法: .\scripts\ci-build.ps1

$ErrorActionPreference = "Stop"

Write-Host "========================================"
Write-Host "  CI Build: 智净大师"
Write-Host "========================================"

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $projectRoot

# 安装依赖
Write-Host ">>> [1/3] Installing dependencies..."
pnpm install --frozen-lockfile
if ($LASTEXITCODE -ne 0) { throw "pnpm install failed" }

# 代码检查
Write-Host ">>> [2/3] Running TypeScript check..."
pnpm tsc --noEmit
if ($LASTEXITCODE -ne 0) { throw "TypeScript check failed" }

# 构建
Write-Host ">>> [3/3] Building Tauri app..."
pnpm tauri build --release
if ($LASTEXITCODE -ne 0) { throw "Tauri build failed" }

Write-Host ">>> CI Build completed successfully!"
