# 智净大师 (Wisweep) - 生产构建脚本
# ==============================================
# 用法: .\scripts\build.ps1 [--release]
# 参数:
#   --release    构建 Release 版本（默认 Debug）
#   --targets    bundles targets: msi,nsis,all (默认 all)

param(
    [switch]$Release = $false,
    [string]$Targets = "all"
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  智净大师 - 生产构建" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$buildType = if ($Release) { "release" } else { "debug" }

Write-Host "构建类型: $buildType" -ForegroundColor Gray
Write-Host "打包目标: $Targets" -ForegroundColor Gray
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

Set-Location $projectRoot

# 阶段 1: 安装依赖
Write-Host "[1/4] 安装前端依赖..." -ForegroundColor Yellow
pnpm install
if ($LASTEXITCODE -ne 0) { throw "依赖安装失败" }
Write-Host "[完成]" -ForegroundColor Green

# 阶段 2: 前端构建
Write-Host "[2/4] 构建前端代码..." -ForegroundColor Yellow
pnpm build
if ($LASTEXITCODE -ne 0) { throw "前端构建失败" }
Write-Host "[完成]" -ForegroundColor Green

# 阶段 3: Rust 代码检查
Write-Host "[3/4] 检查 Rust 代码..." -ForegroundColor Yellow
$cargoCmd = if ($Release) { "cargo build --release" } else { "cargo check" }
Invoke-Expression $cargoCmd
if ($LASTEXITCODE -ne 0) { throw "Rust 检查失败" }
Write-Host "[完成]" -ForegroundColor Green

# 阶段 4: Tauri 打包
Write-Host "[4/4] 打包 Tauri 应用..." -ForegroundColor Yellow
$tauriArgs = @("tauri", "build")
if ($Release) { $tauriArgs += "--release" }
if ($Targets -ne "all") { $tauriArgs += "--bundles", $Targets }

pnpm @tauriArgs
if ($LASTEXITCODE -ne 0) { throw "Tauri 打包失败" }

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  构建成功!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 显示输出文件
$targetDir = if ($Release) { "release" } else { "debug" }
$bundleDir = Join-Path $projectRoot "src-tauri\target\$targetDir\bundle"
if (Test-Path $bundleDir) {
    Write-Host "输出文件:" -ForegroundColor Yellow
    Get-ChildItem $bundleDir -Recurse -File | ForEach-Object {
        Write-Host "  $($_.FullName)" -ForegroundColor Gray
    }
}
