# Rust TUI Development Structure

## Overview

This directory structure is designed to optimize Rust TUI (Terminal User Interface) development by centralizing shared resources and reducing code duplication across multiple TUI applications. The structure follows the DRY (Don't Repeat Yourself) principle and leverages Rust's workspace features to minimize disk space usage and simplify dependency management.

## Directory Structure

```
rust/
├── Cargo.toml              # Workspace root configuration
├── Rust-TUI-Structure.md   # This documentation
├── resources/              # Shared resources for all TUI applications
│   ├── components/         # Reusable TUI components library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── base_layout.rs
│   │       ├── file_browser.rs
│   │       ├── form_panel.rs
│   │       ├── helpers.rs
│   │       ├── list_panel.rs
│   │       ├── managers/
│   │       │   ├── mod.rs
│   │       │   ├── bounding_box.rs
│   │       │   └── tab_bar.rs
│   │       ├── popup.rs
│   │       ├── rect_handle.rs
│   │       ├── tab_bar.rs
│   │       └── toast.rs
│   └── crates/             # Additional shared crates (future use)
└── dev/                    # Individual TUI application projects
    ├── bootstrapper/       # Bootstrapper TUI
    ├── chamon/             # Chamon TUI
    ├── detour/             # Detour TUI
    └── hasync/             # Hsync TUI
```

## Design Goals

### 1. Space Efficiency
By using a Rust workspace with shared dependencies and components, we significantly reduce the overall disk space required. Instead of each TUI application maintaining its own copy of:
- Common dependencies (ratatui, crossterm, serde, etc.)
- Shared UI components (file browsers, form panels, list panels, toast notifications)
- Utility functions and helpers

All of these are centralized in the `resources/` directory and referenced by each TUI project, eliminating duplication.

### 2. Code Reusability
The `resources/components/` crate provides a library of pre-built, DRY (Don't Repeat Yourself) components that can be included and used across all TUI applications. This ensures:
- Consistent UI/UX across all applications
- Single source of truth for component implementations
- Easier maintenance and bug fixes (fix once, benefit everywhere)
- Faster development of new TUI applications

### 3. Dependency Management
The workspace-level `Cargo.toml` defines all shared dependencies in one place, ensuring:
- Consistent dependency versions across all projects
- Simplified dependency updates (update once, apply everywhere)
- Reduced compilation time through shared dependency compilation
- Better dependency resolution and conflict management

### 4. Project Organization
Each TUI application in the `dev/` directory maintains its own:
- Application-specific logic and state management
- Unique UI layouts and workflows
- Project-specific configuration
- Binary entry points

This separation allows each TUI to be developed independently while still benefiting from shared resources.

## Component Structure

### Shared Components (`resources/components/`)

The `tui-components` crate provides reusable components that are used across multiple TUI applications:

- **`base_layout.rs`**: Base layout component providing title, status bar, and content area management
- **`file_browser.rs`**: File browser component for navigating file systems
- **`form_panel.rs`**: Form input panel with validation
- **`helpers.rs`**: Utility functions and helper methods
- **`list_panel.rs`**: Scrollable list panel component
- **`managers/`**: OOP-style manager wrappers for TUI components
  - **`bounding_box.rs`**: `BoundingBox` manager for OOP-style bounding box operations with handle support
  - **`tab_bar.rs`**: `TabBarManager` wrapper and YAML configuration helpers for tab bar operations
- **`popup.rs`**: Popup dialog component with dimming support
- **`rect_handle.rs`**: Handle-based registry system (HWND-like) for managing UI element positioning and state
- **`tab_bar.rs`**: Tab bar component with multiple styles (tabbed, boxed, text, static variants) and state-based coloring
- **`toast.rs`**: Toast notification system

#### Rect Handle System

The `rect_handle.rs` module provides a handle-based (HWND-like) system for managing UI element positioning. The `RectRegistry` maintains a mapping of handle names to rectangle metrics, allowing elements to be referenced by name and positioned relative to each other.

**Key Features:**
- **Handle-Based Access**: Elements can be registered with unique handle names (e.g., "hwndMainContentBox")
- **Property Access**: Direct access to x, y, width, height properties
- **Relative Positioning**: Elements can be positioned relative to other elements
- **State Management**: Supports storing additional state (e.g., tab bar state) associated with handles

**BoundingBox Manager Pattern:**

The `managers/bounding_box.rs` module provides a `BoundingBox` wrapper struct for OOP-like bounding box management. Applications can use it similar to `TabBarManager`:

```rust
use tui_components::{BoundingBox, get_box_by_name};

const HWND_MAIN_CONTENT_BOX: &str = "hwndMainContentBox";

// Create bounding box
let main_box = BoundingBox::create(&mut registry, HWND_MAIN_CONTENT_BOX, rect);

// Access properties
let x = main_box.x(&registry);
main_box.set_x(&mut registry, 10);

// Relative positioning
let box2 = BoundingBox::create(&mut registry, "box2", rect2);
box2.set_relative_x(&mut registry, &main_box, 1);  // box2.x = main_box.x + 1
box2.set_relative_y(&mut registry, &main_box, 1);  // box2.y = main_box.y + 1

// Prepare and render
if let Some(rect) = main_box.prepare(&mut registry) {
    // Use rect for rendering
}
main_box.render(&mut f, &mut registry, &dimming);

// Helper functions
let box_ref = get_box_by_name(&registry, HWND_MAIN_CONTENT_BOX);
```

These components are designed to be generic and configurable, allowing each TUI application to customize them for their specific needs while maintaining a consistent base implementation.

#### Tab Bar Component

The tab bar component supports an OOP-like approach using handle-based identifiers (HWND-like). Applications can use the `TabBarManager` wrapper (from `managers/tab_bar.rs`) to associate all tab bar operations with a handle identifier:

```rust
use tui_components::TabBarManager;

// Create tab bar manager (OOP-like: object associated with handle)
let main_content_tab_bar = TabBarManager::create(&mut registry, HWND_MAIN_CONTENT_TAB_BAR, tab_bar_config);

// All operations are accessed through the manager object
main_content_tab_bar.navigate_previous(&mut registry);
main_content_tab_bar.navigate_next(&mut registry);
main_content_tab_bar.set_active(&mut registry, index);
```

The tab bar supports:
- **Multiple Styles**: `tabbed`, `boxed`, `text`, `box_static`, `text_static`
- **State-Based Coloring**: For static tab bars, tabs can have states (active, negate, disabled) with custom colors
- **Handle-Based Management**: Tab bars are referenced by unique handle names (HWND-like identifiers) stored in the `RectRegistry`
- **Keyboard Navigation**: Built-in support for arrow keys and custom key bindings
- **Mouse Interaction**: Click detection for tab selection
- **Automatic Anchor Adjustment**: When using Tab style with handle-based positioning (TopOfHandle/BottomOfHandle), the anchor box is automatically adjusted (y+1, height-1) to provide proper spacing for the tab bar

**Tab Style Anchor Adjustment:**

When a tab bar uses Tab style and is attached to a bounding box via handle-based positioning, the component automatically adjusts the anchor box in the `from_registry()` prepare phase:
- Moves the anchor box down by 1 line (y+1)
- Reduces the anchor box height by 1 (height-1)

This adjustment happens before rendering, allowing other elements to calculate relative positions correctly. The tab bar then attaches directly to the adjusted anchor position without gaps.

```rust
// Tab bar anchor is automatically adjusted in from_registry() when Tab style is used
let tab_bar_result = main_content_tab_bar.prepare(&mut registry, Some(tab_style));
// Anchor box (HWND_MAIN_CONTENT_BOX) is now adjusted (y+1, height-1)

// Render content using the adjusted anchor box
let render_area = get_box_by_name(&registry, HWND_MAIN_CONTENT_BOX)
    .and_then(|b| b.prepare(&mut registry))
    .unwrap_or(content_area);
render_content(f, render_area, &dimming);

// Render tab bar (attaches to the adjusted anchor position)
tab_bar.render_with_state(f, &mut registry, &tab_bar_state, Some(&dimming));
```

#### Managers Module

The `managers/` subdirectory provides OOP-style manager wrappers for TUI components:

**Tab Bar Manager (`managers/tab_bar.rs`)**:
- **`TabBarManager`**: OOP-style wrapper for tab bar operations
- **YAML Configuration Structures**: `TabBarConfigYaml`, `TabConfigYaml`, `AlignmentConfigYaml`, `TabBarColorsYaml` - serde-deserializable structures for YAML configuration
- **Conversion Functions**: `convert_tab_bar_config()` and `create_tab_configs()` to transform YAML structures to internal registry structures
- **Initialization Helper**: `create_tab_bar_from_config()` - convenience function that combines conversion and initialization

```rust
use tui_components::{TabBarConfigYaml, TabBarManager, create_tab_bar_from_config};

// Load YAML config (using serde_yaml)
let config: TabBarConfigYaml = serde_yaml::from_str(&yaml_contents)?;

// Option 1: Use TabBarManager wrapper
let tab_bar_manager = TabBarManager::create(&mut registry, "my_tab_bar", &config);

// Option 2: Direct initialization
let handle = create_tab_bar_from_config(&mut registry, "my_tab_bar", &config);
```

**Bounding Box Manager (`managers/bounding_box.rs`)**:
- **`BoundingBox`**: OOP-style wrapper for bounding box operations with handle support
- **Helper Functions**: `get_box_by_name()` and `list_all_boxes()` for working with bounding boxes

This design keeps components decoupled from YAML/file I/O while providing reusable configuration handling utilities and OOP-style management patterns.

### Individual TUI Projects (`dev/`)

Each TUI project follows a similar structure based on the patterns established in the original `chamon` and `detour` projects:

#### Common Modules
- **`app.rs`**: Application state management
- **`config.rs`**: Configuration loading and management
- **`events.rs`**: Event handling and input processing
- **`ui.rs`**: UI rendering and layout
- **`main.rs`**: Binary entry point
- **`lib.rs`**: Library exports for use as a dependency

#### Project-Specific Modules
Some projects may have additional modules based on their specific functionality:
- **`chamon`**: Includes `baseline.rs` for baseline management
- **`detour`**: Includes modules for file operations, diff, injection, mirror, validation, and more

## Workspace Configuration

The root `Cargo.toml` defines a Rust workspace that includes:
- All TUI projects in `dev/`
- The shared components library in `resources/components/`

This configuration enables:
- Building all projects with a single `cargo build` command
- Running tests across all projects
- Sharing compiled dependencies across projects
- Unified dependency version management

## Usage

### Building All Projects
```bash
cd /root/_playground/rust
cargo build
```

### Building a Specific Project
```bash
cd /root/_playground/rust
cargo build -p bootstrapper
cargo build -p chamon-tui
cargo build -p detour
cargo build -p hasync
```

### Running a TUI Application
```bash
cd /root/_playground/rust
cargo run --bin bootstrapper
cargo run --bin chamon
cargo run --bin detour
cargo run --bin hasync
```

### Using Shared Components
In any TUI project's `Cargo.toml`:
```toml
[dependencies]
tui-components = { path = "../../resources/components" }
```

Then in your Rust code:
```rust
use tui_components::{
    FileBrowser, FormPanel, ListPanel, Toast,
    TabBar, TabBarConfigYaml, TabBarManager, BoundingBox,
    create_tab_bar_from_config, get_box_by_name,
};
```

## Benefits

1. **Reduced Disk Space**: Shared dependencies and components mean less duplication
2. **Faster Development**: Reusable components speed up new TUI development
3. **Consistency**: Shared components ensure consistent UI/UX across applications
4. **Maintainability**: Bug fixes and improvements to shared components benefit all applications
5. **Simplified Updates**: Update dependencies once at the workspace level
6. **Better Organization**: Clear separation between shared resources and application-specific code

## Future Enhancements

The `resources/crates/` directory is reserved for additional shared crates that may be needed in the future, such as:
- Shared configuration management
- Common data structures
- Utility libraries
- Shared networking or I/O abstractions

## Migration Notes

This structure is based on the existing TUI projects:
- `/root/RPi-Full/_playground/_dev/packages/chamon`
- `/root/RPi-Full/_playground/_dev/packages/detour`
- `/root/RPi-Full/_playground/_dev/packages/_tui-components`

The skeleton structure maintains compatibility with the original project layouts while providing the benefits of centralized resource management.

