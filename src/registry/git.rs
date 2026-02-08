use crate::error::{ForjaError, Result};
use std::path::Path;
use std::process::Command;

pub fn clone(url: &str, target: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(target)
        .output()
        .map_err(ForjaError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForjaError::Git(format!("git clone failed: {stderr}")));
    }

    Ok(())
}

pub fn pull(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_path)
        .args(["pull", "--ff-only"])
        .output()
        .map_err(ForjaError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForjaError::Git(format!("git pull failed: {stderr}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
