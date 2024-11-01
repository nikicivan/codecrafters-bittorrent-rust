use anyhow::Result;
use std::{fs::File, io::Read};

// 将读取文件为字节数组的函数定义在这里
pub fn read_file_to_bytes(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buff = Vec::new();
    file.read_to_end(&mut buff)?;

    Ok(buff)
}
