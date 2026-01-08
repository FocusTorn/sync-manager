// Dashboard state management module

use std::sync::Arc;

/// Maximum output lines to keep in memory
#[allow(dead_code)]  // Will be used when output functionality is added
const MAX_OUTPUT_LINES: usize = 1000;

/// Dashboard state structure
#[derive(Debug, Clone)]
pub struct DashboardState {
    pub status_text: Arc<str>,
    pub output_lines: Vec<String>,
    pub output_scroll: usize,
    /// Auto-scroll enabled flag - when true, new lines automatically scroll to bottom
    pub auto_scroll_enabled: bool,
}

/// Sentinel value to indicate "scroll to bottom" - renderer will calculate actual position
pub const SCROLL_TO_BOTTOM: usize = usize::MAX;

impl DashboardState {
    /// Create a new dashboard state
    pub fn new() -> Self {
        Self {
            status_text: Arc::from("Ready"),
            output_lines: Vec::new(),
            output_scroll: 0,
            auto_scroll_enabled: true,
        }
    }
    
    /// Scroll output up - disables auto-scroll when user manually scrolls
    pub fn scroll_output_up(&mut self, amount: usize) {
        // User manually scrolled - disable auto-scroll
        self.auto_scroll_enabled = false;
        if self.output_scroll > 0 && self.output_scroll != SCROLL_TO_BOTTOM {
            self.output_scroll = self.output_scroll.saturating_sub(amount);
        }
    }
    
    /// Scroll output down - disables auto-scroll when user manually scrolls
    pub fn scroll_output_down(&mut self, amount: usize) {
        // User manually scrolled - disable auto-scroll
        self.auto_scroll_enabled = false;
        // Use a conservative estimate - actual max_scroll will be calculated during render
        let total_lines = self.output_lines.len();
        if total_lines > 0 {
            // Estimate max_scroll conservatively (assume at least 1 line visible)
            let estimated_max = total_lines.saturating_sub(1);
            let current_scroll = if self.output_scroll == SCROLL_TO_BOTTOM {
                0  // Start from 0 if we were at bottom
            } else {
                self.output_scroll
            };
            if current_scroll < estimated_max {
                self.output_scroll = (current_scroll + amount).min(estimated_max);
            }
        }
    }
    
    /// Scroll to bottom of output (called by renderer with correct visible_height)
    #[allow(dead_code)]  // Will be used when dashboard output is implemented
    pub fn scroll_to_bottom(&mut self, visible_height: usize) {
        if self.output_lines.is_empty() {
            self.output_scroll = 0;
            return;
        }
        let total_lines = self.output_lines.len();
        // Calculate maximum scroll position
        let max_scroll = if total_lines > visible_height {
            total_lines - visible_height
        } else {
            0
        };
        self.output_scroll = max_scroll;
    }
    
    /// Add a line to output, enforcing size limit
    /// If auto-scroll is enabled, marks scroll position for "scroll to bottom" during render
    #[allow(dead_code)]  // Will be used when output functionality is added
    pub fn add_output_line(&mut self, line: String) {
        self.output_lines.push(line);
        
        // Enforce size limit by removing oldest lines
        if self.output_lines.len() > MAX_OUTPUT_LINES {
            let remove_count = self.output_lines.len() - MAX_OUTPUT_LINES;
            self.output_lines.drain(0..remove_count);
            
            // Adjust scroll position if needed (but preserve SCROLL_TO_BOTTOM sentinel)
            if self.output_scroll != SCROLL_TO_BOTTOM {
                if self.output_scroll >= remove_count {
                    self.output_scroll -= remove_count;
                } else {
                    self.output_scroll = 0;
                }
            }
        }
        
        // If auto-scroll is enabled, mark for scrolling to bottom during render
        // The renderer will calculate the correct position with visible_height
        if self.auto_scroll_enabled {
            self.output_scroll = SCROLL_TO_BOTTOM;
        }
    }
    
    /// Set status text
    #[allow(dead_code)]  // Will be used when status updates are added
    pub fn set_status_text(&mut self, text: &str) {
        self.status_text = Arc::from(text);
    }
}
