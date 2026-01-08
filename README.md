# Sync Manager

A Terminal User Interface (TUI) application for managing file synchronization across projects and shared resources.

## Features

- **Visual Diff Viewer**: See changes between shared resources and project files
- **Side-by-Side Comparison**: Word-level diff highlighting with folding support
- **Bidirectional Sync**: Sync files from shared to project or project to shared
- **Git Integration**: Track repository status and manage commits
- **Modular Architecture**: Clean separation of concerns for easy maintenance

## Project Structure

```
sync-manager/
├── Cargo.toml              # Rust package manifest
├── sync-manager.yaml       # Project configuration (per-project, only file needed at runtime)
├── README.md
├── build.rs                # Build script (compiles src/config.yaml into binary)
└── src/
    ├── config.yaml         # Development defaults (compiled into binary)
    ├── main.rs             # Entry point
    ├── lib.rs              # Library exports
    ├── core/               # Core infrastructure
    │   ├── mod.rs
    │   ├── app.rs          # Application state management
    │   ├── app_config.rs   # Config (compiled from config.yaml)
    │   ├── project_config.rs # Project config (sync-manager.yaml)
    │   └── events.rs       # Event handling
    ├── operations/         # Business logic
    │   ├── mod.rs
    │   ├── diff.rs         # Diff computation engine
    │   ├── sync.rs         # File synchronization
    │   └── git.rs          # Git operations
    ├── ui/                 # TUI components
    │   ├── mod.rs
    │   ├── app_view.rs     # Main application layout
    │   ├── diff_list.rs    # File list component
    │   ├── diff_view.rs    # Unified diff view
    │   ├── side_by_side.rs # Side-by-side diff view
    │   └── styles.rs       # Color scheme and styling
    └── utilities/          # Helper functions
        ├── mod.rs
        ├── paths.rs        # Path manipulation
        └── patterns.rs     # Pattern matching
```

## Configuration

### Project Config (`sync-manager.yaml`)

The compiled executable looks for `sync-manager.yaml` in the project/workspace root. This file defines what to sync:

```yaml
workspace_settings:
  my-project:
    shared-cursor:
      mappings:
        - shared: _shared-resources/shared-cursor/rules
          project: .cursor/rules

managed_packages:
  - name: shared-cursor
    enabled: true
    location: shared-cursor

global_settings:
  sync_direction: both
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Tab` | Switch between views |
| `↑/↓` or `j/k` | Navigate list / Scroll diff |
| `Enter/Space` | Toggle side-by-side view |
| `f` | Toggle fold unchanged regions |
| `PgUp/PgDn` | Scroll diff view |
| `Esc` | Go back / Exit current view |
| `r` | Refresh diffs |
| `s` | Sync selected file |
| `S` | Sync all files |

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## Usage

1. Copy `sync-manager.yaml.example` to your project root as `sync-manager.yaml`
2. Configure your mappings and packages
3. Run `sync-manager` from your project root

```bash
# Run from project directory
./sync-manager

# Or specify workspace root
WORKSPACE_ROOT=/path/to/project ./sync-manager
```

## License

MIT
