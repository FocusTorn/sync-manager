// Build script - reads config.yaml at compile time and generates defaults
// This allows changing defaults during development without editing source code

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun if config.yaml changes
    println!("cargo:rerun-if-changed=src/config.yaml");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compiled_config.rs");
    
    // Try to read config.yaml from src/, fall back to hardcoded defaults if not found
    let config = if Path::new("src/config.yaml").exists() {
        let content = fs::read_to_string("src/config.yaml")
            .expect("Failed to read src/config.yaml");
        parse_config(&content)
    } else {
        // Fallback defaults if config.yaml doesn't exist
        CompiledConfig::default()
    };
    
    // Generate Rust code with the compiled-in values
    let generated = format!(
        r#"// Auto-generated from config.yaml at compile time
// Do not edit - modify config.yaml and rebuild instead

pub const SHOW_LINE_NUMBERS: bool = {show_line_numbers};
pub const SYNTAX_HIGHLIGHTING: bool = {syntax_highlighting};
pub const CONTEXT_LINES: usize = {context_lines};
pub const MOUSE_ENABLED: bool = {mouse_enabled};
pub const THEME: &str = "{theme}";

pub const SYNC_DIRECTION: &str = "{sync_direction}";
pub const CONFLICT_RESOLUTION: &str = "{conflict_resolution}";
pub const CONTINUE_ON_ERROR: bool = {continue_on_error};
pub const CREATE_BACKUPS: bool = {create_backups};

pub const GLOBAL_EXCLUDES: &[&str] = &[
{excludes}
];

// Side-by-side diff highlight colors (RGB tuples)
pub const SOURCE_DIM_BG: (u8, u8, u8) = {source_dim_bg};
pub const SOURCE_BRIGHT_BG: (u8, u8, u8) = {source_bright_bg};
pub const DEST_DIM_BG: (u8, u8, u8) = {dest_dim_bg};
pub const DEST_BRIGHT_BG: (u8, u8, u8) = {dest_bright_bg};
"#,
        show_line_numbers = config.show_line_numbers,
        syntax_highlighting = config.syntax_highlighting,
        context_lines = config.context_lines,
        mouse_enabled = config.mouse_enabled,
        theme = config.theme,
        sync_direction = config.sync_direction,
        conflict_resolution = config.conflict_resolution,
        continue_on_error = config.continue_on_error,
        create_backups = config.create_backups,
        excludes = config.global_excludes
            .iter()
            .map(|e| format!("    \"{}\",", e))
            .collect::<Vec<_>>()
            .join("\n"),
        source_dim_bg = format!("({}, {}, {})", config.source_dim_bg.0, config.source_dim_bg.1, config.source_dim_bg.2),
        source_bright_bg = format!("({}, {}, {})", config.source_bright_bg.0, config.source_bright_bg.1, config.source_bright_bg.2),
        dest_dim_bg = format!("({}, {}, {})", config.dest_dim_bg.0, config.dest_dim_bg.1, config.dest_dim_bg.2),
        dest_bright_bg = format!("({}, {}, {})", config.dest_bright_bg.0, config.dest_bright_bg.1, config.dest_bright_bg.2),
    );
    
    fs::write(&dest_path, generated).expect("Failed to write compiled config");
}

struct CompiledConfig {
    show_line_numbers: bool,
    syntax_highlighting: bool,
    context_lines: usize,
    mouse_enabled: bool,
    theme: String,
    sync_direction: String,
    conflict_resolution: String,
    continue_on_error: bool,
    create_backups: bool,
    global_excludes: Vec<String>,
    source_dim_bg: (u8, u8, u8),
    source_bright_bg: (u8, u8, u8),
    dest_dim_bg: (u8, u8, u8),
    dest_bright_bg: (u8, u8, u8),
}

impl Default for CompiledConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            syntax_highlighting: false,
            context_lines: 3,
            mouse_enabled: true,
            theme: "default".to_string(),
            sync_direction: "both".to_string(),
            conflict_resolution: "prompt".to_string(),
            continue_on_error: true,
            create_backups: true,
            global_excludes: vec![
                ".git".to_string(),
                "__pycache__".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.swp".to_string(),
                "*.swo".to_string(),
                "*~".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
            ],
            source_dim_bg: (55, 4, 4),      // #370404
            source_bright_bg: (95, 3, 3),   // #5f0303
            dest_dim_bg: (35, 41, 21),      // #232915
            dest_bright_bg: (80, 102, 31),  // #50661f
        }
    }
}

