// Tab Bar Component
// A flexible tab bar component with multiple styling and positioning options

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::core::{RectHandle, RectRegistry, AlignmentConfigData};
use crate::utilities::DimmingContext;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabBarStyle {
    /// Curved brackets around active tab: ╭─────╮
    Tab,
    /// Plain text with separators: ─ TAB ─
    Text,
    /// Square brackets around active tab: [ TAB ]
    Boxed,
    /// Static boxed style: all tabs in brackets [ TAB ]─[ TAB ]
    BoxStatic,
    /// Static text style: all tabs as plain text ─ TAB ─ TAB
    TextStatic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabBarAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub enum TabBarPosition {
    /// Attach to top or bottom of a bounding box
    TopOf(Rect),
    BottomOf(Rect),
    /// Attach to top or bottom of a bounding box by handle (HWND-like)
    TopOfHandle(RectHandle),
    BottomOfHandle(RectHandle),
    /// Direct coordinates (x1, x2, y)
    Coords { x1: u16, x2: u16, y: u16 },
}

#[derive(Debug, Clone)]
pub struct TabBarItem {
    pub name: String,
    pub active: bool,
    /// State for state-based coloring (for tab bars with type: state)
    pub state: Option<crate::core::TabState>,
}

/// Bounding box for a tab (for click detection)
#[derive(Debug, Clone, Copy)]
pub struct TabBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl TabBounds {
    /// Check if a coordinate (x, y) is within this tab's bounds
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub struct TabBar {
    pub items: Vec<TabBarItem>,
    pub style: TabBarStyle,
    pub alignment: TabBarAlignment,
    pub position: TabBarPosition,
    pub color: Color,
    /// State-based colors (for tab bars with type: state)
    pub state_colors: Option<crate::core::TabBarStateColors>,
}

impl TabBar {
    pub fn new(items: Vec<TabBarItem>, style: TabBarStyle, alignment: TabBarAlignment) -> Self {
        Self {
            items,
            style,
            alignment,
            position: TabBarPosition::Coords { x1: 0, x2: 0, y: 0 },
            color: Color::White,
            state_colors: None,
        }
    }

    pub fn with_position(mut self, position: TabBarPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Calculate the bounds of each tab based on the tab bar's current position and style
    /// Returns a vector of TabBounds for click detection
    /// Call this after determining the tab bar's area (for click handling)
    pub fn calculate_tab_bounds(&self, registry: Option<&RectRegistry>) -> Vec<TabBounds> {
        let area = self.calculate_area_with_registry(Rect::default(), registry);
        if area.width == 0 || area.height == 0 {
            return Vec::new();
        }

        let mut bounds = Vec::new();
        let mut current_x = area.x;
        let tab_y = area.y;

        // Calculate leading separator width
        let leading_width = match self.style {
            TabBarStyle::Tab => if self.items.first().map(|i| i.active).unwrap_or(false) { 2 } else { 3 },
            TabBarStyle::Text | TabBarStyle::TextStatic => 3,
            TabBarStyle::Boxed | TabBarStyle::BoxStatic => 2,
        };
        current_x += leading_width;

        for (idx, item) in self.items.iter().enumerate() {
            let tab_width = match self.style {
                TabBarStyle::Tab => {
                    if item.active {
                        item.name.len() as u16 + 4 // "╯ NAME ╰"
                    } else {
                        item.name.len() as u16
                    }
                }
                TabBarStyle::Boxed => {
                    if item.active {
                        item.name.len() as u16 + 4 // "[ NAME ]"
                    } else {
                        item.name.len() as u16
                    }
                }
                TabBarStyle::BoxStatic => item.name.len() as u16 + 4, // "[ NAME ]"
                TabBarStyle::Text | TabBarStyle::TextStatic => item.name.len() as u16,
            };

            bounds.push(TabBounds {
                x: current_x,
                y: tab_y,
                width: tab_width,
                height: 1,
            });

            current_x += tab_width;

            // Add separator width if not last tab
            if idx < self.items.len() - 1 {
                let sep_width = match self.style {
                    TabBarStyle::Tab | TabBarStyle::Boxed => if item.active { 2 } else { 3 },
                    TabBarStyle::Text => 3,
                    TabBarStyle::BoxStatic | TabBarStyle::TextStatic => if idx < self.items.len() - 1 { 1 } else { 0 },
                };
                current_x += sep_width;
            }
        }

        bounds
    }

    /// Get the index of the tab at the given coordinates (for click handling)
    /// Returns None if no tab was clicked
    pub fn get_tab_at(&self, x: u16, y: u16, registry: Option<&RectRegistry>) -> Option<usize> {
        let bounds = self.calculate_tab_bounds(registry);
        bounds
            .iter()
            .enumerate()
            .find(|(_, b)| b.contains(x, y))
            .map(|(idx, _)| idx)
    }

    pub fn render(&self, f: &mut Frame) {
        self.render_with_registry(f, None, None)
    }

    pub fn render_with_registry(&self, f: &mut Frame, mut registry: Option<&mut RectRegistry>, dimming: Option<&DimmingContext>) {
        self.render_with_registry_and_handle(f, registry.as_deref_mut(), None, dimming)
    }

    pub fn render_with_registry_and_handle(&self, f: &mut Frame, mut registry: Option<&mut RectRegistry>, handle_name: Option<&str>, dimming: Option<&DimmingContext>) {
        // Note: Tab style anchor box adjustment (x+1, height-1) is now handled in from_registry()
        // during prepare(), so the box is adjusted before rendering, allowing content to render
        // with the adjusted position and other elements to calculate relative positions correctly
        
        // Calculate area using the registry (box may have been adjusted for Tab style during prepare)
        let area = self.calculate_area_with_registry(f.area(), registry.as_deref());
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Render the decorative line above the tab bar (only for Tab style)
        // Tab bar text is at rect.y (on the border), top decorative line is at rect.y - 1 (one line above)
        if self.style == TabBarStyle::Tab {
            if let Some(active_tab) = self.items.iter().find(|item| item.active) {
                // Top line is one line above the tab bar text
                // Tab bar is at area.y (which is rect.y), so top line is at area.y - 1 = rect.y - 1
                let top_line_area = Rect {
                    x: area.x,
                    y: area.y.saturating_sub(1), // One line above tab bar text
                    width: area.width,
                    height: 1,
                };
                
                if top_line_area.y < f.area().height {
                    let top_line = self.build_top_line(area, active_tab, dimming);
                    let paragraph = Paragraph::new(top_line);
                    f.render_widget(paragraph, top_line_area);
                }
            }
        }

        // Use the estimated width, not the area width, to ensure all tabs are shown
        let estimated_width = self.estimate_width();
        let line = self.build_tab_line(estimated_width.max(area.width), dimming);
        let paragraph = Paragraph::new(line);
        f.render_widget(paragraph, area);
        
        // Register the tab bar with its handle name if provided
        if let (Some(registry), Some(handle_name)) = (registry.as_mut(), handle_name) {
            registry.register(Some(handle_name), area);
        }
    }
    
    fn build_top_line(&self, tab_area: Rect, _active_tab: &TabBarItem, dimming: Option<&DimmingContext>) -> Line<'static> {
        // Helper to get dimmed color
        let dim_color = |color: Color| -> Color {
            dimming.map(|d| d.dim_color(color)).unwrap_or(color)
        };
        let mut spans = Vec::new();
        
        // Find the position of the active tab within the tab bar
        let mut current_x = 0;
        let first_is_active = self.items.first().map(|item| item.active && self.style == TabBarStyle::Tab).unwrap_or(false);
        let leading_sep = if first_is_active {
            "──" // No space, connects directly to ╯
        } else {
            "── " // Space after for inactive tabs
        };
        current_x += leading_sep.chars().count() as u16;
        
        let mut active_tab_start = 0;
        let mut active_tab_width = 0;
        
        // Calculate where the active tab starts (relative to tab bar start)
        let mut prev_was_active = false;
        for (idx, item) in self.items.iter().enumerate() {
            if idx > 0 {
                // Skip separator if previous tab was active (we already added separator after it)
                if !prev_was_active {
                    let is_before_active = item.active && self.style == TabBarStyle::Tab;
                    let separator = if is_before_active {
                        " ─" // Space-dash, creates gap before ╯
                    } else {
                        " ─ " // Space before and after for inactive tabs
                    };
                    current_x += separator.chars().count() as u16;
                }
            }
            
            if item.active {
                active_tab_start = current_x;
                let text = format!("╯ {} ╰", item.name);
                active_tab_width = text.chars().count() as u16;
                break;
            } else {
                // Inactive tab: add its width
                current_x += item.name.chars().count() as u16;
                // Check if previous tab (at idx-1) was active
                if idx > 0 {
                    prev_was_active = self.items[idx - 1].active && self.style == TabBarStyle::Tab;
                    if prev_was_active {
                        // Previous tab was active, so we already added separator after it
                        // Reset for next iteration
                        prev_was_active = false;
                    }
                }
            }
        }
        
        // Build the top line: spaces before, ╭───╮ for active tab, spaces after
        // The line should align with the tab area
        let tab_area_start = tab_area.x;
        let active_tab_absolute_start = tab_area_start + active_tab_start;
        
        // Fill from start of tab area to start of active tab
        if active_tab_absolute_start > tab_area_start {
            let spaces_before = (active_tab_absolute_start - tab_area_start) as usize;
            if spaces_before > 0 {
                spans.push(Span::styled(" ".repeat(spaces_before), Style::default().fg(dim_color(Color::White))));
            }
        }
        
        // Add the top bracket line for the active tab: ╭─────╮
        // The width should match the tab width (minus the brackets)
        let inner_width = active_tab_width.saturating_sub(2); // Subtract ╯ and ╰
        let bracket_line = if inner_width > 0 {
            format!("╭{}╮", "─".repeat(inner_width as usize))
        } else {
            "╭╮".to_string()
        };
        spans.push(Span::styled(bracket_line, Style::default().fg(dim_color(Color::White))));
        
        // Fill the rest with spaces (if needed)
        let line_end = active_tab_absolute_start + active_tab_width;
        let tab_area_end = tab_area.x + tab_area.width;
        if line_end < tab_area_end {
            let spaces_after = (tab_area_end - line_end) as usize;
            if spaces_after > 0 {
                spans.push(Span::styled(" ".repeat(spaces_after), Style::default().fg(dim_color(Color::White))));
            }
        }
        
        Line::from(spans)
    }

    fn calculate_area_with_registry(&self, _frame_area: Rect, registry: Option<&RectRegistry>) -> Rect {
        match &self.position {
            TabBarPosition::TopOf(rect) => {
                // Calculate tab bar width
                let tab_bar_width = self.estimate_width();
                
                // The rect passed in is the bounding box area (the Block's area)
                // The border characters are at the edges: left at rect.x, right at rect.x + rect.width - 1
                // For TopOf, we want to align on the border line itself (at rect.y)
                
                // Calculate x position based on alignment
                let x = match self.alignment {
                    TabBarAlignment::Left => {
                        // Align to left, starting after the left border character
                        rect.x + 1
                    }
                    TabBarAlignment::Center => {
                        // Center of the border line (accounting for border characters)
                        // The border line spans from rect.x to rect.x + rect.width - 1
                        // Center is at: rect.x + (rect.width - 1) / 2
                        // But we want the center of the visible area (between borders)
                        // Visible area: from rect.x + 1 to rect.x + rect.width - 2
                        // Center: rect.x + 1 + (rect.width - 3) / 2
                        // Simplified: rect.x + (rect.width + 1) / 2 - 1
                        let border_line_center = rect.x + rect.width / 2;
                        let tab_bar_center = tab_bar_width / 2;
                        border_line_center.saturating_sub(tab_bar_center)
                    }
                    TabBarAlignment::Right => {
                        // Align to right, ending before the right border character
                        let total_width = self.estimate_width();
                        (rect.x + rect.width).saturating_sub(total_width + 1)
                    }
                };
                
                // Ensure x doesn't go before the left border + 1 (to leave space for border char)
                let x = x.max(rect.x + 1);
                
                // Render on the top border (rect.y)
                // For Tab style, add a row on top for decorative line, so align to bottom (rect.y + 1)
                // For other styles, align to top (rect.y)
                // Calculate available width from x to just before the right border
                let right_edge = rect.x + rect.width - 1; // Right border character position
                let available_width = right_edge.saturating_sub(x) + 1;
                let y = if self.style == TabBarStyle::Tab {
                    rect.y.saturating_add(1) // Bottom edge for Tab style (adds row on top)
                } else {
                    rect.y // Top edge for other styles
                };
                Rect {
                    x,
                    y,
                    width: tab_bar_width.min(available_width),
                    height: 1,
                }
            }
            TabBarPosition::BottomOf(rect) => {
                let width = rect.width;
                let x = match self.alignment {
                    TabBarAlignment::Left => rect.x + 1,
                    TabBarAlignment::Center => {
                        let total_width = self.estimate_width();
                        rect.x + (rect.width.saturating_sub(total_width)) / 2
                    }
                    TabBarAlignment::Right => {
                        let total_width = self.estimate_width();
                        rect.x + rect.width.saturating_sub(total_width) - 1
                    }
                };
                Rect {
                    x,
                    y: rect.y + rect.height - 1,
                    width: width.min(self.estimate_width()),
                    height: 1,
                }
            }
            TabBarPosition::TopOfHandle(handle) => {
                // Look up the rect from the registry
                if let Some(registry) = registry {
                    if let Some(metrics) = registry.get_metrics(*handle) {
                        let rect: Rect = metrics.into();
                        
                        // For Tab style, the anchor box has already been adjusted (y+1, height-1) in from_registry()
                        // So rect.y is already the adjusted position - attach directly at rect.y
                        // For Text/Boxed styles, use the container as-is (no adjustment was made)
                        
                        // Use the same logic as TopOf
                        let tab_bar_width = self.estimate_width();
                        let x = match self.alignment {
                            TabBarAlignment::Left => rect.x + 1,
                            TabBarAlignment::Center => {
                                let border_line_center = rect.x + rect.width / 2;
                                let tab_bar_center = tab_bar_width / 2;
                                border_line_center.saturating_sub(tab_bar_center)
                            }
                            TabBarAlignment::Right => {
                                let total_width = self.estimate_width();
                                (rect.x + rect.width).saturating_sub(total_width + 1)
                            }
                        };
                        let x = x.max(rect.x + 1);
                        let right_edge = rect.x + rect.width - 1;
                        let available_width = right_edge.saturating_sub(x) + 1;
                        // For Tab style, the anchor box is already adjusted (moved down by 1 row) in from_registry()
                        // So attach directly at rect.y (the adjusted position)
                        // For other styles, attach at rect.y as well (no adjustment was made)
                        let y = rect.y; // Attach at the container's top border (already adjusted if Tab style)
                        Rect {
                            x,
                            y,
                            width: tab_bar_width.min(available_width),
                            height: 1,
                        }
                    } else {
                        // Handle not found, return empty rect
                        Rect { x: 0, y: 0, width: 0, height: 0 }
                    }
                } else {
                    // No registry provided, return empty rect
                    Rect { x: 0, y: 0, width: 0, height: 0 }
                }
            }
            TabBarPosition::BottomOfHandle(handle) => {
                // Look up the rect from the registry
                if let Some(registry) = registry {
                    if let Some(metrics) = registry.get_metrics(*handle) {
                        let rect: Rect = metrics.into();
                        // Use the same logic as BottomOf
                        let width = rect.width;
                        let x = match self.alignment {
                            TabBarAlignment::Left => rect.x + 1,
                            TabBarAlignment::Center => {
                                let total_width = self.estimate_width();
                                rect.x + (rect.width.saturating_sub(total_width)) / 2
                            }
                            TabBarAlignment::Right => {
                                let total_width = self.estimate_width();
                                rect.x + rect.width.saturating_sub(total_width) - 1
                            }
                        };
                        Rect {
                            x,
                            y: rect.y + rect.height - 1,
                            width: width.min(self.estimate_width()),
                            height: 1,
                        }
                    } else {
                        // Handle not found, return empty rect
                        Rect { x: 0, y: 0, width: 0, height: 0 }
                    }
                } else {
                    // No registry provided, return empty rect
                    Rect { x: 0, y: 0, width: 0, height: 0 }
                }
            }
            TabBarPosition::Coords { x1, x2, y } => Rect {
                x: *x1,
                y: *y,
                width: x2.saturating_sub(*x1),
                height: 1,
            },
        }
    }

    pub fn estimate_width(&self) -> u16 {
        // Calculate based on actual tab text and dividers (using character count)
        // Leading separator depends on if first tab is active (only for Tab style)
        let first_is_active = self.items.first().map(|item| item.active && self.style == TabBarStyle::Tab).unwrap_or(false);
        let leading = match self.style {
            TabBarStyle::Tab => {
                if first_is_active {
                    "──" // No space, connects directly to ╯
                } else {
                    "── " // Space after for inactive tabs
                }
            }
            TabBarStyle::Text | TabBarStyle::Boxed | TabBarStyle::BoxStatic | TabBarStyle::TextStatic => {
                "── " // Text, Boxed, and static styles always have space after leading separator
            }
        };
        let mut width = leading.chars().count() as u16;
        
        let mut prev_was_active = false;
        for (idx, item) in self.items.iter().enumerate() {
            if idx > 0 {
                // Separator before each tab (except first)
                // Skip if previous tab was active (we already added separator after it) - only for Tab and Boxed styles
                // Static variants always add separators
                if !prev_was_active || self.style == TabBarStyle::BoxStatic || self.style == TabBarStyle::TextStatic {
                    let separator = match self.style {
                        TabBarStyle::Tab => {
                            // Check if separator is before active tab
                            let is_before_active = item.active;
                            if is_before_active {
                                " ─" // Space-dash, creates gap before ╯
                            } else {
                                " ─ " // Space before and after for inactive tabs
                            }
                        }
                        TabBarStyle::Boxed => {
                            // Check if separator is before active tab
                            let is_before_active = item.active;
                            if is_before_active {
                                " ─" // Space-dash, creates gap before [
                            } else {
                                " ─ " // Space before and after for inactive tabs
                            }
                        }
                        TabBarStyle::Text | TabBarStyle::TextStatic => {
                            " ─ " // Text and TextStatic styles always use consistent separators
                        }
                        TabBarStyle::BoxStatic => {
                            "─" // Just dash, connects to [ for static boxed style
                        }
                    };
                    width += separator.chars().count() as u16;
                }
            }
            
            // Tab text width (using character count)
            match self.style {
                TabBarStyle::Tab => {
                    if item.active {
                        // Active tab: "╯ BASELINES ╰"
                        let text = format!("╯ {} ╰", item.name);
                        width += text.chars().count() as u16;
                    } else {
                        // Inactive tab: just the name
                        width += item.name.chars().count() as u16;
                    }
                }
                TabBarStyle::Boxed => {
                    if item.active {
                        // Active tab: "[ BASELINES ]"
                        let text = format!("[ {} ]", item.name);
                        width += text.chars().count() as u16;
                    } else {
                        // Inactive tab: just the name
                        width += item.name.chars().count() as u16;
                    }
                }
                TabBarStyle::Text | TabBarStyle::TextStatic => {
                    // Plain text: just the name
                    width += item.name.chars().count() as u16;
                }
                TabBarStyle::BoxStatic => {
                    // Static boxed style: all tabs in brackets [ TAB ]
                    let text = format!("[ {} ]", item.name);
                    width += text.chars().count() as u16;
                }
            }
            
            // Separator after tab if there's a next tab
            // For Tab and Boxed: only if active
            // For static variants: always add separator
            let should_add_sep = match self.style {
                TabBarStyle::Tab | TabBarStyle::Boxed => {
                    item.active && idx < self.items.len() - 1
                }
                TabBarStyle::BoxStatic | TabBarStyle::TextStatic => {
                    idx < self.items.len() - 1 // Always add separator for static variants
                }
                _ => false,
            };
            
            if should_add_sep {
                match self.style {
                    TabBarStyle::Tab => {
                        width += "─ ".chars().count() as u16; // Dash-space, connects to ╰ then space before next tab
                    }
                    TabBarStyle::Boxed => {
                        width += "─ ".chars().count() as u16; // Dash-space, creates gap after ] before next tab
                    }
                    TabBarStyle::BoxStatic => {
                        width += "─".chars().count() as u16; // Just dash, connects to ] then next [
                    }
                    TabBarStyle::TextStatic => {
                        width += " ─ ".chars().count() as u16; // Space-dash-space for text static
                    }
                    _ => {}
                }
                if self.style == TabBarStyle::Tab || self.style == TabBarStyle::Boxed {
                    prev_was_active = true;
                }
            } else {
                prev_was_active = false;
            }
        }
        
        // Add trailing separator
        // Check if last tab is inactive to add space before trailing separator
        // Static variants always have space before trailing separator
        let last_is_active = match self.style {
            TabBarStyle::BoxStatic | TabBarStyle::TextStatic => false, // Static variants never have active state
            _ => self.items.last().map(|item| {
                item.active && (self.style == TabBarStyle::Tab || self.style == TabBarStyle::Boxed)
            }).unwrap_or(false),
        };
        let trailing_sep = if last_is_active {
            "──" // No space needed if last tab is active (it already has separator)
        } else {
            " ──" // Add space before trailing separator if last tab is inactive
        };
        width += trailing_sep.chars().count() as u16;
        
        width
    }

    pub fn build_tab_line(&self, max_width: u16, dimming: Option<&DimmingContext>) -> Line<'static> {
        let mut spans = Vec::new();
        let mut current_width = 0;

        // Helper to get dimmed color
        let dim_color = |color: Color| -> Color {
            dimming.map(|d| d.dim_color(color)).unwrap_or(color)
        };
        
        // Helper to get state color for a tab item
        let get_state_color = |item: &TabBarItem| -> Option<Color> {
            if let (Some(state), Some(state_colors)) = (item.state, &self.state_colors) {
                let color_str = match state {
                    crate::core::TabState::Active => state_colors.active.as_ref(),
                    crate::core::TabState::Negate => state_colors.negate.as_ref(),
                    crate::core::TabState::Disabled => state_colors.disabled.as_ref(),
                    crate::core::TabState::Default => None,
                };
                color_str.map(|s| parse_color(s))
            } else {
                None
            }
        };

        // Check if first tab is active to determine leading separator (only for Tab style)
        let first_is_active = self.items.first().map(|item| item.active && self.style == TabBarStyle::Tab).unwrap_or(false);
        
        // Start with leading separator
        let leading_sep = match self.style {
            TabBarStyle::Tab => {
                if first_is_active {
                    "──" // No space, connects directly to ╯
                } else {
                    "── " // Space after for inactive tabs
                }
            }
            TabBarStyle::Text | TabBarStyle::Boxed | TabBarStyle::BoxStatic | TabBarStyle::TextStatic => {
                "── " // Text, Boxed, and static styles always have space after leading separator
            }
        };
        spans.push(Span::styled(leading_sep, Style::default().fg(dim_color(Color::White))));
        current_width += leading_sep.chars().count() as u16;

        // Track if previous tab was active to skip separator before next tab (only for Tab and Boxed styles)
        // Static variants don't have active states, so always add separators
        let mut prev_was_active = false;
        
        // Add tabs with separators - ensure we show all tabs
        for (idx, item) in self.items.iter().enumerate() {
            if idx > 0 {
                // Add separator before each tab (except first)
                // Skip if previous tab was active (we already added separator after it) - only for Tab and Boxed styles
                // Static variants always add separators
                if !prev_was_active || self.style == TabBarStyle::BoxStatic || self.style == TabBarStyle::TextStatic {
                    let separator = match self.style {
                        TabBarStyle::Tab => {
                            // Check if this is the separator before the active tab
                            let is_before_active = item.active;
                            if is_before_active {
                                " ─" // Space-dash, creates gap before ╯
                            } else {
                                " ─ " // Space before and after for inactive tabs
                            }
                        }
                        TabBarStyle::Boxed => {
                            // Check if this is the separator before the active tab
                            let is_before_active = item.active;
                            if is_before_active {
                                " ─" // Space-dash, creates gap before [
                            } else {
                                " ─ " // Space before and after for inactive tabs
                            }
                        }
                        TabBarStyle::Text | TabBarStyle::TextStatic => {
                            " ─ " // Text and TextStatic styles always use consistent separators
                        }
                        TabBarStyle::BoxStatic => {
                            "─" // Just dash, connects to [ for static boxed style
                        }
                    };
                    let sep_width = separator.chars().count() as u16; // Use char count, not byte length
                    
                    if current_width + sep_width <= max_width {
                        spans.push(Span::styled(separator, Style::default().fg(dim_color(Color::White))));
                        current_width += sep_width;
                    } else {
                        // If we can't fit the separator, we can't fit the tab either
                        break;
                    }
                }
            }

            // Check if we can fit this tab before building it
            let tab_width = match self.style {
                TabBarStyle::Tab => {
                    if item.active {
                        // Active tab with curved brackets: ╯ BASELINES ╰
                        format!("╯ {} ╰", item.name).chars().count() as u16
                    } else {
                        // Inactive tab: plain text
                        item.name.chars().count() as u16
                    }
                }
                TabBarStyle::Boxed => {
                    if item.active {
                        // Active tab with square brackets: [ BASELINES ]
                        format!("[ {} ]", item.name).chars().count() as u16
                    } else {
                        // Inactive tab: plain text
                        item.name.chars().count() as u16
                    }
                }
                TabBarStyle::Text | TabBarStyle::TextStatic => {
                    // Plain text style
                    item.name.chars().count() as u16
                }
                TabBarStyle::BoxStatic => {
                    // Static boxed style: all tabs in brackets [ TAB ]
                    format!("[ {} ]", item.name).chars().count() as u16
                }
            };

            if current_width + tab_width > max_width {
                break; // Can't fit this tab
            }

            // Render tab text - for active Tab and Boxed styles, split into spans to color only the name
            // Use state color if available, otherwise use default color logic
            let state_color = get_state_color(item);
            let text_color = state_color.unwrap_or_else(|| {
                if item.active && (self.style == TabBarStyle::Tab || self.style == TabBarStyle::Boxed || self.style == TabBarStyle::Text) {
                    self.color
                } else {
                    Color::White
                }
            });
            
            match self.style {
                TabBarStyle::Tab => {
                    if item.active {
                        // Active tab: split into ╯ (white), name (colored), ╰ (white)
                        spans.push(Span::styled("╯ ", Style::default().fg(dim_color(Color::White))));
                        spans.push(Span::styled(
                            item.name.clone(),
                            Style::default()
                                .fg(dim_color(text_color))
                                .add_modifier(Modifier::BOLD)
                        ));
                        spans.push(Span::styled(" ╰", Style::default().fg(dim_color(Color::White))));
                    } else {
                        // Inactive tab: use state color if available, otherwise white
                        spans.push(Span::styled(item.name.clone(), Style::default().fg(dim_color(text_color))));
                    }
                }
                TabBarStyle::Boxed => {
                    if item.active {
                        // Active tab: split into [ (white), name (colored), ] (white)
                        spans.push(Span::styled("[ ", Style::default().fg(dim_color(Color::White))));
                        spans.push(Span::styled(
                            item.name.clone(),
                            Style::default()
                                .fg(dim_color(text_color))
                                .add_modifier(Modifier::BOLD)
                        ));
                        spans.push(Span::styled(" ]", Style::default().fg(dim_color(Color::White))));
                    } else {
                        // Inactive tab: use state color if available, otherwise white
                        spans.push(Span::styled(item.name.clone(), Style::default().fg(dim_color(text_color))));
                    }
                }
                TabBarStyle::Text => {
                    // Text style: use state color if available, otherwise color only if active
                    let style = if state_color.is_some() || item.active {
                        Style::default()
                            .fg(dim_color(text_color))
                            .add_modifier(if item.active { Modifier::BOLD } else { Modifier::empty() })
                    } else {
                        Style::default().fg(dim_color(Color::White))
                    };
                    spans.push(Span::styled(item.name.clone(), style));
                }
                TabBarStyle::BoxStatic => {
                    // Static boxed style: use state color if available, otherwise white
                    spans.push(Span::styled("[ ", Style::default().fg(dim_color(Color::White))));
                    spans.push(Span::styled(item.name.clone(), Style::default().fg(dim_color(text_color))));
                    spans.push(Span::styled(" ]", Style::default().fg(dim_color(Color::White))));
                }
                TabBarStyle::TextStatic => {
                    // Static text style: use state color if available, otherwise white
                    spans.push(Span::styled(item.name.clone(), Style::default().fg(dim_color(text_color))));
                }
            }
            current_width += tab_width;
            
            // Add separator after tab if there's a next tab
            // For Tab and Boxed: only if active
            // For static variants: always add separator
            let should_add_sep = match self.style {
                TabBarStyle::Tab | TabBarStyle::Boxed => {
                    item.active && idx < self.items.len() - 1
                }
                TabBarStyle::BoxStatic | TabBarStyle::TextStatic => {
                    idx < self.items.len() - 1 // Always add separator for static variants
                }
                _ => false,
            };
            
            if should_add_sep {
                let next_sep = match self.style {
                    TabBarStyle::Tab => "─ ", // Dash-space, connects to ╰ then space before next tab
                    TabBarStyle::Boxed => "─ ", // Dash-space, creates gap after ] before next tab
                    TabBarStyle::BoxStatic => "─", // Just dash, connects to ] then next [
                    TabBarStyle::TextStatic => " ─ ", // Space-dash-space for text static
                    _ => "",
                };
                let next_sep_width = next_sep.chars().count() as u16;
                if current_width + next_sep_width <= max_width {
                    spans.push(Span::styled(next_sep, Style::default().fg(dim_color(Color::White))));
                    current_width += next_sep_width;
                    if self.style == TabBarStyle::Tab || self.style == TabBarStyle::Boxed {
                        prev_was_active = true; // Mark that we added separator after active tab
                    }
                } else {
                    prev_was_active = false;
                }
            } else {
                prev_was_active = false;
            }
        }

        // Add trailing separator if there's space
        // Check if last tab is inactive to add space before trailing separator
        let last_is_active = self.items.last().map(|item| {
            item.active && (self.style == TabBarStyle::Tab || self.style == TabBarStyle::Boxed)
        }).unwrap_or(false);
        let trailing_sep = if last_is_active {
            "──" // No space needed if last tab is active (it already has separator)
        } else {
            " ──" // Add space before trailing separator if last tab is inactive
        };
        let trailing_sep_width = trailing_sep.chars().count() as u16;
        if max_width >= current_width + trailing_sep_width {
            spans.push(Span::styled(trailing_sep, Style::default().fg(dim_color(Color::White))));
        }

        Line::from(spans)
    }

    /// Prepare tab bar from registry state - creates TabBar but does NOT render
    /// Returns (TabBar, anchor_handle, tab_bar_state) if successful
    pub fn from_registry(
        registry: &mut RectRegistry,
        tab_bar_handle: RectHandle,
        tab_style_override: Option<TabBarStyle>,
    ) -> Option<(Self, RectHandle, crate::core::TabBarState)> {
        
        // Clone state to avoid borrow checker issues
        let tab_bar_state = registry.get_tab_bar_state(tab_bar_handle)?.clone();
        
        // Parse configuration from stored state (use override if provided)
        let tab_style = tab_style_override.unwrap_or_else(|| TabBarStyle::from_str(&tab_bar_state.config.style));
        let parsed_alignment = parse_alignment_from_config(&tab_bar_state.config.alignment);
        let tab_color = parse_color(&tab_bar_state.config.color);
        
        // Get anchor handle
        let anchor_handle = registry.get_handle(&tab_bar_state.config.anchor)?;
        
        // Get anchor metrics for positioning
        let anchor_metrics = registry.get_metrics(anchor_handle)?;
        let anchor_rect: Rect = anchor_metrics.into();
        
        // Get active tab index
        let active_tab_index = tab_bar_state.active_tab_index;
        
        // Create tab items from registry state
        // Include state if tab bar type is "state"
        let include_state = tab_bar_state.config.tab_bar_type.as_ref()
            .map(|t| t == "state")
            .unwrap_or(false);
        
        let tab_items: Vec<TabBarItem> = tab_bar_state.tab_configs
            .iter()
            .enumerate()
            .map(|(idx, tab_config)| TabBarItem {
                name: tab_config.name.clone(),
                active: idx == active_tab_index && tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic,
                state: if include_state { Some(tab_config.state) } else { None },
            })
            .collect();
        
        // Create TabBarPosition based on parsed alignment
        // For Tab style with handle-based positioning, adjust the anchor box: y+1 and height-1
        // This adjustment happens before creating the position so other elements can calculate relative positions correctly
        let tab_position = if parsed_alignment.offset_x == 0 && parsed_alignment.offset_y == 0 {
            // Handle-based positioning (TopOfHandle or BottomOfHandle) - adjust anchor box for Tab style
            if tab_style == TabBarStyle::Tab {
                if let Some(metrics) = registry.get_metrics(anchor_handle) {
                    let mut updated_metrics = metrics;
                    updated_metrics.y = updated_metrics.y.saturating_add(1); // Move box down 1 line
                    updated_metrics.height = updated_metrics.height.saturating_sub(1).max(1); // Reduce height by 1
                    registry.update(anchor_handle, updated_metrics.into());
                }
            }
            // No offsets: use handle-based positioning
            match parsed_alignment.vertical {
                VerticalPosition::Top => TabBarPosition::TopOfHandle(anchor_handle),
                VerticalPosition::Bottom => TabBarPosition::BottomOfHandle(anchor_handle),
            }
        } else {
            // Offsets specified: calculate coordinates from anchor rect
            let estimated_tab_width: u16 = tab_bar_state.tab_configs.iter()
                .map(|t| t.name.len() as u16 + 4)
                .sum::<u16>() + 10;
            
            let y = match parsed_alignment.vertical {
                VerticalPosition::Top => anchor_rect.y.saturating_add(parsed_alignment.offset_y),
                VerticalPosition::Bottom => {
                    let bottom_y = anchor_rect.y + anchor_rect.height - 1;
                    if parsed_alignment.offset_y <= bottom_y {
                        bottom_y.saturating_sub(parsed_alignment.offset_y)
                    } else {
                        bottom_y
                    }
                }
            };
            
            let (x1, x2) = match parsed_alignment.horizontal {
                TabBarAlignment::Left => {
                    let x1 = anchor_rect.x + 1 + parsed_alignment.offset_x;
                    let x2 = (x1 + estimated_tab_width).min(anchor_rect.x + anchor_rect.width - 1);
                    (x1, x2)
                }
                TabBarAlignment::Center => {
                    let center_x = anchor_rect.x + anchor_rect.width / 2;
                    let half_width = estimated_tab_width / 2;
                    let x1 = center_x.saturating_sub(half_width).saturating_add(parsed_alignment.offset_x);
                    let x2 = (x1 + estimated_tab_width).min(anchor_rect.x + anchor_rect.width - 1);
                    (x1, x2)
                }
                TabBarAlignment::Right => {
                    let x2 = if parsed_alignment.offset_x <= (anchor_rect.x + anchor_rect.width - 1) {
                        (anchor_rect.x + anchor_rect.width - 1).saturating_sub(1).saturating_sub(parsed_alignment.offset_x)
                    } else {
                        anchor_rect.x + anchor_rect.width - 2
                    };
                    let x1 = x2.saturating_sub(estimated_tab_width).max(anchor_rect.x + 1);
                    (x1, x2)
                }
            };
            
            TabBarPosition::Coords { x1, x2, y }
        };
        
        // Create tab bar with position and horizontal alignment
        let mut tab_bar = TabBar::new(tab_items, tab_style, parsed_alignment.horizontal)
            .with_color(tab_color)
            .with_position(tab_position);
        
        // Set state colors if tab bar type is "state"
        tab_bar.state_colors = tab_bar_state.config.state_colors.clone();
        
        Some((tab_bar, anchor_handle, tab_bar_state))
    }

    /// Render tab bar with registry and handle name
    pub fn render_with_state(
        &self,
        f: &mut Frame,
        registry: &mut RectRegistry,
        tab_bar_state: &crate::core::TabBarState,
        dimming: Option<&DimmingContext>,
    ) {
        self.render_with_registry_and_handle(f, Some(registry), Some(&tab_bar_state.config.hwnd), dimming);
    }

    /// Initialize tab bar in registry from configuration
    /// This creates the handle, converts tab configs, and stores the state
    pub fn initialize_in_registry(
        registry: &mut RectRegistry,
        handle_name: &str,
        config: &crate::core::TabBarConfigData,
        tab_configs: Vec<crate::core::TabConfigData>,
    ) -> RectHandle {
        use crate::core::TabBarState;
        
        // Get or create handle (register with empty rect first, will be updated on render)
        let handle = registry.register(Some(handle_name), Rect { x: 0, y: 0, width: 0, height: 0 });
        
        // Extract tab names
        let tab_names: Vec<String> = tab_configs.iter().map(|t| t.name.clone()).collect();
        
        // Find initial active tab index
        let initial_active_tab_index = tab_configs.iter()
            .position(|t| t.active)
            .unwrap_or(0);
        
        // Create and store tab bar state with all configuration
        let state = TabBarState {
            active_tab_index: initial_active_tab_index,
            tab_count: tab_configs.len(),
            tab_names,
            tab_configs,
            config: config.clone(),
            last_navigation_time: None,
        };
        
        registry.set_tab_bar_state(handle, state);
        handle
    }
}

// Parsing helpers for tab bar configuration

/// Parse tab style from string
impl TabBarStyle {
    pub fn from_str(style: &str) -> Self {
        match style.to_lowercase().as_str() {
            "tabbed" | "tab" => TabBarStyle::Tab,
            "boxed" => TabBarStyle::Boxed,
            "text" => TabBarStyle::Text,
            "box_static" | "boxstatic" => TabBarStyle::BoxStatic,
            "text_static" | "textstatic" => TabBarStyle::TextStatic,
            _ => TabBarStyle::Tab, // Default
        }
    }
}

/// Parse color from string
pub fn parse_color(color: &str) -> Color {
    match color.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        // Dark colors using RGB values (ratatui doesn't have Dark* variants)
        "dark_red" | "darkred" => Color::Rgb(139, 0, 0),
        "dark_green" | "darkgreen" => Color::Rgb(0, 100, 0),
        "dark_yellow" | "darkyellow" => Color::Rgb(184, 134, 11),
        "dark_blue" | "darkblue" => Color::Rgb(0, 0, 139),
        "dark_magenta" | "darkmagenta" => Color::Rgb(139, 0, 139),
        "dark_cyan" | "darkcyan" => Color::Rgb(0, 139, 139),
        _ => Color::Cyan, // Default
    }
}

/// Vertical position for tab bar alignment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalPosition {
    Top,
    Bottom,
}

/// Parsed alignment configuration
#[derive(Debug, Clone, Copy)]
pub struct ParsedAlignment {
    pub vertical: VerticalPosition,
    pub horizontal: TabBarAlignment,
    pub offset_x: u16,
    pub offset_y: u16,
}

/// Parse alignment configuration from AlignmentConfigData
pub fn parse_alignment_from_config(alignment: &AlignmentConfigData) -> ParsedAlignment {
    let vertical = match alignment.vertical.to_lowercase().as_str() {
        "top" => VerticalPosition::Top,
        "bottom" => VerticalPosition::Bottom,
        _ => VerticalPosition::Top,
    };
    
    let horizontal = match alignment.horizontal.to_lowercase().as_str() {
        "left" => TabBarAlignment::Left,
        "center" => TabBarAlignment::Center,
        "right" => TabBarAlignment::Right,
        _ => TabBarAlignment::Center,
    };
    
    ParsedAlignment {
        vertical,
        horizontal,
        offset_x: alignment.offset_x,
        offset_y: alignment.offset_y,
    }
}

