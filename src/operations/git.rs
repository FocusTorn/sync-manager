// Git Operations
// Git integration for repository status and operations

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// Git repository status information
#[derive(Debug, Clone)]
pub struct GitStatus {
    /// Whether the path is a git repository
    pub is_repo: bool,
    /// Whether the repository has a remote configured
    pub has_remote: bool,
    /// Remote URL if available
    pub remote_url: Option<String>,
    /// Current branch name
    pub branch: Option<String>,
    /// Number of commits ahead of remote
    pub commits_ahead: u32,
    /// Number of commits behind remote
    pub commits_behind: u32,
    /// Whether there are uncommitted changes
    pub has_uncommitted_changes: bool,
}

/// Git operations handler
pub struct GitOps;

impl GitOps {
    /// Check if a path is a git repository
    pub fn is_repo(path: &Path) -> bool {
        path.join(".git").exists()
    }
    
    /// Get full status of a git repository
    pub fn status(repo_path: &Path) -> Result<GitStatus> {
        let is_repo = Self::is_repo(repo_path);
        
        if !is_repo {
            return Ok(GitStatus {
                is_repo: false,
                has_remote: false,
                remote_url: None,
                branch: None,
                commits_ahead: 0,
                commits_behind: 0,
                has_uncommitted_changes: false,
            });
        }
        
        let (has_remote, remote_url) = Self::check_remote(repo_path)?;
        let branch = Self::current_branch(repo_path).ok();
        let (commits_ahead, commits_behind) = if has_remote {
            Self::commit_status(repo_path).unwrap_or((0, 0))
        } else {
            (0, 0)
        };
        let has_uncommitted_changes = Self::has_uncommitted_changes(repo_path)?;
        
        Ok(GitStatus {
            is_repo,
            has_remote,
            remote_url,
            branch,
            commits_ahead,
            commits_behind,
            has_uncommitted_changes,
        })
    }
    
    /// Check if repository has a remote and get its URL
    fn check_remote(repo_path: &Path) -> Result<(bool, Option<String>)> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(repo_path)
            .output()?;
        
        if output.status.success() {
            let url = String::from_utf8(output.stdout)?
                .trim()
                .to_string();
            Ok((true, Some(url)))
        } else {
            Ok((false, None))
        }
    }
    
    /// Get the current branch name
    fn current_branch(repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(repo_path)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?
                .trim()
                .to_string())
        } else {
            bail!("Failed to get current branch")
        }
    }
    
    /// Get commits ahead/behind remote
    fn commit_status(repo_path: &Path) -> Result<(u32, u32)> {
        let output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...origin/HEAD"])
            .current_dir(repo_path)
            .output()?;
        
        if output.status.success() {
            let text = String::from_utf8(output.stdout)?;
            let parts: Vec<&str> = text.trim().split_whitespace().collect();
            
            if parts.len() >= 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                Ok((ahead, behind))
            } else {
                Ok((0, 0))
            }
        } else {
            Ok((0, 0))
        }
    }
    
    /// Check if there are uncommitted changes
    fn has_uncommitted_changes(repo_path: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path)
            .output()?;
        
        if output.status.success() {
            let text = String::from_utf8(output.stdout)?;
            Ok(!text.trim().is_empty())
        } else {
            Ok(false)
        }
    }
    
    /// Fetch from remote
    pub fn fetch(repo_path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["fetch", "--all"])
            .current_dir(repo_path)
            .output()?;
        
        if !output.status.success() {
            bail!("Git fetch failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    /// Pull from remote
    pub fn pull(repo_path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["pull"])
            .current_dir(repo_path)
            .output()?;
        
        if !output.status.success() {
            bail!("Git pull failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    /// Push to remote
    pub fn push(repo_path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["push"])
            .current_dir(repo_path)
            .output()?;
        
        if !output.status.success() {
            bail!("Git push failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    /// Stage a file
    pub fn add(repo_path: &Path, file_path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["add", &file_path.to_string_lossy()])
            .current_dir(repo_path)
            .output()?;
        
        if !output.status.success() {
            bail!("Git add failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    /// Commit staged changes
    pub fn commit(repo_path: &Path, message: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(repo_path)
            .output()?;
        
        if !output.status.success() {
            bail!("Git commit failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
}
