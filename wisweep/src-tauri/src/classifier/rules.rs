use std::path::Path;

/// 检查路径是否为系统临时目录
pub fn is_temp_directory(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    #[cfg(windows)]
    {
        if let Ok(temp) = std::env::var("TEMP") {
            if path_str.starts_with(&temp.to_lowercase()) {
                return true;
            }
        }
        if let Ok(tmp) = std::env::var("TMP") {
            if path_str.starts_with(&tmp.to_lowercase()) {
                return true;
            }
        }
        // Windows 系统临时目录
        if path_str.contains("\\windows\\temp\\")
            || path_str.contains("\\windows\\prefetch\\")
            || path_str.contains("\\$recycle.bin\\")
        {
            return true;
        }
    }

    #[cfg(not(windows))]
    {
        if path_str.starts_with("/tmp/") || path_str.starts_with("/var/tmp/") {
            return true;
        }
    }

    false
}

/// 检查路径是否为浏览器缓存目录
pub fn is_browser_cache(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Chrome / Chromium
    if path_str.contains("google\\chrome\\user data\\default\\cache")
        || path_str.contains("google/chrome/user data/default/cache")
        || path_str.contains("google\\chrome\\user data\\default\\code cache")
        || path_str.contains("google/chrome/user data/default/code cache")
        || path_str.contains("google\\chrome\\user data\\default\\service worker\\cachestorage")
        || path_str.contains("chromium\\user data\\default\\cache")
        || path_str.contains("chromium/user data/default/cache")
        // Edge (Chromium)
        || path_str.contains("microsoft\\edge\\user data\\default\\cache")
        || path_str.contains("microsoft/edge/user data/default/cache")
        || path_str.contains("microsoft\\edge\\user data\\default\\code cache")
        || path_str.contains("microsoft/edge/user data/default/code cache")
    {
        return true;
    }

    // Firefox
    if path_str.contains("mozilla\\firefox\\profiles")
        || path_str.contains("mozilla/firefox/profiles")
    {
        if path_str.contains("cache2")
            || path_str.contains("offlinecache")
            || path_str.contains("thumbnails")
        {
            return true;
        }
    }

    // 通用浏览器缓存关键词
    if path_str.contains("\\browser\\cache\\") || path_str.contains("/browser/cache/") {
        return true;
    }

    false
}

/// 检查路径是否为构建产物/依赖缓存目录
pub fn is_build_cache(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    let build_keywords = [
        "node_modules",
        "\\target\\",  "/target/",
        "\\.venv\\",   "/.venv/",
        "\\venv\\",    "/venv/",
        "__pycache__",
        "\\.gradle\\", "/.gradle/",
        "\\pods\\",    "/pods/",
        "\\vendor\\bundle\\", "/vendor/bundle/",
        "\\.next\\",   "/.next/",
        "\\.nuxt\\",   "/.nuxt/",
        "\\jspm_packages\\", "/jspm_packages/",
        "\\bower_components\\", "/bower_components/",
    ];

    for keyword in &build_keywords {
        if path_str.contains(keyword) {
            return true;
        }
    }

    false
}

/// 检查路径是否为包管理器缓存
pub fn is_package_cache(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("\\npm\\_cacache\\") || path_str.contains("/npm/_cacache/") {
        return true;
    }
    if path_str.contains("\\pip\\cache\\") || path_str.contains("/pip/cache/") {
        return true;
    }
    if path_str.contains("\\.cargo\\registry\\cache\\")
        || path_str.contains("/.cargo/registry/cache/")
        || path_str.contains("\\cargo\\registry\\cache\\")
        || path_str.contains("/cargo/registry/cache/")
    {
        return true;
    }
    if path_str.contains("\\nuget\\http-cache\\")
        || path_str.contains("/nuget/http-cache/")
        || path_str.contains("\\nuget\\packages\\")
        || path_str.contains("/nuget/packages/")
    {
        return true;
    }
    if path_str.contains("\\yarn\\cache\\") || path_str.contains("/yarn/cache/") {
        return true;
    }
    if path_str.contains("\\.pnpm-store\\") || path_str.contains("/.pnpm-store/") {
        return true;
    }

    false
}

