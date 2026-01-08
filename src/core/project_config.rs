// Project Configuration
// Project-level settings that define what to sync and where

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Project-level configuration
/// This defines what files/directories to sync for a specific project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Workspace-level settings for different projects
    #[serde(default)]
    pub workspace_settings: WorkspaceSettings,
    
    /// Managed packages (shared resources)
    #[serde(default)]
    pub managed_packages: Vec<ManagedPackage>,
    
    /// Global settings that apply to all sync operations
    #[serde(default)]
    pub global_settings: GlobalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceSettings {
    /// Map of project name -> project settings
    #[serde(flatten)]
    pub projects: HashMap<String, ProjectSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    /// Map of package name -> package settings
    #[serde(flatten)]
    pub packages: HashMap<String, PackageSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageSettings {
    /// List of path mappings for this package
    pub mappings: Vec<Mapping>,
}

/// A path mapping between shared resources and project locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    /// Path in the shared resources (source)
    pub shared: String,
    
    /// Path in the project (destination)
    pub project: String,
    
    /// Patterns to exclude from syncing
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// A managed package definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedPackage {
    /// Package identifier
    pub name: String,
    
    /// Package type (e.g., "cursor-rules", "generic")
    #[serde(rename = "type")]
    pub package_type: Option<String>,
    
    /// Whether this package is active
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Human-readable description
    pub description: Option<String>,
    
    /// Location within shared resources
    pub location: String,
    
    /// Git remote name for this package
    pub git_remote: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    /// Sync direction: "both", "to_project", "to_shared"
    pub sync_direction: Option<String>,
    
    /// Conflict resolution strategy
    pub conflict_resolution: Option<String>,
    
    /// Auto-check interval in seconds (0 = disabled)
    pub auto_check_interval: Option<u64>,
    
    /// Show changelog on updates
    pub show_changelog: Option<bool>,
    
    /// Continue on individual file errors
    pub continue_on_error: Option<bool>,
    
    /// Auto-initialize git repository if not present
    pub auto_init_repo: Option<bool>,
}

fn default_true() -> bool { true }

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            workspace_settings: WorkspaceSettings::default(),
            managed_packages: Vec::new(),
            global_settings: GlobalSettings::default(),
        }
    }
}

impl ProjectConfig {
    /// Load project configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read project config: {}", path.display()))?;
        
        let config: ProjectConfig = serde_yaml::from_str(&content)
            .context("Failed to parse project config YAML")?;
        
        Ok(config)
    }
    
    /// Load project configuration from a workspace root
    pub fn load_from_workspace(workspace_root: &Path, config_name: &str) -> Result<Self> {
        let config_path = workspace_root.join(config_name);
        Self::load(&config_path)
    }
    
    /// Get all mappings for a specific project
    pub fn get_project_mappings(&self, project_name: &str) -> Vec<&Mapping> {
        let mut mappings = Vec::new();
        
        if let Some(project) = self.workspace_settings.projects.get(project_name) {
            for package_settings in project.packages.values() {
                mappings.extend(package_settings.mappings.iter());
            }
        }
        
        mappings
    }
    
    /// Get an enabled package by name
    pub fn get_package(&self, name: &str) -> Option<&ManagedPackage> {
        self.managed_packages
            .iter()
            .find(|p| p.name == name && p.enabled)
    }
    
    /// Get all enabled packages
    pub fn enabled_packages(&self) -> impl Iterator<Item = &ManagedPackage> {
        self.managed_packages.iter().filter(|p| p.enabled)
    }
    
    /// Resolve a shared path relative to the workspace
    pub fn resolve_shared_path(&self, workspace_root: &Path, shared_path: &str) -> PathBuf {
        if shared_path.starts_with("_shared-resources/") {
            workspace_root.join(shared_path)
        } else {
            workspace_root.join("_shared-resources").join(shared_path)
        }
    }
    
    /// Resolve a project path relative to the workspace
    pub fn resolve_project_path(&self, workspace_root: &Path, project_path: &str) -> PathBuf {
        workspace_root.join(project_path)
    }
    
    /// Save project configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize project config")?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write project config: {}", path.display()))?;
        
        Ok(())
    }
}
