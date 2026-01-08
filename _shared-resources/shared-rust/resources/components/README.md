# TUI Components Library

A comprehensive library of reusable Terminal User Interface (TUI) components for Rust applications, built on top of `ratatui` and `crossterm`.

## Structure

The library is organized into four main modules:

- **`core/`** - Core infrastructure and foundational systems
- **`elements/`** - Visual GUI components that render UI elements
- **`managers/`** - OOP-style convenience wrappers for easier component management
- **`utilities/`** - Helper functions and quality-of-life utilities

## Quick Start

```toml
[dependencies]
tui-components = { path = "../../resources/components" }
```

```rust
use tui_components::{
    // Core infrastructure
    RectRegistry, RectHandle, RectMetrics,
    
    // GUI Elements
    BaseLayout, TabBar, FileBrowser, FormPanel, ListPanel, Popup, Toast,
    
    // Managers (OOP-style wrappers)
    TabBarManager, BoundingBox, get_box_by_name,
    
    // Utilities
    DimmingContext,
};
```

## Core Infrastructure (`core/`)

### `RectRegistry` and `RectHandle`

Handle-based (HWND-like) system for managing UI element positioning and state.

```rust
use tui_components::{RectRegistry, RectHandle};

let mut registry = RectRegistry::new();

// Register a rectangle with a name (HWND-like)
let handle = registry.register(Some("mainContentBox"), rect);

// Get metrics
if let Some(metrics) = registry.get_metrics(handle) {
    println!("Position: {},{} size: {}x{}", 
             metrics.x, metrics.y, metrics.width, metrics.height);
}

// Update position
registry.set_x(handle, 10);
registry.set_y(handle, 5);
```

**Key Types:**
- `RectHandle` - Unique identifier for a registered rectangle
- `RectMetrics` - Position and size (x, y, width, height)
- `RectRegistry` - Central registry managing all handles

## GUI Elements (`elements/`)

### BaseLayout

Provides a complete application layout structure with title bar, content area, and status bar.

```rust
use tui_components::{BaseLayout, BaseLayoutConfig, BaseLayoutResult};

let config = BaseLayoutConfig {
    title: "My Application".to_string(),
    tabs: vec![],
    global_bindings: vec![],
    status_bar: StatusBarConfig::default(),
};

let mut layout = BaseLayout::new(config);

// In your render loop
let result = layout.render(f, &mut registry, &dimming);
match result {
    BaseLayoutResult::Content(content_area) => {
        // Render your content in content_area
    }
    BaseLayoutResult::Exit => {
        // Handle exit
    }
}
```

### TabBar

Flexible tab bar component with multiple styles and positioning options.

```rust
use tui_components::{TabBar, TabBarStyle, TabBarAlignment, TabBarPosition};

let mut tab_bar = TabBar::new(vec![
    TabBarItem { name: "Dashboard".to_string(), active: true, state: None },
    TabBarItem { name: "Settings".to_string(), active: false, state: None },
]);

// Render in different styles
tab_bar.render_with_state(f, &mut registry, &tab_bar_state, Some(&dimming));
```

**Styles:** `Tab`, `Box`, `Text`, `BoxStatic`, `TextStatic`

### FileBrowser

File system navigation component with directory browsing.

```rust
use tui_components::FileBrowser;

let mut file_browser = FileBrowser::new(PathBuf::from("/home/user"));
file_browser.render(f, area, &dimming, is_active);
```

### FormPanel

Form input panel with validation support.

```rust
use tui_components::FormPanel;

let mut form = FormPanel::new();
form.render(f, area, &dimming);
```

### ListPanel

Scrollable list panel component.

```rust
use tui_components::ListPanel;

let mut list = ListPanel::new(vec!["Item 1", "Item 2", "Item 3"]);
list.render(f, area, &dimming, is_active);
```

### Popup

Modal popup dialog with dimming support.

```rust
use tui_components::{Popup, PopupType};

let popup = Popup::new(
    PopupType::Info,
    "Confirmation",
    "Are you sure you want to continue?",
);
render_popup(f, &popup, &dimming);
```

### Toast

Toast notification system for temporary messages.

```rust
use tui_components::{Toast, ToastType};

let toast = Toast::new("Operation successful!", ToastType::Success);
toasts.push(toast);
render_toasts(f, &toasts, area);
```

## Managers (`managers/`)

OOP-style convenience wrappers that provide easier component management through handle-based identifiers.

