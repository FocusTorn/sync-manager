// Rectangle Handle System (HWND-like)
// Provides window handle functionality for tracking rendered rectangles
//
// Usage:
//   let mut registry = RectRegistry::new();
//   let handle = registry.register("my-window", rect);
//   // Later...
//   if let Some(metrics) = registry.get_metrics(handle) {
//       println!("Window at: {},{} size: {}x{}", metrics.x, metrics.y, metrics.width, metrics.height);
//   }

use ratatui::layout::Rect;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Handle to a registered rectangle (similar to Windows HWND)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RectHandle(u64);

impl RectHandle {
    /// Get the internal ID of this handle
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Metrics for a registered rectangle
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectMetrics {
    /// Top-left X coordinate
    pub x: u16,
    /// Top-left Y coordinate
    pub y: u16,
    /// Width of the rectangle
    pub width: u16,
    /// Height of the rectangle
    pub height: u16,
}

impl From<Rect> for RectMetrics {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<RectMetrics> for Rect {
    fn from(metrics: RectMetrics) -> Self {
        Self {
            x: metrics.x,
            y: metrics.y,
            width: metrics.width,
            height: metrics.height,
        }
    }
}

/// Registry entry for a rectangle
#[derive(Debug, Clone)]
struct RegistryEntry {
    /// Optional name/identifier for the rectangle
    name: Option<String>,
    /// Current metrics (position and size)
    metrics: RectMetrics,
}

/// Tab bar state stored in registry
#[derive(Debug, Clone)]
pub struct TabBarState {
    /// Index of the currently active tab
    pub active_tab_index: usize,
    /// Total number of tabs
    pub tab_count: usize,
    /// Tab names (for rendering and navigation)
    pub tab_names: Vec<String>,
    /// Tab configurations (for future use - content, etc.)
    pub tab_configs: Vec<TabConfigData>,
    /// Tab bar configuration (stored as strings to avoid circular dependencies)
    pub config: TabBarConfigData,
    /// Last time a tab navigation occurred (for debouncing key repeats)
    pub last_navigation_time: Option<Instant>,
}

/// State-based colors for tab bars with type: state
#[derive(Debug, Clone)]
pub struct TabBarStateColors {
    /// Color for active state (e.g., "green")
    pub active: Option<String>,
    /// Color for negate state (e.g., "red")
    pub negate: Option<String>,
    /// Color for disabled state (None = use default)
    pub disabled: Option<String>,
}

/// Tab bar configuration data stored in registry
#[derive(Debug, Clone)]
pub struct TabBarConfigData {
    /// Handle name (HWND)
    pub hwnd: String,
    /// Anchor HWND name
    pub anchor: String,
    /// Style as string (will be parsed when needed)
    pub style: String,
    /// Color as string (will be parsed when needed)
    pub color: String,
    /// Tab bar type (e.g., "state" for state-based coloring)
    pub tab_bar_type: Option<String>,
    /// State-based colors (for type: state)
    pub state_colors: Option<TabBarStateColors>,
    /// Alignment configuration
    pub alignment: AlignmentConfigData,
    /// Minimum tab width
    pub min_tab_width: u16,
    /// Show tooltips
    pub tab_tooltips: bool,
}

/// Alignment configuration data
#[derive(Debug, Clone)]
pub struct AlignmentConfigData {
    /// Vertical position: "top" or "bottom"
    pub vertical: String,
    /// Horizontal alignment: "left", "center", or "right"
    pub horizontal: String,
    /// Horizontal offset
    pub offset_x: u16,
    /// Vertical offset
    pub offset_y: u16,
}

/// Tab state for state-based coloring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    /// Default state (no special coloring)
    Default,
    /// Active state (e.g., green)
    Active,
    /// Negate state (e.g., red)
    Negate,
    /// Disabled state (uses default or specified color)
    Disabled,
}

/// Tab configuration data stored in registry
#[derive(Debug, Clone)]
pub struct TabConfigData {
    pub id: String,
    pub name: String,
    pub active: bool,
    /// State for state-based coloring (for tab bars with type: state)
    pub state: TabState,
}

/// Registry for tracking rendered rectangles with handles
#[derive(Debug, Clone)]
pub struct RectRegistry {
    /// Map of handle ID to registry entry
    handles: HashMap<u64, RegistryEntry>,
    /// Map of name to handle ID (for lookup by name)
    name_to_handle: HashMap<String, u64>,
    /// Next handle ID to assign
    next_id: u64,
    /// Tab bar state storage (keyed by handle ID)
    tab_bar_states: HashMap<u64, TabBarState>,
}

impl RectRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            name_to_handle: HashMap::new(),
            next_id: 1, // Start at 1, 0 can be used as invalid handle
            tab_bar_states: HashMap::new(),
        }
    }

    /// Register a rectangle and return a handle
    /// If a name is provided and a rectangle with that name already exists, it will be updated
    pub fn register(&mut self, name: Option<&str>, rect: Rect) -> RectHandle {
        let metrics = RectMetrics::from(rect);
        
        // If name provided, check if it already exists
        if let Some(name_str) = name {
            if let Some(&existing_handle_id) = self.name_to_handle.get(name_str) {
                // Update existing entry
                if let Some(entry) = self.handles.get_mut(&existing_handle_id) {
                    entry.metrics = metrics;
                    return RectHandle(existing_handle_id);
                }
            }
        }
        
        // Create new handle
        let handle_id = self.next_id;
        self.next_id += 1;
        let handle = RectHandle(handle_id);
        
        // Create entry
        let entry = RegistryEntry {
            name: name.map(|s| s.to_string()),
            metrics,
        };
        
        // Store entry
        self.handles.insert(handle_id, entry.clone());
        
        // Store name mapping if provided
        if let Some(name_str) = name {
            self.name_to_handle.insert(name_str.to_string(), handle_id);
        }
        
        handle
    }

    /// Update an existing rectangle's metrics by handle
    pub fn update(&mut self, handle: RectHandle, rect: Rect) -> bool {
        if let Some(entry) = self.handles.get_mut(&handle.0) {
            entry.metrics = RectMetrics::from(rect);
            true
        } else {
            false
        }
    }

    /// Update an existing rectangle's metrics by name
    pub fn update_by_name(&mut self, name: &str, rect: Rect) -> bool {
        if let Some(&handle_id) = self.name_to_handle.get(name) {
            if let Some(entry) = self.handles.get_mut(&handle_id) {
                entry.metrics = RectMetrics::from(rect);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Set absolute Y position by handle
    pub fn set_y(&mut self, handle: RectHandle, y: u16) -> bool {
        if let Some(entry) = self.handles.get_mut(&handle.0) {
            entry.metrics.y = y;
            true
        } else {
            false
        }
    }

    /// Set absolute Y position by name
    pub fn set_y_by_name(&mut self, name: &str, y: u16) -> bool {
        if let Some(&handle_id) = self.name_to_handle.get(name) {
            if let Some(entry) = self.handles.get_mut(&handle_id) {
                entry.metrics.y = y;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Set absolute X position by handle
    pub fn set_x(&mut self, handle: RectHandle, x: u16) -> bool {
        if let Some(entry) = self.handles.get_mut(&handle.0) {
            entry.metrics.x = x;
            true
        } else {
            false
        }
    }

    /// Set absolute X position by name
    pub fn set_x_by_name(&mut self, name: &str, x: u16) -> bool {
        if let Some(&handle_id) = self.name_to_handle.get(name) {
            if let Some(entry) = self.handles.get_mut(&handle_id) {
                entry.metrics.x = x;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Offset Y position relative to current position by handle
    /// Positive values move down, negative values move up (using saturating arithmetic)
    pub fn offset_y(&mut self, handle: RectHandle, offset: i16) -> bool {
        if let Some(entry) = self.handles.get_mut(&handle.0) {
            if offset >= 0 {
                entry.metrics.y = entry.metrics.y.saturating_add(offset as u16);
            } else {
                entry.metrics.y = entry.metrics.y.saturating_sub((-offset) as u16);
            }
            true
        } else {
            false
        }
    }

    /// Offset Y position relative to current position by name
    /// Positive values move down, negative values move up (using saturating arithmetic)
    pub fn offset_y_by_name(&mut self, name: &str, offset: i16) -> bool {
        if let Some(&handle_id) = self.name_to_handle.get(name) {
            if let Some(entry) = self.handles.get_mut(&handle_id) {
                if offset >= 0 {
                    entry.metrics.y = entry.metrics.y.saturating_add(offset as u16);
                } else {
                    entry.metrics.y = entry.metrics.y.saturating_sub((-offset) as u16);
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Offset X position relative to current position by handle
    /// Positive values move right, negative values move left (using saturating arithmetic)
    pub fn offset_x(&mut self, handle: RectHandle, offset: i16) -> bool {
        if let Some(entry) = self.handles.get_mut(&handle.0) {
            if offset >= 0 {
                entry.metrics.x = entry.metrics.x.saturating_add(offset as u16);
            } else {
                entry.metrics.x = entry.metrics.x.saturating_sub((-offset) as u16);
            }
            true
        } else {
            false
        }
    }

    /// Offset X position relative to current position by name
    /// Positive values move right, negative values move left (using saturating arithmetic)
    pub fn offset_x_by_name(&mut self, name: &str, offset: i16) -> bool {
        if let Some(&handle_id) = self.name_to_handle.get(name) {
            if let Some(entry) = self.handles.get_mut(&handle_id) {
                if offset >= 0 {
                    entry.metrics.x = entry.metrics.x.saturating_add(offset as u16);
                } else {
                    entry.metrics.x = entry.metrics.x.saturating_sub((-offset) as u16);
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get current metrics for a handle
    pub fn get_metrics(&self, handle: RectHandle) -> Option<RectMetrics> {
        self.handles.get(&handle.0).map(|entry| entry.metrics)
    }

    /// Get current metrics by name
    pub fn get_metrics_by_name(&self, name: &str) -> Option<RectMetrics> {
        self.name_to_handle
            .get(name)
            .and_then(|&handle_id| self.handles.get(&handle_id))
            .map(|entry| entry.metrics)
    }

    /// Get handle by name
    pub fn get_handle(&self, name: &str) -> Option<RectHandle> {
        self.name_to_handle.get(name).map(|&id| RectHandle(id))
    }

    /// Get name for a handle (if it was registered with a name)
    pub fn get_name(&self, handle: RectHandle) -> Option<&String> {
        self.handles.get(&handle.0).and_then(|entry| entry.name.as_ref())
    }

    /// Remove a rectangle from the registry by handle
    pub fn unregister(&mut self, handle: RectHandle) -> bool {
        if let Some(entry) = self.handles.remove(&handle.0) {
            if let Some(name) = entry.name {
                self.name_to_handle.remove(&name);
            }
            true
        } else {
            false
        }
    }

    /// Remove a rectangle from the registry by name
    pub fn unregister_by_name(&mut self, name: &str) -> bool {
        if let Some(handle_id) = self.name_to_handle.remove(name) {
            self.handles.remove(&handle_id);
            true
        } else {
            false
        }
    }

    /// Get all registered handles
    pub fn all_handles(&self) -> Vec<RectHandle> {
        self.handles.keys().map(|&id| RectHandle(id)).collect()
    }

    /// Get all registered names
    pub fn all_names(&self) -> Vec<&String> {
        self.name_to_handle.keys().collect()
    }

    /// Set tab bar state for a handle
    pub fn set_tab_bar_state(&mut self, handle: RectHandle, state: TabBarState) {
        self.tab_bar_states.insert(handle.0, state);
    }

    /// Get tab bar state for a handle
    pub fn get_tab_bar_state(&self, handle: RectHandle) -> Option<&TabBarState> {
        self.tab_bar_states.get(&handle.0)
    }

    /// Get mutable tab bar state for a handle
    pub fn get_tab_bar_state_mut(&mut self, handle: RectHandle) -> Option<&mut TabBarState> {
        self.tab_bar_states.get_mut(&handle.0)
    }

    /// Update active tab index for a tab bar handle
    pub fn set_active_tab(&mut self, handle: RectHandle, active_index: usize) -> bool {
        if let Some(state) = self.tab_bar_states.get_mut(&handle.0) {
            if active_index < state.tab_count {
                state.active_tab_index = active_index;
                return true;
            }
        }
        false
    }

    /// Navigate to a tab with minimal debouncing to prevent hardware bounce
    /// Key repeat events are filtered at the event handler level, so this is just for hardware safety
    /// Returns true if navigation occurred, false if debounced
    pub fn navigate_tab(&mut self, handle: RectHandle, direction: i32) -> bool {
        const DEBOUNCE_DURATION: Duration = Duration::from_millis(50); // Reduced from 150ms since key repeats are filtered at event level
        
        if let Some(state) = self.tab_bar_states.get_mut(&handle.0) {
            // Check debounce - only allow navigation if enough time has passed (prevents hardware bounce)
            let now = Instant::now();
            if let Some(last_time) = state.last_navigation_time {
                if now.duration_since(last_time) < DEBOUNCE_DURATION {
                    return false; // Debounced - too soon since last navigation
                }
            }
            
            // Calculate new index
            let current_index = state.active_tab_index;
            let new_index = if direction < 0 {
                // Navigate left/previous
                if current_index > 0 {
                    current_index - 1
                } else {
                    state.tab_count - 1
                }
            } else {
                // Navigate right/next
                (current_index + 1) % state.tab_count
            };
            
            // Update tab and record navigation time
            if new_index < state.tab_count {
                state.active_tab_index = new_index;
                state.last_navigation_time = Some(now);
                return true;
            }
        }
        false
    }

    /// Set state for a specific tab (for state-based coloring)
    /// Returns true if state was set, false if tab not found
    pub fn set_tab_state(&mut self, handle: RectHandle, tab_index: usize, state: TabState) -> bool {
        if let Some(tab_bar_state) = self.tab_bar_states.get_mut(&handle.0) {
            if tab_index < tab_bar_state.tab_configs.len() {
                tab_bar_state.tab_configs[tab_index].state = state;
                return true;
            }
        }
        false
    }

    /// Get state for a specific tab
    pub fn get_tab_state(&self, handle: RectHandle, tab_index: usize) -> Option<TabState> {
        if let Some(tab_bar_state) = self.tab_bar_states.get(&handle.0) {
            if tab_index < tab_bar_state.tab_configs.len() {
                return Some(tab_bar_state.tab_configs[tab_index].state);
            }
        }
        None
    }

    /// Get active tab index for a tab bar handle
    pub fn get_active_tab(&self, handle: RectHandle) -> Option<usize> {
        self.tab_bar_states.get(&handle.0).map(|s| s.active_tab_index)
    }

    /// Clear all registered rectangles
    pub fn clear(&mut self) {
        self.handles.clear();
        self.name_to_handle.clear();
        self.tab_bar_states.clear();
        self.next_id = 1; // Reset ID counter
    }

    /// Check if a handle exists
    pub fn exists(&self, handle: RectHandle) -> bool {
        self.handles.contains_key(&handle.0)
    }

    /// Check if a name exists
    pub fn name_exists(&self, name: &str) -> bool {
        self.name_to_handle.contains_key(name)
    }
}

impl Default for RectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_query() {
        let mut registry = RectRegistry::new();
        let rect = Rect {
            x: 10,
            y: 20,
            width: 100,
            height: 50,
        };

        let handle = registry.register(Some("test-window"), rect);
        
        let metrics = registry.get_metrics(handle).unwrap();
        assert_eq!(metrics.x, 10);
        assert_eq!(metrics.y, 20);
        assert_eq!(metrics.width, 100);
        assert_eq!(metrics.height, 50);
    }

    #[test]
    fn test_update_metrics() {
        let mut registry = RectRegistry::new();
        let rect1 = Rect { x: 10, y: 20, width: 100, height: 50 };
        let handle = registry.register(Some("test"), rect1);

        let rect2 = Rect { x: 15, y: 25, width: 110, height: 60 };
        registry.update(handle, rect2);

        let metrics = registry.get_metrics(handle).unwrap();
        assert_eq!(metrics.x, 15);
        assert_eq!(metrics.y, 25);
        assert_eq!(metrics.width, 110);
        assert_eq!(metrics.height, 60);
    }

    #[test]
    fn test_lookup_by_name() {
        let mut registry = RectRegistry::new();
        let rect = Rect { x: 10, y: 20, width: 100, height: 50 };
        let handle = registry.register(Some("my-window"), rect);

        let found_handle = registry.get_handle("my-window").unwrap();
        assert_eq!(handle, found_handle);

        let metrics = registry.get_metrics_by_name("my-window").unwrap();
        assert_eq!(metrics.x, 10);
    }

    #[test]
    fn test_set_y_absolute() {
        let mut registry = RectRegistry::new();
        let rect = Rect { x: 10, y: 20, width: 100, height: 50 };
        let handle = registry.register(Some("test-window"), rect);

        // Set absolute Y position
        registry.set_y(handle, 12);
        let metrics = registry.get_metrics(handle).unwrap();
        assert_eq!(metrics.y, 12);
        assert_eq!(metrics.x, 10); // X should remain unchanged

        // Set absolute Y by name
        registry.set_y_by_name("test-window", 25);
        let metrics = registry.get_metrics_by_name("test-window").unwrap();
        assert_eq!(metrics.y, 25);
    }

    #[test]
    fn test_offset_y_relative() {
        let mut registry = RectRegistry::new();
        let rect = Rect { x: 10, y: 20, width: 100, height: 50 };
        let handle = registry.register(Some("test-window"), rect);

        // Move down by 3 (relative)
        registry.offset_y(handle, 3);
        let metrics = registry.get_metrics(handle).unwrap();
        assert_eq!(metrics.y, 23); // 20 + 3

        // Move up by 5 (relative, negative offset)
        registry.offset_y(handle, -5);
        let metrics = registry.get_metrics(handle).unwrap();
        assert_eq!(metrics.y, 18); // 23 - 5

        // Move by name
        registry.offset_y_by_name("test-window", 10);
        let metrics = registry.get_metrics_by_name("test-window").unwrap();
        assert_eq!(metrics.y, 28); // 18 + 10
    }

    #[test]
    fn test_set_x_and_offset_x() {
        let mut registry = RectRegistry::new();
        let rect = Rect { x: 10, y: 20, width: 100, height: 50 };
        let handle = registry.register(Some("test-window"), rect);

        // Set absolute X
        registry.set_x(handle, 15);
        let metrics = registry.get_metrics(handle).unwrap();
        assert_eq!(metrics.x, 15);

        // Offset X relative
        registry.offset_x(handle, -3);
        let metrics = registry.get_metrics(handle).unwrap();
        assert_eq!(metrics.x, 12); // 15 - 3
    }
}

/// Helper function to render a widget and register its rectangle
/// This allows automatic registration of rectangles during rendering
pub fn render_with_handle<W: ratatui::widgets::Widget>(
    frame: &mut ratatui::Frame,
    registry: &mut RectRegistry,
    name: Option<&str>,
    widget: W,
    area: Rect,
) -> RectHandle {
    // Render the widget
    frame.render_widget(widget, area);
    
    // Register the rectangle and return handle
    registry.register(name, area)
}

