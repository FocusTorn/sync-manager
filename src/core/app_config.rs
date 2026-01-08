// Application Configuration
// Defaults compiled from config.yaml at build time
// Modify config.yaml and rebuild to change these values

// Include the auto-generated config from build.rs
pub mod compiled {
    include!(concat!(env!("OUT_DIR"), "/compiled_config.rs"));
}

/// Application-level configuration for sync-manager
/// Values are compiled in from config.yaml at build time
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// UI and display settings
    pub ui: UiSettings,
    
    /// Default behavior settings
    pub defaults: DefaultSettings,
    
    /// Global exclude patterns
    pub global_excludes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UiSettings {
    /// Show line numbers in diff views
    pub show_line_numbers: bool,
    
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    
    /// Number of context lines around changes
    pub context_lines: usize,
    
    /// Enable mouse support
    pub mouse_enabled: bool,
    
    /// UI theme
    pub theme: String,
}

#[derive(Debug, Clone)]
pub struct DefaultSettings {
    /// Default sync direction: "both", "to_project", "to_shared"
    pub sync_direction: String,
    
    /// Default conflict resolution: "prompt", "newer", "source", "destination"
    pub conflict_resolution: String,
    
    /// Continue syncing even if individual files fail
    pub continue_on_error: bool,
    
    /// Create backups before overwriting files
    pub create_backups: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_line_numbers: compiled::SHOW_LINE_NUMBERS,
            syntax_highlighting: compiled::SYNTAX_HIGHLIGHTING,
            context_lines: compiled::CONTEXT_LINES,
            mouse_enabled: compiled::MOUSE_ENABLED,
            theme: compiled::THEME.to_string(),
        }
    }
}

impl Default for DefaultSettings {
    fn default() -> Self {
        Self {
            sync_direction: compiled::SYNC_DIRECTION.to_string(),
            conflict_resolution: compiled::CONFLICT_RESOLUTION.to_string(),
            continue_on_error: compiled::CONTINUE_ON_ERROR,
            create_backups: compiled::CREATE_BACKUPS,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ui: UiSettings::default(),
            defaults: DefaultSettings::default(),
            global_excludes: compiled::GLOBAL_EXCLUDES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}
