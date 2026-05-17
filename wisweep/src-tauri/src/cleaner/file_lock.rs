use std::path::Path;

/// 检查文件是否被锁定（正在被使用）
pub fn is_file_locked(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use winapi::um::winnt::{FILE_SHARE_READ, GENERIC_READ};

        // 尝试以独占模式打开文件
        match std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(path)
        {
            Ok(_) => false,
            Err(e) => {
                // 如果是共享冲突错误，说明文件被锁定
                let raw_os_error = e.raw_os_error().unwrap_or(0);
                // ERROR_SHARING_VIOLATION = 32
                raw_os_error == 32
            }
        }
    }

    #[cfg(not(windows))]
    {
        // Unix 系统：尝试获取文件锁
        use std::fs::File;
        use std::os::unix::fs::FileExt;

        match File::open(path) {
            Ok(file) => {
                // 尝试获取排他锁
                match file.try_lock_exclusive() {
                    Ok(_) => {
                        // 成功获取锁，说明文件未被锁定
                        // 释放锁（文件关闭时自动释放）
                        false
                    }
                    Err(_) => {
                        // 无法获取锁，文件可能被锁定
                        true
                    }
                }
            }
            Err(_) => true,
        }
    }
}

#[cfg(windows)]
pub mod windows_restart_manager {
    use std::path::Path;
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::restartmanager::*;
    use winapi::um::winnt::WCHAR;

    /// 使用 Restart Manager API 检查文件是否被使用
    pub fn check_file_in_use(path: &Path) -> Result<bool, anyhow::Error> {
        unsafe {
            let mut session_handle: DWORD = 0;
            let mut session_key: [WCHAR; CCH_RM_SESSION_KEY + 1] = [0; CCH_RM_SESSION_KEY + 1];

            // 启动 Restart Manager 会话
            let result = RmStartSession(&mut session_handle, 0, session_key.as_mut_ptr());

            if result != 0 {
                return Err(anyhow::anyhow!("Failed to start Restart Manager session"));
            }

            // 注册文件
            let path_wide: Vec<WCHAR> = path
                .to_string_lossy()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let path_ptr = path_wide.as_ptr();
            let result = RmRegisterResources(
                session_handle,
                1,
                &path_ptr as *const *const u16 as *mut *const u16,
                0,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            );

            if result != 0 {
                RmEndSession(session_handle);
                return Err(anyhow::anyhow!("Failed to register resources"));
            }

            // 获取使用该文件的进程列表
            let mut process_info_needed: DWORD = 0;
            let mut process_info: [RM_PROCESS_INFO; 1] = std::mem::zeroed();
            let mut process_info_size: DWORD = 1;
            let mut reboot_reasons: DWORD = 0;

            let result = RmGetList(
                session_handle,
                &mut process_info_needed,
                &mut process_info_size,
                process_info.as_mut_ptr(),
                &mut reboot_reasons,
            );

            RmEndSession(session_handle);

            // 如果有进程在使用该文件
            Ok(process_info_needed > 0)
        }
    }
}
