use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

/// 安全擦除文件
pub fn secure_delete_file(path: &Path, passes: u32) -> Result<(), anyhow::Error> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist"));
    }

    let file_size = std::fs::metadata(path)?.len();

    // 打开文件进行覆写
    let mut file = File::options().write(true).open(path)?;

    // 多次覆写
    for pass in 0..passes {
        // 随机数据
        let pattern = match pass % 3 {
            0 => 0x00, // 全零
            1 => 0xFF, // 全一
            _ => {
                // 随机数据
                let mut rng = rand::thread_rng();
                rand::Rng::gen::<u8>(&mut rng)
            }
        };

        // 覆写整个文件
        file.seek(SeekFrom::Start(0))?;
        let buffer = vec![pattern; 1024 * 1024]; // 1MB buffer
        let mut remaining = file_size;

        while remaining > 0 {
            let to_write = std::cmp::min(remaining, buffer.len() as u64);
            file.write_all(&buffer[..to_write as usize])?;
            remaining -= to_write;
        }

        file.sync_data()?;
    }

    // 截断文件
    file.set_len(0)?;
    file.sync_data()?;

    // 重命名为随机名称
    let random_name = uuid::Uuid::new_v4().to_string();
    let temp_path = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?
        .join(random_name);

    drop(file);
    std::fs::rename(path, &temp_path)?;

    // 删除文件
    std::fs::remove_file(&temp_path)?;

    // 验证删除成功：路径不应再存在
    if temp_path.exists() {
        return Err(anyhow::anyhow!("删除失败：无法删除文件，可能没有权限"));
    }

    Ok(())
}

// 简单的随机数生成器（避免引入 rand crate）
mod rand {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct ThreadRng {
        state: u64,
    }

    impl ThreadRng {
        pub fn new() -> Self {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
            Self { state: seed }
        }

        pub fn gen_u8(&mut self) -> u8 {
            // 简单的 LCG 随机数生成器
            self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
            (self.state >> 24) as u8
        }
    }

    pub fn thread_rng() -> ThreadRng {
        ThreadRng::new()
    }

    pub trait Rng {
        fn gen<T>(&mut self) -> T;
    }

    impl Rng for ThreadRng {
        fn gen<T>(&mut self) -> T {
            // 这里只支持 u8
            unsafe { std::mem::transmute_copy(&self.gen_u8()) }
        }
    }
}
