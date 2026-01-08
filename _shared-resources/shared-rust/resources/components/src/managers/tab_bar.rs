// Tab Bar Manager
// Provides YAML configuration helpers and OOP-style tab bar manager wrapper

use serde::Deserialize;
use crate::core::{RectHandle, RectRegistry, TabBarConfigData, TabConfigData, AlignmentConfigData, TabBarStateColors, TabState, TabBarState};
use crate::elements::tab_bar::{TabBar, TabBarStyle};

// ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
// │                                    YAML Configuration Structures                               │
// └────────────────────────────────────────────────────────────────────────────────────────────────┘

/// Alignment configuration from YAML
#[derive(Debug, Clone, Deserialize)]
pub struct AlignmentConfigYaml {
    /// Vertical position: "top" or "bottom"
    pub vertical: String,
    /// Horizontal alignment: "left", "center", or "right"
    pub horizontal: String,
    /// Horizontal offset (optional)
    pub offset_x: Option<u16>,
    /// Vertical offset (optional)
    pub offset_y: Option<u16>,
}

/// Tab bar colors configuration from YAML (for state-based coloring)
#[derive(Debug, Clone, Deserialize)]
pub struct TabBarColorsYaml {
    /// Color for active state (e.g., "green")
    pub active: Option<String>,
    /// Color for negate state (e.g., "red")
    pub negate: Option<String>,
    /// Color for disabled state (None = use default)
    pub disabled: Option<String>,
}

/// Tab bar configuration from YAML
#[derive(Debug, Clone, Deserialize)]
pub struct TabBarConfigYaml {
    /// Handle name (HWND)
    pub hwnd: String,
    /// Anchor HWND name of the container to anchor to
    pub anchor: String,
    /// Alignment configuration
    pub alignment: AlignmentConfigYaml,
    /// Style string (e.g., "tab", "text", "boxed", "box_static", "text_static")
    pub style: String,
    /// Default color (defaults to "cyan" if not specified)
    #[serde(default = "default_tab_bar_color")]
    pub color: String,
    /// Tab bar type (e.g., "state") - only valid for static styles (box_static, text_static)
    #[serde(rename = "type")]
    pub tab_bar_type: Option<String>,
    /// State-based colors - only valid when type: "state" and style is static
    pub colors: Option<TabBarColorsYaml>,
    /// Minimum tab width (optional, defaults to 8)
    pub min_tab_width: Option<u16>,
    /// Show tooltips (optional, defaults to true)
    pub tab_tooltips: Option<bool>,
    /// List of tabs
    pub tabs: Vec<TabConfigYaml>,
}

fn default_tab_bar_color() -> String {
    "cyan".to_string()
}

/// Tab configuration from YAML
#[derive(Debug, Clone, Deserialize)]
pub struct TabConfigYaml {
    /// Tab ID
    pub id: String,
    /// Tab display name
    pub name: String,
    /// Optional: "active" for non-static bars (sets active tab), or state ("active", "negate", "disabled") for static bars
    pub default: Option<String>,
}

// ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
// │                                    Configuration Conversion Functions                          │
// └────────────────────────────────────────────────────────────────────────────────────────────────┘

/// Convert YAML tab bar configuration to internal tab bar configuration data
pub fn convert_tab_bar_config(config: &TabBarConfigYaml) -> TabBarConfigData {
    let is_static = config.style == "box_static" || config.style == "text_static";
    
    // Validate and convert type and colors
    // type is only valid for static styles (box_static, text_static)
    // colors is only valid when type is "state"
    let (tab_bar_type, state_colors) = if is_static {
        // For static styles, type is allowed
        let tab_type = config.tab_bar_type.clone();
        
        // colors is only valid when type is "state"
        let colors = if tab_type.as_ref().map(|t| t == "state").unwrap_or(false) {
            config.colors.as_ref().map(|colors| TabBarStateColors {
                active: colors.active.clone(),
                negate: colors.negate.clone(),
                disabled: colors.disabled.clone(),
            })
        } else {
            // If type is not "state", ignore colors (or warn if provided)
            if config.colors.is_some() {
                eprintln!("Warning: 'colors' is only valid when 'type: state'. Ignoring colors for tab bar '{}'", config.hwnd);
            }
            None
        };
        
        (tab_type, colors)
    } else {
        // For non-static styles, type and colors are not used
        if config.tab_bar_type.is_some() {
            eprintln!("Warning: 'type' is only valid for static styles (box_static, text_static). Ignoring type for tab bar '{}'", config.hwnd);
        }
        if config.colors.is_some() {
            eprintln!("Warning: 'colors' is only valid for static styles with 'type: state'. Ignoring colors for tab bar '{}'", config.hwnd);
        }
        (None, None)
    };
    
    TabBarConfigData {
        hwnd: config.hwnd.clone(),
        anchor: config.anchor.clone(),
        style: config.style.clone(),
        color: config.color.clone(),
        tab_bar_type,
        state_colors,
        alignment: AlignmentConfigData {
            vertical: config.alignment.vertical.clone(),
            horizontal: config.alignment.horizontal.clone(),
            offset_x: config.alignment.offset_x.unwrap_or(0),
            offset_y: config.alignment.offset_y.unwrap_or(0),
        },
        min_tab_width: config.min_tab_width.unwrap_or(8),
        tab_tooltips: config.tab_tooltips.unwrap_or(true),
    }
}

