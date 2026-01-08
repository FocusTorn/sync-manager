// Shared TUI components library
// Reusable components for all TUI applications

// Core infrastructure
pub mod core;
// GUI elements (visual components)
pub mod elements;
// OOP-style manager wrappers
pub mod managers;
// Utilities and helpers
pub mod utilities;

// Re-export commonly used items
// Note: ambiguous_glob_reexports warning is intentional - tab_bar exists in both elements and managers
// but refers to different types (TabBar struct vs TabBarManager), so disambiguation is expected
#[allow(ambiguous_glob_reexports)]
pub use core::*;
#[allow(ambiguous_glob_reexports)]
pub use elements::*;
#[allow(ambiguous_glob_reexports)]
pub use managers::*;
pub use utilities::*;

