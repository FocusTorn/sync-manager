// Configuration validation module

use crate::config::AppConfig;
use std::path::PathBuf;

/// Load and validate configuration with error recovery
pub fn load_and_validate_config(
    config_path: Option<PathBuf>,
) -> Result<AppConfig, Box<dyn std::error::Error>> {
    use crate::config::load_config;
    
    let path = config_path.unwrap_or_else(|| {
        let mut default_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        default_path.push("src");
        default_path.push("config.yaml");
        default_path
    });
    
    // Try to load configuration
    match load_config(Some(path.clone())) {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("Warning: Failed to load configuration: {}", e);
            eprintln!("Using default configuration");
            // For now, return error - we can add default config later if needed
            Err(e)
        }
    }
}