/// Convert YAML tab configurations to internal tab configuration data
pub fn create_tab_configs(config: &TabBarConfigYaml) -> Vec<TabConfigData> {
    // Convert YAML config to registry config format
    // Check if this is a static tab bar (for state-based defaults)
    let is_static = config.style == "box_static" || config.style == "text_static";
    
    // Determine which tab should be active (for non-static bars):
    // - If any tab has default: "active", use that one
    // - Otherwise, default the first tab (index 0) to active
    let has_explicit_active = config.tabs.iter().any(|t| t.default.as_ref().map(|d| d == "active").unwrap_or(false));
    let default_active_index = if has_explicit_active {
        // Find the first tab with default: "active"
        config.tabs.iter().position(|t| t.default.as_ref().map(|d| d == "active").unwrap_or(false)).unwrap_or(0)
    } else {
        // No explicit active tab, default to first tab (for non-static bars)
        0
    };
    
    let tab_configs: Vec<TabConfigData> = config.tabs.iter().enumerate().map(|(idx, t)| {
        // Parse default state for static bars, or active flag for non-static bars
        let (active, state) = if is_static {
            // For static bars, parse default as state
            let tab_state = t.default.as_ref()
                .and_then(|d| match d.to_lowercase().as_str() {
                    "active" => Some(TabState::Active),
                    "negate" => Some(TabState::Negate),
                    "disabled" => Some(TabState::Disabled),
                    _ => Some(TabState::Default),
                })
                .unwrap_or(TabState::Default);
            (false, tab_state) // Static bars don't have active tabs
        } else {
            // For non-static bars, parse default: "active" as active flag
            let is_active = if has_explicit_active {
                t.default.as_ref().map(|d| d == "active").unwrap_or(false)
            } else {
                // Default first tab to active if none specified
                idx == default_active_index
            };
            (is_active, TabState::Default) // Non-static bars use active flag, not state
        };
        
        TabConfigData {
            id: t.id.clone(),
            name: t.name.clone(),
            active,
            state,
        }
    }).collect();
    
    tab_configs
}

/// Create and initialize a tab bar from YAML configuration
/// This is a convenience function that combines config conversion and tab bar initialization
pub fn create_tab_bar_from_config(
    registry: &mut RectRegistry,
    handle_name: &str,
    config: &TabBarConfigYaml,
) -> RectHandle {
    let tab_configs = create_tab_configs(config);
    let tab_bar_config = convert_tab_bar_config(config);
    TabBar::initialize_in_registry(registry, handle_name, &tab_bar_config, tab_configs)
}

// ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
// │                            Tab Bar Manager - OOP Style Tab Bar Operations                      │
// └────────────────────────────────────────────────────────────────────────────────────────────────┘

/// Tab Bar Manager wrapper for OOP-style tab bar operations
/// Associates all tab bar operations with a handle identifier
pub struct TabBarManager {
    handle: RectHandle,
    #[allow(dead_code)]
    handle_name: String,
}

impl TabBarManager {
    /// Create and initialize a new tab bar from config
    pub fn create(registry: &mut RectRegistry, handle_name: &str, config: &TabBarConfigYaml) -> Self {
        let handle = create_tab_bar_from_config(registry, handle_name, config);
        Self {
            handle,
            handle_name: handle_name.to_string(),
        }
    }
    
    /// Get the handle (object identifier)
    pub fn handle(&self) -> RectHandle {
        self.handle
    }
    
    /// Prepare tab bar for rendering (returns TabBar instance and state)
    /// For Tab style, this also adjusts the anchor box (y+1, height-1) before returning
    pub fn prepare(&self, registry: &mut RectRegistry, style_override: Option<TabBarStyle>) -> Option<(TabBar, RectHandle, TabBarState)> {
        TabBar::from_registry(registry, self.handle, style_override)
    }
    
    /// Navigate to the previous tab
    pub fn navigate_previous(&self, registry: &mut RectRegistry) -> bool {
        registry.navigate_tab(self.handle, -1)
    }
    
    /// Navigate to the next tab
    pub fn navigate_next(&self, registry: &mut RectRegistry) -> bool {
        registry.navigate_tab(self.handle, 1)
    }
    
    /// Set the active tab by index
    pub fn set_active(&self, registry: &mut RectRegistry, index: usize) -> bool {
        registry.set_active_tab(self.handle, index)
    }
    
    /// Set the state for a specific tab (for state-based coloring)
    pub fn set_tab_state(&self, registry: &mut RectRegistry, tab_index: usize, state: TabState) -> bool {
        registry.set_tab_state(self.handle, tab_index, state)
    }
}

