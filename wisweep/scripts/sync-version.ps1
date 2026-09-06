# 智净大师 (Wisweep) - 版本号同步脚本
# ==============================================
# 单一版本源: package.json 的 "version" 字段
# 作用: 将 package.json 的版本号同步到 src-tauri/Cargo.toml
#       (tauri.conf.json 通过 "version": "../package.json" 自动引用, 无需处理)
#
# 用法: .\scripts\sync-version.ps1

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$packageJson = Join-Path $projectRoot "package.json"
$cargoToml = Join-Path $projectRoot "src-tauri\Cargo.toml"

# 1. 从 package.json 读取版本号
if (-not (Test-Path $packageJson)) {
    throw "未找到 package.json: $packageJson"
}
$pkg = Get-Content $packageJson -Raw | ConvertFrom-Json
$version = $pkg.version
if (-not $version) {
    throw "package.json 中缺少 version 字段"
}

# 2. 校验 semver 格式
if ($version -notmatch '^\d+\.\d+\.\d+') {
    throw "版本号格式无效: $version (应为 x.y.z)"
}

# 3. 同步到 Cargo.toml
if (-not (Test-Path $cargoToml)) {
    throw "未找到 Cargo.toml: $cargoToml"
}
$content = Get-Content $cargoToml -Raw
$updated = $content -replace '(?m)^version\s*=\s*"[^"]*"', "version = `"$version`""

if ($updated -eq $content) {
    Write-Host "[同步] Cargo.toml 未找到 version 字段，跳过" -ForegroundColor Yellow
} else {
    Set-Content -Path $cargoToml -Value $updated -NoNewline -Encoding UTF8
    Write-Host "[同步] 版本号已统一为 $version" -ForegroundColor Green
}

Write-Host "[同步] package.json -> $version" -ForegroundColor Green