### TabBarManager

Wrapper for tab bar operations with handle-based management.

```rust
use tui_components::{TabBarManager, TabBarConfigYaml};

const HWND_MAIN_TAB_BAR: &str = "mainTabBar";

// Create from YAML config
let tab_bar_manager = TabBarManager::create(&mut registry, HWND_MAIN_TAB_BAR, &config);

// All operations through the manager
tab_bar_manager.navigate_previous(&mut registry);
tab_bar_manager.navigate_next(&mut registry);
tab_bar_manager.set_active(&mut registry, 2);

// Prepare and render
if let Some((tab_bar, handle, state)) = tab_bar_manager.prepare(&mut registry, None) {
    tab_bar.render_with_state(f, &mut registry, &state, Some(&dimming));
}
```

### BoundingBox

OOP-style wrapper for bounding box operations with handle support.

```rust
use tui_components::{BoundingBox, get_box_by_name};

const HWND_MAIN_CONTENT: &str = "mainContentBox";

// Create a bounding box
let main_box = BoundingBox::create(&mut registry, HWND_MAIN_CONTENT, rect);

// Access properties
let x = main_box.x(&registry);
main_box.set_x(&mut registry, 10);
main_box.set_y(&mut registry, 5);

// Relative positioning
let box2 = BoundingBox::create(&mut registry, "box2", rect2);
box2.set_relative_x(&mut registry, &main_box, 1);  // box2.x = main_box.x + 1
box2.set_relative_y(&mut registry, &main_box, 1);  // box2.y = main_box.y + 1

// Prepare and render
if let Some(rect) = main_box.prepare(&mut registry) {
    // Use rect for rendering
}
main_box.render(&mut f, &mut registry, &dimming);

// Get by name
let box_ref = get_box_by_name(&registry, HWND_MAIN_CONTENT);
```

## Utilities (`utilities/`)

### DimmingContext

Manages dimming state for modal dialogs.

```rust
use tui_components::DimmingContext;

let mut dimming = DimmingContext::new(false);

// When modal opens
dimming.modal_visible = true;

// Use in rendering
let color = dimming.dim_color(Color::White);
```

### Helper Functions

```rust
use tui_components::{hex_color, centered_rect, get_text_color};

// Color conversion
let color = hex_color(0xFF5733);

// Layout utilities
let centered = centered_rect(50, 50, area);

// Styling
let text_color = get_text_color(is_active, dimming.modal_visible);
```

## Configuration

### YAML Configuration

Tab bars can be configured via YAML for easier setup:

```yaml
tab_bars:
  main_content_tab_bar:
    hwnd: "hwndMainContentTabBar"
    anchor: "hwndMainContentBox"
    style: "tab"
    color: "cyan"
    alignment:
      vertical: "top"
      horizontal: "left"
    tabs:
      - id: "dashboard"
        name: "Dashboard"
        default: "active"
      - id: "settings"
        name: "Settings"
```

```rust
use tui_components::{TabBarConfigYaml, TabBarManager};
use serde_yaml;

let yaml_str = std::fs::read_to_string("config.yaml")?;
let config: TabBarConfigYaml = serde_yaml::from_str(&yaml_str)?;

let tab_bar = TabBarManager::create(&mut registry, "myTabBar", &config);
```

## Examples

See `resources/examples/` for complete working examples:
- `LayoutDemo__TabbedManager` - Complete TUI application demonstrating all components

## Architecture Notes

### Why a Single Crate?

This library is organized as a **single crate** rather than separate crates because:

1. **Shared Infrastructure**: All components depend on `core/rect_handle` and `utilities/` modules
2. **Tight Integration**: Components are designed to work together (e.g., `TabBar` uses `RectRegistry`, `BaseLayout` uses `TabBar`)
3. **Consistent Versions**: Ensures all components use the same versions of dependencies (`ratatui`, `crossterm`, `serde`)
4. **Simplified Imports**: Single `use tui_components::*;` provides everything needed

### Module Organization

```
core/          - Foundation that other modules depend on
elements/      - Visual components that depend on core + utilities
managers/      - Convenience wrappers that depend on core + elements
utilities/     - Helper functions used by all modules
```

This organization allows for clear dependency flow and easy navigation of the codebase.

## Dependencies

- `ratatui = "0.29.0"` - TUI library
- `crossterm = "0.29.0"` - Terminal manipulation
- `serde = "1.0"` - Serialization (for YAML config support)

## License

MIT

