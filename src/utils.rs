use std::path::PathBuf;

use anyhow::Result;

pub fn get_tmp_dir_path() -> Result<PathBuf> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.push("tmp");
    Ok(path)
}

pub async fn create_tmp_dir() -> Result<()> {
    let tmp_path = get_tmp_dir_path()?;
    if !tmp_path.exists() {
        tokio::fs::create_dir(tmp_path).await?;
    }
    return Ok(());
}
