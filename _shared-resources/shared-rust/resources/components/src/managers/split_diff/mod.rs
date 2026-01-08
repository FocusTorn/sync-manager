// Split Diff Manager
// OOP-style manager for split diff view operations
// Business logic is split into separate modules: diff_algorithms, line_wrapping, rendering

use crate::core::{RectHandle, RectRegistry};
use crate::elements::{SplitDiffViewConfig, SplitDiffViewState};
use crate::utilities::SyntaxHighlighter;

// Submodules
pub mod diff_algorithms;
pub mod line_wrapping;
pub mod rendering;

// Re-export types from submodules
pub use diff_algorithms::LineAlignment;
pub use rendering::{SplitDiffRenderData, RenderParams, compute_render_data_static};

/// Split Diff Manager wrapper for OOP-style diff view operations
/// Associates all diff view operations with a handle identifier
/// Delegates business logic to specialized modules
pub struct SplitDiffManager {
    handle: RectHandle,
    handle_name: String,
    config: SplitDiffViewConfig,
    state: SplitDiffViewState,
}

impl SplitDiffManager {
    /// Create a new split diff manager with the given configuration
    pub fn create(
        registry: &mut RectRegistry,
        handle_name: &str,
        config: SplitDiffViewConfig,
    ) -> Self {
        // Register the diff view area (will be set during rendering)
        let handle = registry.register(Some(handle_name), ratatui::layout::Rect::default());
        
        Self {
            handle,
            handle_name: handle_name.to_string(),
            config,
            state: SplitDiffViewState::default(),
        }
    }
    
    /// Get the handle (object identifier)
    pub fn handle(&self) -> RectHandle {
        self.handle
    }
    
    /// Get the handle name
    pub fn handle_name(&self) -> &str {
        &self.handle_name
    }
    
    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut SplitDiffViewConfig {
        &mut self.config
    }
    
    /// Get a reference to the configuration
    pub fn config(&self) -> &SplitDiffViewConfig {
        &self.config
    }
    
    /// Get a mutable reference to the state
    pub fn state_mut(&mut self) -> &mut SplitDiffViewState {
        &mut self.state
    }
    
    /// Get a reference to the state
    pub fn state(&self) -> &SplitDiffViewState {
        &self.state
    }
    
    /// Update the scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.state.scroll_offset = offset;
    }
    
    /// Update the fold unchanged setting
    pub fn set_fold_unchanged(&mut self, fold: bool) {
        self.state.fold_unchanged = fold;
    }
    
    /// Scroll up by the specified number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        self.state.scroll_offset = self.state.scroll_offset.saturating_sub(lines);
    }
    
    /// Scroll down by the specified number of lines
    pub fn scroll_down(&mut self, lines: usize) {
        self.state.scroll_offset = self.state.scroll_offset.saturating_add(lines);
    }
    
    /// Compute render-ready data for the split diff view (static method)
    /// Uses RenderParams struct for cleaner API
    pub fn compute_render_data_static(params: rendering::RenderParams<'_>) -> SplitDiffRenderData {
        rendering::compute_render_data_static(params)
    }
    
    /// Compute render-ready data for the split diff view (instance method)
    /// Creates RenderParams from instance fields and passed parameters
    pub fn compute_render_data(
        &mut self,
        source_lines: &[String],
        dest_lines: &[String],
        text_width: usize,
        gutter_width: usize,
        max_line_digits: usize,
        available_height: usize,
    ) -> SplitDiffRenderData {
        let params = rendering::RenderParams::new(
            &self.config,
            &mut self.state,
            source_lines,
            dest_lines,
            text_width,
            gutter_width,
            max_line_digits,
            available_height,
        );
        Self::compute_render_data_static(params)
    }

    /// Compute render-ready data for the split diff view with syntax highlighting
    /// Creates RenderParams from instance fields and passed parameters
    pub fn compute_render_data_with_syntax(
        &mut self,
        source_lines: &[String],
        dest_lines: &[String],
        text_width: usize,
        gutter_width: usize,
        max_line_digits: usize,
        available_height: usize,
        syntax_highlighter: Option<&SyntaxHighlighter>,
    ) -> SplitDiffRenderData {
        let params = rendering::RenderParams::with_syntax_highlighter(
            &self.config,
            &mut self.state,
            source_lines,
            dest_lines,
            text_width,
            gutter_width,
            max_line_digits,
            available_height,
            syntax_highlighter,
        );
        Self::compute_render_data_static(params)
    }
}
