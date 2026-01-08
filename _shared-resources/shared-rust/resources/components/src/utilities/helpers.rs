// Helper utilities for TUI components
use ratatui::style::{Color, Modifier, Style};

/// Convert hex color to ratatui Color
pub fn hex_color(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

/// Dimming context - tracks if modal is visible
pub struct DimmingContext {
    pub modal_visible: bool,
}

impl DimmingContext {
    pub fn new(modal_visible: bool) -> Self {
        Self { modal_visible }
    }

    /// Apply dimming to a color based on modal state
    pub fn dim_color(&self, color: Color) -> Color {
        if self.modal_visible {
            // Dim all colors to grey when modal is visible
            hex_color(0x444444)
        } else {
            color
        }
    }

    /// Apply dimming to a style based on modal state
    pub fn dim_style(&self, style: Style) -> Style {
        if self.modal_visible {
            style.fg(hex_color(0x444444))
        } else {
            style
        }
    }

    /// Get dimmed text color
    pub fn text_color(&self, is_active: bool) -> Color {
        if self.modal_visible {
            hex_color(0x444444)  // Dimmed when modal visible
        } else if is_active {
            hex_color(0xFFFFFF)  // White when focused
        } else {
            hex_color(0x777777)  // Grey when unfocused
        }
    }

    /// Get dimmed border color
    pub fn border_color(&self, is_active: bool) -> Color {
        if self.modal_visible {
            hex_color(0x222222)  // Dimmed when modal active
        } else if is_active {
            Color::White          // White when focused
        } else {
            hex_color(0x333333)  // Grey when unfocused
        }
    }

    /// Get dimmed selection style
    pub fn selection_style(&self, is_active: bool) -> Style {
        if self.modal_visible {
            Style::default()
                .bg(hex_color(0x0D0D0D))  // Nearly invisible highlight
                .fg(hex_color(0x444444))  // Dimmed grey text
        } else if is_active {
            Style::default()
                .bg(hex_color(0x1A2A2A))  // Dim cyan background
                .fg(Color::Cyan)           // Cyan text
        } else {
            Style::default()
                .bg(hex_color(0x151515))  // Very subtle grey background
                .fg(hex_color(0x777777))  // Grey text
        }
    }
}

/// Get text color based on state
pub fn get_text_color(is_active: bool, modal_visible: bool) -> Color {
    if modal_visible {
        hex_color(0x444444)  // Dimmed when modal visible
    } else if is_active {
        hex_color(0xFFFFFF)  // White when focused
    } else {
        hex_color(0x777777)  // Grey when unfocused
    }
}

/// Get border style based on state
pub fn get_border_style(is_active: bool, modal_visible: bool) -> (Style, ratatui::widgets::BorderType) {
    let border_style = if modal_visible {
        Style::default().fg(hex_color(0x222222))  // Dimmed when modal active
    } else if is_active {
        Style::default().fg(Color::White)          // White when focused
    } else {
        Style::default().fg(hex_color(0x333333))  // Grey when unfocused
    };
    
    let border_type = if is_active {
        ratatui::widgets::BorderType::Thick   // Thick border when focused
    } else {
        ratatui::widgets::BorderType::Plain   // Plain border when unfocused
    };
    
    (border_style, border_type)
}

/// Get selection style based on state
pub fn get_selection_style(is_active: bool) -> Style {
    if is_active {
        Style::default()
            .bg(hex_color(0x1A2A2A))  // Dim cyan background
            .fg(Color::Cyan)           // Cyan text
    } else {
        Style::default()
            .bg(hex_color(0x151515))  // Very subtle grey background
            .fg(hex_color(0x777777))  // Grey text
    }
}

/// Get selection style when modal is visible
pub fn get_selection_style_modal() -> Style {
    Style::default()
        .bg(hex_color(0x0D0D0D))  // Nearly invisible highlight
        .fg(hex_color(0x444444))  // Dimmed grey text
}

/// Accent color helper
pub fn accent_color() -> Style {
    Style::default().fg(Color::Cyan)
}

/// Bold accent color helper
pub fn bold_accent_color() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

/// Centered rectangle helper for popups
pub fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Wrap text to fit within max width
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let mut current_line = String::new();
        
        for word in words {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    
    lines
}
