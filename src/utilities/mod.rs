// Utilities module
// Helper functions and tools

pub mod paths;
pub mod patterns;

pub use paths::{normalize_path, resolve_path};
pub use patterns::{matches_pattern, PatternMatcher};
