# 智净大师 (Wisweep) - 打包发布脚本
# ==============================================
# 用法: .\scripts\package.ps1
# 功能:
#   1. 清理旧的构建产物
#   2. 构建 Release 版本
#   3. 打包为安装程序 (MSI + NSIS)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  智净大师 - 发布打包" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $projectRoot

# 检查 git 状态
Write-Host "[1/5] 检查 Git 状态..." -ForegroundColor Yellow
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Host "  [警告] 工作区有未提交的更改:" -ForegroundColor Yellow
    $gitStatus | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    $confirm = Read-Host "  是否继续打包? (y/N)"
    if ($confirm -ne "y" -and $confirm -ne "Y") {
        Write-Host "  已取消" -ForegroundColor Red
        exit 0
    }
}
Write-Host "[完成]" -ForegroundColor Green

# 清理旧的构建产物
Write-Host "[2/5] 清理旧的构建产物..." -ForegroundColor Yellow
if (Test-Path "dist") {
    Remove-Item -Recurse -Force "dist"
    Write-Host "  - 已清理 dist/" -ForegroundColor Gray
}
$cargoTarget = Join-Path $projectRoot "src-tauri\target"
if (Test-Path $cargoTarget) {
    # 只清理 release 目录，保留 cache
    $releaseDir = Join-Path $cargoTarget "release"
    if (Test-Path $releaseDir) {
        Remove-Item -Recurse -Force $releaseDir
        Write-Host "  - 已清理 Rust release 产物" -ForegroundColor Gray
    }
}
Write-Host "[完成]" -ForegroundColor Green

# 安装依赖
Write-Host "[3/5] 安装依赖..." -ForegroundColor Yellow
pnpm install
if ($LASTEXITCODE -ne 0) { throw "依赖安装失败" }
Write-Host "[完成]" -ForegroundColor Green

# 构建前端
Write-Host "[4/5] 构建前端 + Rust..." -ForegroundColor Yellow
pnpm build
if ($LASTEXITCODE -ne 0) { throw "前端构建失败" }
Write-Host "[完成]" -ForegroundColor Green

# Tauri 打包（Release）
Write-Host "[5/5] 打包 Tauri 应用 (Release)..." -ForegroundColor Yellow
pnpm tauri build --release --bundles all
if ($LASTEXITCODE -ne 0) { throw "Tauri 打包失败" }

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  发布包已生成!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 显示打包产物
$bundleDir = Join-Path $projectRoot "src-tauri\target\release\bundle"
if (Test-Path $bundleDir) {
    Write-Host "安装包列表:" -ForegroundColor Yellow
    $files = Get-ChildItem $bundleDir -Recurse -File
    $totalSize = 0
    foreach ($f in $files) {
        $sizeMB = [math]::Round($f.Length / 1MB, 2)
        $totalSize += $f.Length
        Write-Host "  - $($f.Name)  ($sizeMB MB)" -ForegroundColor Gray
        Write-Host "    $($f.FullName)" -ForegroundColor DarkGray
    }
    $totalSizeMB = [math]::Round($totalSize / 1MB, 2)
    Write-Host ""
    Write-Host "总大小: $totalSizeMB MB" -ForegroundColor Green
}