fn parse_config(content: &str) -> CompiledConfig {
    let mut config = CompiledConfig::default();
    
    // Simple YAML parsing (avoiding external dependencies in build script)
    let mut in_ui = false;
    let mut in_defaults = false;
    let mut _in_paths = false;
    let mut in_excludes = false;
    let mut in_colors = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Track which section we're in
        if trimmed.starts_with("ui:") {
            in_ui = true;
            in_defaults = false;
            _in_paths = false;
            in_excludes = false;
            in_colors = false;
            continue;
        } else if trimmed.starts_with("defaults:") {
            in_ui = false;
            in_defaults = true;
            _in_paths = false;
            in_excludes = false;
            in_colors = false;
            continue;
        } else if trimmed.starts_with("paths:") {
            in_ui = false;
            in_defaults = false;
            _in_paths = true;
            in_excludes = false;
            in_colors = false;
            continue;
        } else if trimmed.starts_with("colors:") {
            in_colors = true;
            continue;
        } else if trimmed.starts_with("global_excludes:") {
            in_excludes = true;
            config.global_excludes.clear(); // Start fresh when we see the section
            continue;
        }
        
        // Parse key-value pairs first (before checking if we should stop parsing colors)
        if let Some((key, value)) = parse_kv(trimmed) {
            if in_colors {
                // Check if this is a known color key
                match key {
                    "source_dim_bg" => {
                        config.source_dim_bg = parse_hex_color(value);
                        continue; // Continue to next line after parsing
                    }
                    "source_bright_bg" => {
                        config.source_bright_bg = parse_hex_color(value);
                        continue;
                    }
                    "dest_dim_bg" => {
                        config.dest_dim_bg = parse_hex_color(value);
                        continue;
                    }
                    "dest_bright_bg" => {
                        config.dest_bright_bg = parse_hex_color(value);
                        continue;
                    }
                    _ => {
                        // Unknown key in colors section - stop parsing colors
                        in_colors = false;
                    }
                }
            } else if in_ui {
                match key {
                    "show_line_numbers" => config.show_line_numbers = parse_bool(value),
                    "syntax_highlighting" => config.syntax_highlighting = parse_bool(value),
                    "context_lines" => config.context_lines = value.parse().unwrap_or(3),
                    "mouse_enabled" => config.mouse_enabled = parse_bool(value),
                    "theme" => config.theme = value.to_string(),
                    _ => {}
                }
            } else if in_defaults {
                match key {
                    "sync_direction" => config.sync_direction = value.to_string(),
                    "conflict_resolution" => config.conflict_resolution = value.to_string(),
                    "continue_on_error" => config.continue_on_error = parse_bool(value),
                    "create_backups" => config.create_backups = parse_bool(value),
                    _ => {}
                }
            }
        }
        
        // Stop parsing colors when we hit a non-color, non-empty, non-comment line
        // (only if we haven't already parsed it as a color key above)
        if in_colors && !trimmed.is_empty() && !trimmed.starts_with('#') {
            // Check if line is indented (color lines should be indented)
            // If not indented and not a known color key, we've left the colors section
            if !line.starts_with("    ") && !line.starts_with('\t') {
                in_colors = false;
            }
        }
        
        // Parse list items for global_excludes
        if in_excludes && trimmed.starts_with("- ") {
            let value = trimmed[2..].trim().trim_matches('"');
            config.global_excludes.push(value.to_string());
            continue;
        }
        
        // Stop parsing excludes when we hit a non-list line
        if in_excludes && !trimmed.starts_with("- ") && !trimmed.is_empty() && !trimmed.starts_with('#') {
            in_excludes = false;
        }
    }
    
    config
}

fn parse_kv(line: &str) -> Option<(&str, &str)> {
    // Skip comments and empty lines
    if line.starts_with('#') || line.is_empty() {
        return None;
    }
    
    // Find the colon separator
    let colon_pos = line.find(':')?;
    let key = line[..colon_pos].trim();
    let mut value = line[colon_pos + 1..].trim();
    
    // Remove inline comments (everything after # that's not part of a hex color)
    // But preserve # at start of value (hex color) or in quotes
    if let Some(comment_pos) = value.find(" #") {
        // Only remove if # is preceded by a space (comment, not hex color)
        value = &value[..comment_pos];
        value = value.trim();
    }
    
    // Skip if value is empty (section header)
    if value.is_empty() {
        return None;
    }
    
    Some((key, value))
}

fn parse_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "yes" | "1")
}

fn parse_hex_color(s: &str) -> (u8, u8, u8) {
    // Remove quotes if present
    let s = s.trim().trim_matches('"').trim_matches('\'');
    
    // Remove # if present
    let s = if s.starts_with('#') { &s[1..] } else { s };
    
    // Parse hex string
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return (r, g, b);
        }
    }
    
    // Fallback to default if parsing fails
    (0, 0, 0)
}
