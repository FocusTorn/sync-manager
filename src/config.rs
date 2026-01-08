// Configuration loading module

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tui_components::TabBarConfigYaml;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub application: ApplicationConfig,
    #[serde(rename = "tab_bars")]
    pub tab_bars: std::collections::HashMap<String, TabBarConfigYaml>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationConfig {
    pub title: String,
    pub bindings: Vec<BindingConfigYaml>,
    pub status_bar: StatusBarConfigYaml,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BindingConfigYaml {
    pub key: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusBarConfigYaml {
    pub default_text: String,
    #[serde(default)]
    pub modal_text: Option<String>,
}

pub fn load_config(config_path: Option<PathBuf>) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let path = config_path.unwrap_or_else(|| {
        let mut default_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        default_path.push("src");
        default_path.push("config.yaml");
        default_path
    });
    
    let contents = fs::read_to_string(&path)?;
    let config: AppConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}
