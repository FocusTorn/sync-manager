// Application State
// Main application state management and lifecycle

use anyhow::Result;
use std::path::PathBuf;

use super::{AppConfig, ProjectConfig};
use crate::operations::DiffEntry;

/// Project config file name
const PROJECT_CONFIG_NAME: &str = "sync-manager.yaml";

/// The current view mode in the application
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewMode {
    /// Viewing changes from shared -> project
    SharedToProject,
    /// Viewing changes from project -> shared  
    ProjectToShared,
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Application configuration (built-in defaults)
    pub config: AppConfig,
    
    /// Project configuration (loaded from sync-manager.yaml)
    pub project_config: Option<ProjectConfig>,
    
    /// Workspace root path
    pub workspace_root: PathBuf,
    
    /// Current view mode
    pub view_mode: ViewMode,
    
    /// Diffs for shared -> project direction
    pub shared_to_project_diffs: Vec<DiffEntry>,
    
    /// Diffs for project -> shared direction
    pub project_to_shared_diffs: Vec<DiffEntry>,
    
    /// Selected index in shared -> project list
    pub shared_to_project_index: usize,
    
    /// Selected index in project -> shared list
    pub project_to_shared_index: usize,
    
    /// Whether to show side-by-side diff view
    pub show_side_by_side: bool,
    
    /// Whether to fold unchanged regions in diff
    pub fold_unchanged: bool,
    
    /// Current scroll offset in diff view
    pub diff_scroll_offset: usize,
    
    /// Cached diff content for the current selection
    pub cached_diff_content: Option<String>,
    
    /// Path of currently cached diff
    pub cached_diff_path: Option<PathBuf>,
    
    /// Source lines for side-by-side view
    pub side_by_side_source: Option<Vec<String>>,
    
    /// Destination lines for side-by-side view
    pub side_by_side_dest: Option<Vec<String>>,
    
    /// Whether the application should quit
    pub should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let workspace_root = Self::detect_workspace_root()?;
        
        // Load project config from sync-manager.yaml
        let project_config = ProjectConfig::load_from_workspace(
            &workspace_root,
            PROJECT_CONFIG_NAME,
        ).ok();
        
        let mut app = Self {
            config: AppConfig::default(),
            project_config,
            workspace_root,
            view_mode: ViewMode::SharedToProject,
            shared_to_project_diffs: Vec::new(),
            project_to_shared_diffs: Vec::new(),
            shared_to_project_index: 0,
            project_to_shared_index: 0,
            show_side_by_side: false,
            fold_unchanged: true,
            diff_scroll_offset: 0,
            cached_diff_content: None,
            cached_diff_path: None,
            side_by_side_source: None,
            side_by_side_dest: None,
            should_quit: false,
        };
        
        // Load initial diffs if project config is available
        if app.project_config.is_some() {
            app.refresh_diffs()?;
        }
        
        Ok(app)
    }
    
    /// Detect the workspace root directory
    fn detect_workspace_root() -> Result<PathBuf> {
        // First try environment variable
        if let Ok(path) = std::env::var("WORKSPACE_ROOT") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }
        
        // Try to detect from current directory
        let cwd = std::env::current_dir()?;
        
        // Walk up looking for sync-manager.yaml
        let mut current = cwd.as_path();
        while let Some(parent) = current.parent() {
            // Look for sync-manager.yaml as the primary indicator
            if current.join(PROJECT_CONFIG_NAME).exists()
                || current.join("_shared-resources").exists()
            {
                return Ok(current.to_path_buf());
            }
            current = parent;
        }
        
        // Fall back to current directory
        Ok(cwd)
    }
    
    /// Get the currently selected diff entry
    pub fn selected_diff(&self) -> Option<&DiffEntry> {
        match self.view_mode {
            ViewMode::SharedToProject => {
                self.shared_to_project_diffs.get(self.shared_to_project_index)
            }
            ViewMode::ProjectToShared => {
                self.project_to_shared_diffs.get(self.project_to_shared_index)
            }
        }
    }
    
    /// Get the current diff list based on view mode
    pub fn current_diffs(&self) -> &[DiffEntry] {
        match self.view_mode {
            ViewMode::SharedToProject => &self.shared_to_project_diffs,
            ViewMode::ProjectToShared => &self.project_to_shared_diffs,
        }
    }
    
    /// Get the current selected index
    pub fn current_index(&self) -> usize {
        match self.view_mode {
            ViewMode::SharedToProject => self.shared_to_project_index,
            ViewMode::ProjectToShared => self.project_to_shared_index,
        }
    }
    
    /// Set the current selected index
    pub fn set_current_index(&mut self, index: usize) {
        match self.view_mode {
            ViewMode::SharedToProject => self.shared_to_project_index = index,
            ViewMode::ProjectToShared => self.project_to_shared_index = index,
        }
    }
    
    /// Move selection up
    pub fn select_previous(&mut self) {
        let index = self.current_index();
        if index > 0 {
            self.set_current_index(index - 1);
            self.clear_diff_cache();
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        let index = self.current_index();
        let max = self.current_diffs().len().saturating_sub(1);
        if index < max {
            self.set_current_index(index + 1);
            self.clear_diff_cache();
        }
    }
    
    /// Toggle between view modes
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::SharedToProject => ViewMode::ProjectToShared,
            ViewMode::ProjectToShared => ViewMode::SharedToProject,
        };
        self.clear_diff_cache();
    }
    
    /// Toggle side-by-side view
    pub fn toggle_side_by_side(&mut self) {
        self.show_side_by_side = !self.show_side_by_side;
        
        if self.show_side_by_side {
            // Load source and destination files - clone paths to avoid borrow issues
            let paths = self.selected_diff().map(|diff| {
                (diff.source_path.clone(), diff.destination_path.clone())
            });
            
            if let Some((source_path, dest_path)) = paths {
                self.side_by_side_source = std::fs::read_to_string(&source_path)
                    .ok()
                    .map(|s| s.lines().map(|l| l.to_string()).collect());
                self.side_by_side_dest = std::fs::read_to_string(&dest_path)
                    .ok()
                    .map(|s| s.lines().map(|l| l.to_string()).collect());
            }
        } else {
            self.side_by_side_source = None;
            self.side_by_side_dest = None;
        }
        
        self.diff_scroll_offset = 0;
    }
    
    /// Toggle folding of unchanged regions
    pub fn toggle_fold(&mut self) {
        if self.show_side_by_side {
            self.fold_unchanged = !self.fold_unchanged;
            self.diff_scroll_offset = 0;
        }
    }
    
    /// Clear the diff cache
    pub fn clear_diff_cache(&mut self) {
        self.cached_diff_content = None;
        self.cached_diff_path = None;
        self.show_side_by_side = false;
        self.side_by_side_source = None;
        self.side_by_side_dest = None;
        self.diff_scroll_offset = 0;
    }
    
    /// Scroll diff view up
    pub fn scroll_up(&mut self, amount: usize) {
        self.diff_scroll_offset = self.diff_scroll_offset.saturating_sub(amount);
    }
    
    /// Scroll diff view down
    pub fn scroll_down(&mut self, amount: usize) {
        self.diff_scroll_offset += amount;
    }
    
    /// Refresh diff lists
    pub fn refresh_diffs(&mut self) -> Result<()> {
        let project_config = match &self.project_config {
            Some(config) => config,
            None => return Ok(()), // No config, nothing to do
        };
        
        // Detect project name (directory name)
        let project_name = self.workspace_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sync-manager")
            .to_string();
        
        // Get mappings for this project
        let mappings = project_config.get_project_mappings(&project_name);
        
        if mappings.is_empty() {
            // No mappings found - clear diffs
            self.shared_to_project_diffs.clear();
            self.project_to_shared_diffs.clear();
            return Ok(());
        }
        
        // Collect all diffs from all mappings
        let mut shared_to_project_diffs = Vec::new();
        let mut project_to_shared_diffs = Vec::new();
        
        // Get shared resources base path
        let shared_resources_base = self.workspace_root.join("_shared-resources");
        
        // Create diff engine with global excludes
        let diff_engine = crate::operations::DiffEngine::new()
            .with_excludes(self.config.global_excludes.clone());
        
        // Get shared-cursor package (or first enabled package) for resolving relative paths
        let shared_package = project_config.get_package("shared-cursor")
            .or_else(|| project_config.enabled_packages().next());
        
        let shared_repo_path = if let Some(pkg) = shared_package {
            shared_resources_base.join(&pkg.location)
        } else {
            shared_resources_base // Fallback if no packages
        };
        
        for mapping in mappings {
            // Resolve shared path
            // If shared path starts with '_shared-resources', resolve from workspace root
            // Otherwise, resolve relative to shared_repo_path
            let shared_path = if mapping.shared.starts_with("_shared-resources/") {
                self.workspace_root.join(&mapping.shared)
            } else {
                shared_repo_path.join(&mapping.shared)
            };
            
            // Resolve project path (always relative to workspace root)
            let project_path = self.workspace_root.join(&mapping.project);
            
            // Get exclude patterns for this mapping
            let mapping_excludes: Vec<String> = mapping.exclude.clone();
            
            // Compute diffs in both directions
            let shared_to_proj = diff_engine.compute_diff(
                &shared_path,
                &project_path,
                crate::operations::DiffType::SharedToProject,
                &mapping_excludes,
            ).unwrap_or_default();
            
            let proj_to_shared = diff_engine.compute_diff(
                &project_path,
                &shared_path,
                crate::operations::DiffType::ProjectToShared,
                &mapping_excludes,
            ).unwrap_or_default();
            
            shared_to_project_diffs.extend(shared_to_proj);
            project_to_shared_diffs.extend(proj_to_shared);
        }
        
        // Update the diff lists
        self.shared_to_project_diffs = shared_to_project_diffs;
        self.project_to_shared_diffs = project_to_shared_diffs;
        
        // Reset indices if they're out of bounds
        if self.shared_to_project_index >= self.shared_to_project_diffs.len() {
            self.shared_to_project_index = 0;
        }
        if self.project_to_shared_index >= self.project_to_shared_diffs.len() {
            self.project_to_shared_index = 0;
        }
        
        // Clear cached diff since lists have changed
        self.clear_diff_cache();
        
        Ok(())
    }
    
    /// Request application quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
