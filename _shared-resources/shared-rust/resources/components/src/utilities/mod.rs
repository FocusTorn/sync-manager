// Utilities module
// Helper functions and quality of life utilities

pub mod helpers;
// pub mod layout_calculator;  // TODO: Uncomment when split_diff_view is enabled
pub mod syntax_highlighting;

pub use helpers::*;
// pub use layout_calculator::LayoutCalculator;  // TODO: Uncomment when split_diff_view is enabled
pub use syntax_highlighting::{SyntaxHighlighter, get_file_extension};

