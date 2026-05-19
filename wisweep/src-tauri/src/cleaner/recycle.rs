use std::path::Path;

/// 将文件移至回收站
pub fn move_to_recycle_bin(path: &Path) -> Result<(), anyhow::Error> {
    #[cfg(windows)]
    {
        use std::ptr;
        use winapi::um::shellapi::{
            SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FO_DELETE, SHFILEOPSTRUCTW,
        };

        let path_wide: Vec<u16> = path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .chain(std::iter::once(0)) // 双 null 结尾
            .collect();

        let mut op = SHFILEOPSTRUCTW {
            hwnd: ptr::null_mut(),
            wFunc: FO_DELETE as u32,
            pFrom: path_wide.as_ptr(),
            pTo: ptr::null(),
            fFlags: FOF_ALLOWUNDO | FOF_NOCONFIRMATION,
            fAnyOperationsAborted: 0,
            hNameMappings: ptr::null_mut(),
            lpszProgressTitle: ptr::null(),
        };

        let result = unsafe { SHFileOperationW(&mut op) };

        if result != 0 {
            return Err(anyhow::anyhow!(
                "删除失败：错误码 {}",
                result
            ));
        }

        // 验证删除成功：路径不应再存在
        if path.exists() {
            return Err(anyhow::anyhow!("删除失败：无法移至回收站，可能没有权限"));
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 移动到 ~/.Trash
        let home = std::env::var("HOME")?;
        let trash_dir = Path::new(&home).join(".Trash");

        if !trash_dir.exists() {
            std::fs::create_dir_all(&trash_dir)?;
        }

        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

        let dest = trash_dir.join(file_name);

        // 如果目标已存在，添加时间戳
        let dest = if dest.exists() {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let new_name = format!("{}_{}", file_name.to_string_lossy(), timestamp);
            trash_dir.join(new_name)
        } else {
            dest
        };

        // 尝试移动到回收站，不使用 ? 以便统一通过 exists() 判断权限
        let _ = std::fs::rename(path, &dest);
        // 验证移动成功：原路径不应再存在
        if path.exists() {
            return Err(anyhow::anyhow!("删除失败：无法移至回收站，可能没有权限"));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 使用 FreeDesktop.org Trash 规范
        let home = std::env::var("HOME")?;
        let trash_dir = Path::new(&home).join(".local/share/Trash/files");
        let info_dir = Path::new(&home).join(".local/share/Trash/info");

        std::fs::create_dir_all(&trash_dir)?;
        std::fs::create_dir_all(&info_dir)?;

        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

        let dest = trash_dir.join(file_name);

        // 如果目标已存在，添加时间戳
        let dest = if dest.exists() {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let new_name = format!("{}_{}", file_name.to_string_lossy(), timestamp);
            trash_dir.join(new_name)
        } else {
            dest
        };

        // 创建 .trashinfo 文件
        let info_file = info_dir.join(format!(
            "{}.trashinfo",
            dest.file_name().unwrap().to_string_lossy()
        ));
        let info_content = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            path.to_string_lossy(),
            chrono::Utc::now().to_rfc3339()
        );
        std::fs::write(&info_file, info_content)?;

        // 尝试移动到回收站，不使用 ? 以便统一通过 exists() 判断权限
        let _ = std::fs::rename(path, &dest);
        // 验证移动成功：原路径不应再存在
        if path.exists() {
            return Err(anyhow::anyhow!("删除失败：无法移至回收站，可能没有权限"));
        }
        Ok(())
    }
}
