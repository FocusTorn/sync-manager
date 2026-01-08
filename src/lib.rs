// Sync Manager Library
// A modular TUI application for managing file synchronization across projects

// Core infrastructure - foundational systems
pub mod core;

// Operations - business logic for sync operations
pub mod operations;

// UI - TUI components and views
pub mod ui;

// Utilities - helper functions and tools
pub mod utilities;

// Dashboard state and rendering
pub mod dashboard;

// Application constants
pub mod constants;

// Re-export commonly used items for convenience
pub use core::{App, AppConfig, ProjectConfig};
pub use operations::{DiffEngine, SyncEngine, GitOps};
pub use dashboard::DashboardState;
pub use constants::*;