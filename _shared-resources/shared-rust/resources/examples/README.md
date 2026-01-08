# Component Examples

Standalone examples demonstrating individual TUI components. These examples are separate from any main TUI application and can be run independently.

## Layout Demo - Tabbed Manager

Demonstrates the `BaseLayout` component integrated with the `TabBar` component, showing a complete TUI application structure with:
- Title bar
- Tab bar with multiple styles (tabbed, boxed, text, static variants)
- Content area
- Status bar
- Popup and toast notifications
- Handle-based (HWND-like) UI element management via `RectRegistry`
- OOP-like tab bar management via `TabBarManager`
- **BoundingBox manager** (`managers/bounding_box.rs`) for OOP-style bounding box operations with handle support

### Features

- **Tab Bar Styles**: Switch between different tab bar styles using keys 7, 8, 9
- **Tab Navigation**: Use Left/Right arrows or h/l keys to navigate between tabs
- **State-Based Coloring**: Static tab bars support state-based coloring (active, negate, disabled)
- **Mouse Interaction**: Click on tabs to select them
- **Popup System**: Press 'p' to cycle through popup types (info, warning, error)
- **Toast Notifications**: Press 't' to show toast notifications

### Configuration

The application loads configuration from `config.yaml`, which defines:
- Application title and bindings
- Tab bar configuration (style, alignment, colors, tabs)
- Tab states for static tab bars

The tab bar configuration uses the `tui-components` library's `managers/tab_bar.rs` module, which provides:
- YAML-deserializable structures (`TabBarConfigYaml`, `TabConfigYaml`)
- Conversion functions to transform YAML config to internal structures
- A convenience function `create_tab_bar_from_config()` that handles the complete initialization

### Running

```bash
cd resources/examples
cargo run --bin LayoutDemo__TabbedManager
```

### Controls

- **7/8/9**: Switch tab bar style (tabbed/boxed/text)
- **Left/Right arrows or h/l**: Navigate between tabs
- **Mouse Click**: Click on a tab to select it
- **p**: Cycle through popup types (info → warning → error)
- **t**: Show toast notification
- **q or ESC**: Quit

### Architecture

The example demonstrates the OOP-like approach to tab bar management, bounding box management, and the use of the library's configuration helpers:

```rust
use tui_components::{TabBarManager, BoundingBox, get_box_by_name, TabBarConfigYaml};

// Define handle constants
const HWND_MAIN_CONTENT_BOX: &str = "hwndMainContentBox";
const HWND_MAIN_CONTENT_TAB_BAR: &str = "hwndMainContentTabBar";

// Load config from YAML
let app_config = load_config(None)?;
let tab_bar_config = app_config.tab_bars.get("main_content_tab_bar")?;

// Create tab bar manager (OOP-like: object associated with handle)
// Uses library's create_tab_bar_from_config() internally
let main_content_tab_bar = TabBarManager::create(&mut registry, HWND_MAIN_CONTENT_TAB_BAR, tab_bar_config);

// All operations are accessed through the manager object
main_content_tab_bar.navigate_previous(&mut registry);
main_content_tab_bar.navigate_next(&mut registry);
main_content_tab_bar.set_active(&mut registry, index);

// Create bounding box with handle name
let main_box = BoundingBox::create(&mut registry, HWND_MAIN_CONTENT_BOX, content_area);

// Access bounding box properties
let x = main_box.x(&registry);
main_box.set_y(&mut registry, 10);

// Relative positioning
let box2 = BoundingBox::create(&mut registry, "box2", rect2);
box2.set_relative_x(&mut registry, &main_box, 1);  // box2.x = main_box.x + 1

// Get bounding box by name (useful when you only have the handle name)
let box_ref = get_box_by_name(&registry, HWND_MAIN_CONTENT_BOX);
```

**Configuration Handling**: The `tui-components` library provides the `managers/tab_bar.rs` module which handles:
- YAML deserialization via `TabBarConfigYaml` and `TabConfigYaml` structures
- Conversion from YAML structures to internal `TabBarConfigData` and `TabConfigData`
- Complete initialization via `create_tab_bar_from_config()` function
- OOP-style management via `TabBarManager` wrapper

**Handle-Based Management**: 
- Tab bars are referenced by unique handle names (HWND-like identifiers) stored in the `RectRegistry`, allowing multiple tab bars to coexist and be managed independently
- Bounding boxes can also use handle names for easy reference and relative positioning
- The tab bar's anchor should reference the same handle as the bounding box it's attached to (e.g., both use `"hwndMainContentBox"`)

**Tab Style Anchor Adjustment**:
When Tab style is used with handle-based positioning, the tab bar component automatically adjusts the anchor box (y+1, height-1) during the `prepare()` phase. This happens before rendering, ensuring:
- The anchor box is moved down by 1 line to provide space for the tab bar
- The anchor box height is reduced by 1 to maintain proper spacing
- Other elements can calculate relative positions correctly using the adjusted anchor
- The tab bar attaches directly to the adjusted position without gaps

## Tab Bar Example

Shows the tab bar component with the Tab style, displaying DASHBOARD, CHANGES, and BASELINES tabs.

### Expected Output

With BASELINES as the active tab:
```
                        ╭───────────╮  
── DASHBOARD ─ CHANGES ─╯ BASELINES ╰──
```

### Running

```bash
cd resources/examples
cargo run --bin tab-bar-example
```

### Controls

- **↑/↓ or k/j**: Cycle through tabs
- **1/2/3**: Jump directly to DASHBOARD/CHANGES/BASELINES
- **q or ESC**: Quit

## Adding More Examples

To add a new component example:

1. Create a new binary in `src/` (e.g., `src/my_component_example.rs`)
2. Add a `[[bin]]` entry to `Cargo.toml`:
   ```toml
   [[bin]]
   name = "my-component-example"
   path = "src/my_component_example.rs"
   ```
3. Build and run:
   ```bash
   cargo run --bin my-component-example --package component-examples
   ```

