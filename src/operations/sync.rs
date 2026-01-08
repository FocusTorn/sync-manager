// Sync Engine
// Handles file synchronization operations

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::DiffEntry;

/// Options for sync operations
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Create backups before overwriting
    pub create_backup: bool,
    /// Continue on individual file errors
    pub continue_on_error: bool,
    /// Dry run - don't actually modify files
    pub dry_run: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            continue_on_error: true,
            dry_run: false,
        }
    }
}

/// Result of a sync operation
#[derive(Debug)]
pub struct SyncResult {
    /// Number of files successfully synced
    pub synced: usize,
    /// Number of files that failed
    pub failed: usize,
    /// Number of files skipped
    pub skipped: usize,
    /// Error messages for failed files
    pub errors: Vec<String>,
}

impl SyncResult {
    fn new() -> Self {
        Self {
            synced: 0,
            failed: 0,
            skipped: 0,
            errors: Vec::new(),
        }
    }
}

/// Engine for file synchronization operations
pub struct SyncEngine {
    options: SyncOptions,
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new(SyncOptions::default())
    }
}

impl SyncEngine {
    /// Create a new sync engine with the given options
    pub fn new(options: SyncOptions) -> Self {
        Self { options }
    }
    
    /// Sync a single file from source to destination
    pub fn sync_file(&self, diff: &DiffEntry) -> Result<()> {
        let source = &diff.source_path;
        let dest = &diff.destination_path;
        
        if self.options.dry_run {
            println!("Would sync: {} -> {}", source.display(), dest.display());
            return Ok(());
        }
        
        // Create backup if needed
        if self.options.create_backup && dest.exists() {
            self.create_backup(dest)?;
        }
        
        // Ensure destination directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Copy file
        fs::copy(source, dest)
            .with_context(|| format!("Failed to copy {} to {}", source.display(), dest.display()))?;
        
        // Preserve modification time
        if let Ok(metadata) = fs::metadata(source) {
            if let Ok(mtime) = metadata.modified() {
                let _ = filetime::set_file_mtime(dest, filetime::FileTime::from(mtime));
            }
        }
        
        Ok(())
    }
    
    /// Sync multiple files
    pub fn sync_files(&self, diffs: &[DiffEntry]) -> SyncResult {
        let mut result = SyncResult::new();
        
        for diff in diffs {
            match self.sync_file(diff) {
                Ok(()) => result.synced += 1,
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(format!("{}: {}", diff.path.display(), e));
                    
                    if !self.options.continue_on_error {
                        break;
                    }
                }
            }
        }
        
        result
    }
    
    /// Create a backup of a file
    fn create_backup(&self, path: &Path) -> Result<()> {
        let backup_path = path.with_extension(format!(
            "{}.backup",
            path.extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default()
        ));
        
        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;
        
        Ok(())
    }
    
    /// Delete a file (for removing files that only exist in destination)
    pub fn delete_file(&self, path: &Path) -> Result<()> {
        if self.options.dry_run {
            println!("Would delete: {}", path.display());
            return Ok(());
        }
        
        if self.options.create_backup {
            self.create_backup(path)?;
        }
        
        fs::remove_file(path)
            .with_context(|| format!("Failed to delete: {}", path.display()))?;
        
        Ok(())
    }
}

// Note: filetime crate would need to be added to Cargo.toml for full functionality
// For now, the modification time preservation is best-effort
mod filetime {
    use std::path::Path;
    use std::time::SystemTime;
    
    #[allow(dead_code)]
    pub struct FileTime(SystemTime);
    
    impl From<SystemTime> for FileTime {
        fn from(time: SystemTime) -> Self {
            Self(time)
        }
    }
    
    pub fn set_file_mtime(_path: &Path, _mtime: FileTime) -> std::io::Result<()> {
        // Placeholder - would use actual filetime crate in production
        // The FileTime struct is kept for the API even though the field isn't used yet
        let _ = _mtime; // Suppress unused parameter warning
        Ok(())
    }
}
