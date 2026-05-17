# 智净大师 (Wisweep) - 开发环境启动脚本
# ==============================================
# 用法: .\scripts\dev.ps1
# 前置条件: Node.js 18+, Rust toolchain, pnpm

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  智净大师 - 开发环境启动" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 检查依赖
$hasPnpm = Get-Command pnpm -ErrorAction SilentlyContinue
$hasCargo = Get-Command cargo -ErrorAction SilentlyContinue

if (-not $hasPnpm) {
    Write-Host "[错误] 未找到 pnpm，请先安装: npm install -g pnpm" -ForegroundColor Red
    exit 1
}

if (-not $hasCargo) {
    Write-Host "[错误] 未找到 Rust/Cargo，请先安装: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

Write-Host "[1/3] 安装前端依赖..." -ForegroundColor Yellow
Set-Location (Join-Path $PSScriptRoot "..")
pnpm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "[错误] 依赖安装失败" -ForegroundColor Red
    exit 1
}
Write-Host "[成功] 依赖安装完成" -ForegroundColor Green

Write-Host ""
Write-Host "[2/3] 检查 Rust 工具链..." -ForegroundColor Yellow
$rustcVer = rustc --version
Write-Host "    Rust 版本: $rustcVer" -ForegroundColor Gray

Write-Host ""
Write-Host "[3/3] 启动 Tauri 开发服务器..." -ForegroundColor Yellow
Write-Host "    - 前端开发服务器: http://localhost:1420" -ForegroundColor Gray
Write-Host "    - Tauri 窗口将在编译完成后自动打开" -ForegroundColor Gray
Write-Host ""

pnpm tauri dev
if ($LASTEXITCODE -ne 0) {
    Write-Host "[错误] 开发服务器启动失败" -ForegroundColor Red
    exit 1
}
