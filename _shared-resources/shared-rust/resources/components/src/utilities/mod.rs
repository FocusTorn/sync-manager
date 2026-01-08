// Utilities module
// Helper functions and quality of life utilities

pub mod helpers;
pub mod layout_calculator;
pub mod syntax_highlighting;

pub use helpers::*;
pub use layout_calculator::LayoutCalculator;
pub use syntax_highlighting::{SyntaxHighlighter, get_file_extension};

