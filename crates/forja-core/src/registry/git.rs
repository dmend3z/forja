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

pub fn head_sha(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_path)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(ForjaError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForjaError::Git(format!("git rev-parse HEAD failed: {stderr}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn clone_fails_with_invalid_url() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("repo");

        let result = clone("https://invalid.example.com/nonexistent.git", &target);
        assert!(result.is_err());
        match result.unwrap_err() {
            ForjaError::Git(msg) => assert!(msg.contains("git clone failed")),
            other => panic!("expected Git error, got: {:?}", other),
        }
    }

    #[test]
    fn pull_fails_on_non_repo_directory() {
        let dir = TempDir::new().unwrap();
        let result = pull(dir.path());
        assert!(result.is_err());
    }
}