/// 检查路径是否为 IDE 缓存
pub fn is_ide_cache(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // VS Code
    if path_str.contains("\\.vscode\\") || path_str.contains("/.vscode/") {
        if path_str.contains("cachedata")
            || path_str.contains("cacheddatav2")
            || path_str.contains("gpucache")
        {
            return true;
        }
    }
    if path_str.contains("\\code\\cacheddata\\") || path_str.contains("/code/cacheddata/") {
        return true;
    }

    // JetBrains IDE
    if path_str.contains("\\jetbrains\\")
        || path_str.contains("/jetbrains/")
        || path_str.contains("\\.intellijidea\\")
        || path_str.contains("/.intellijidea/")
        || path_str.contains("\\pycharm\\")
        || path_str.contains("/pycharm/")
    {
        if path_str.contains("system\\caches")
            || path_str.contains("system/caches")
            || path_str.contains("system\\log")
            || path_str.contains("system/log")
        {
            return true;
        }
    }

    false
}

/// 检查路径是否为系统垃圾文件（Thumbs.db、Desktop.ini 等）
pub fn is_system_junk(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
            let lower = name_str.to_lowercase();
            if lower == "thumbs.db"
                || lower == "desktop.ini"
                || lower == ".ds_store"
                || lower == ".localized"
                || lower == "ehthumbs.db"
                || lower == "thumbs.db:encryptable"
                || lower.ends_with(".dmp")
                || lower.ends_with(".dump")
                || lower.ends_with(".hprof")
                || lower.starts_with("hs_err_pid")
            {
                return true;
            }
        }
    }
    false
}

/// 检查路径是否为下载残留
pub fn is_download_residue(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
            let lower = name_str.to_lowercase();
            if lower.ends_with(".aria2")
                || lower.ends_with(".td")
                || lower.ends_with(".crdownload")
                || lower.ends_with(".part")
                || lower.ends_with(".download")
                || lower.ends_with(".tmpdownload")
            {
                return true;
            }
        }
    }

    let path_str = path.to_string_lossy().to_lowercase();
    if (path_str.contains("\\downloads\\") || path_str.contains("/downloads/"))
        && !path_str.contains("\\downloads\\programs\\")
        && !path_str.contains("/downloads/programs/")
    {
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                let lower = name_str.to_lowercase();
                if (lower.starts_with("install") || lower.starts_with("setup"))
                    && (lower.ends_with(".exe") || lower.ends_with(".msi") || lower.ends_with(".dmg"))
                {
                    return true;
                }
            }
        }
    }

    false
}

/// 检查路径是否为版本控制目录
pub fn is_vcs_directory(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains(".git\\")
        || path_str.contains(".git/")
        || path_str.contains(".svn\\")
        || path_str.contains(".svn/")
        || path_str.contains(".hg\\")
        || path_str.contains(".hg/")
}

/// 获取文件的 MIME 类型（简单判断）
pub fn get_mime_type(extension: &str) -> Option<&'static str> {
    match extension.to_lowercase().as_str() {
        "txt" => Some("text/plain"),
        "html" | "htm" => Some("text/html"),
        "css" => Some("text/css"),
        "js" => Some("application/javascript"),
        "json" => Some("application/json"),
        "xml" => Some("application/xml"),
        "pdf" => Some("application/pdf"),
        "zip" => Some("application/zip"),
        "tar" | "gz" | "bz2" | "7z" | "rar" => Some("application/x-compressed"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "svg" => Some("image/svg+xml"),
        "mp3" | "wav" | "ogg" => Some("audio/*"),
        "mp4" | "avi" | "mkv" | "mov" => Some("video/*"),
        "doc" | "docx" => Some("application/msword"),
        "xls" | "xlsx" => Some("application/vnd.ms-excel"),
        "ppt" | "pptx" => Some("application/vnd.ms-powerpoint"),
        _ => None,
    }
}
