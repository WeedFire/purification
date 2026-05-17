use std::path::Path;

/// 打开文件所在目录并选中文件
pub fn open_file_in_explorer(path: &Path) -> Result<(), anyhow::Error> {
    let path_str = path.to_string_lossy();

    #[cfg(windows)]
    {
        // Windows: 使用 explorer /select,
        let command = format!("/select,{}", path_str);
        std::process::Command::new("explorer")
            .arg(&command)
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 open -R
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试多种文件管理器
        if let Ok(_) = std::process::Command::new("nautilus")
            .arg("--select")
            .arg(path)
            .spawn()
        {
            return Ok(());
        }

        if let Ok(_) = std::process::Command::new("dolphin")
            .arg("--select")
            .arg(path)
            .spawn()
        {
            return Ok(());
        }

        // 后备方案：只打开目录
        if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open").arg(parent).spawn()?;
        }
    }

    Ok(())
}

/// 打开目录
pub fn open_directory(path: &Path) -> Result<(), anyhow::Error> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer").arg(path).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }

    Ok(())
}

/// 获取应用数据目录
pub fn get_app_data_dir() -> Result<std::path::PathBuf, anyhow::Error> {
    #[cfg(windows)]
    {
        let app_data = std::env::var("APPDATA")?;
        Ok(std::path::PathBuf::from(app_data).join("wisweep"))
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")?;
        Ok(std::path::PathBuf::from(home).join("Library/Application Support/wisweep"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")?;
        Ok(std::path::PathBuf::from(home).join(".local/share/wisweep"))
    }
}
