// Popup/Modal component for confirmations and inputs
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};
use crate::utilities::{centered_rect, hex_color, wrap_text};

#[derive(Debug, Clone)]
pub enum PopupType {
    Confirm {
        title: String,
        message: String,
        selected: usize,  // 0 = Yes, 1 = No
    },
    Input {
        title: String,
        prompt: String,
        input: String,
        cursor_pos: usize,
    },
    Error {
        title: String,
        message: String,
    },
    Info {
        title: String,
        message: String,
    },
    Warning {
        title: String,
        message: String,
    },
}

pub struct Popup {
    pub popup_type: PopupType,
    pub visible: bool,
}

impl Popup {
    pub fn new(popup_type: PopupType) -> Self {
        Self {
            popup_type,
            visible: true,
        }
    }

    pub fn confirm(title: String, message: String) -> Self {
        Self::new(PopupType::Confirm {
            title,
            message,
            selected: 1, // Default to No
        })
    }

    pub fn input(title: String, prompt: String, initial: String) -> Self {
        Self::new(PopupType::Input {
            title,
            prompt,
            input: initial,
            cursor_pos: 0,
        })
    }

    pub fn error(title: String, message: String) -> Self {
        Self::new(PopupType::Error { title, message })
    }

    pub fn info(title: String, message: String) -> Self {
        Self::new(PopupType::Info { title, message })
    }

    pub fn warning(title: String, message: String) -> Self {
        Self::new(PopupType::Warning { title, message })
    }
}

/// Render popup with proper dimming
/// Everything behind the popup should be dimmed to very dim grey
pub fn render_popup(f: &mut Frame, area: Rect, popup: &Popup) {
    if !popup.visible {
        return;
    }

    // Dim the entire background to very dim grey
    // This creates the dimming effect behind the popup
    let dim_block = Paragraph::new("")
        .style(Style::default().bg(hex_color(0x0A0A0A))); // Very dim grey background
    f.render_widget(dim_block, area);

    match &popup.popup_type {
        PopupType::Confirm { title, message, selected } => {
            render_confirm_popup(f, area, title, message, *selected);
        }
        PopupType::Input { title, prompt, input, cursor_pos } => {
            render_input_popup(f, area, title, prompt, input, *cursor_pos);
        }
        PopupType::Error { title, message } => {
            render_error_popup(f, area, title, message);
        }
        PopupType::Info { title, message } => {
            render_info_popup(f, area, title, message);
        }
        PopupType::Warning { title, message } => {
            render_warning_popup(f, area, title, message);
        }
    }
}

fn render_confirm_popup(f: &mut Frame, area: Rect, title: &str, message: &str, selected: usize) {
    // Wrap message text
    let max_text_width = 50;
    let wrapped_lines = wrap_text(message, max_text_width);
    
    // Calculate popup dimensions
    let max_line_len = wrapped_lines.iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(title.len())
        .max(title.len())
        .max(30);
    
    let popup_width = (max_line_len as u16 + 8)
        .max(40)
        .min((area.width as f32 * 0.60) as u16)
        .min(area.width.saturating_sub(4)); // Ensure at least 2 chars margin on each side
    
    let popup_height = (wrapped_lines.len() as u16 + 7)
        .min(area.height.saturating_sub(4)); // Ensure at least 2 lines margin top/bottom
    
    // Calculate centered position manually to ensure it fits
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    
    // Ensure popup doesn't go outside bounds
    let popup_area = Rect {
        x: popup_x.max(area.x).min(area.x + area.width.saturating_sub(popup_width)),
        y: popup_y.max(area.y).min(area.y + area.height.saturating_sub(popup_height)),
        width: popup_width.min(area.width.saturating_sub(popup_x.max(area.x) - area.x)),
        height: popup_height.min(area.height.saturating_sub(popup_y.max(area.y) - area.y)),
    };
    
    // Use the actual popup area width for rendering (not the calculated width)
    let actual_width = popup_area.width as usize;
    
    // Clear popup area
    f.render_widget(Clear, popup_area);
    
    // Build popup content with double-line box
    let mut popup_lines = Vec::new();
    
    // Top border - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┏{}┓", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Title line - use actual width
    let title_padding = (actual_width - 2).saturating_sub(title.len());
    let title_left_pad = title_padding / 2;
    let title_right_pad = title_padding - title_left_pad;
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}{}{}┃", " ".repeat(title_left_pad), title, " ".repeat(title_right_pad)),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    
    // Empty line - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Content lines (centered) - use actual width
    for line in &wrapped_lines {
        let padding = (actual_width - 2).saturating_sub(line.len());
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;
        let centered = format!("┃{}{}{}┃", " ".repeat(left_pad), line, " ".repeat(right_pad));
        popup_lines.push(Line::from(Span::styled(centered, Style::default().fg(Color::White))));
    }
    
    // Empty line - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Button line: Yes / No - use actual width
    let yes_style = if selected == 0 {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(hex_color(0x777777))
    };
    
    let no_style = if selected == 1 {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(hex_color(0x777777))
    };
    
    let yes_text = "Yes";
    let no_text = "No";
    let buttons = format!("{}  {}", yes_text, no_text);
    
    let button_padding = (actual_width - 2).saturating_sub(buttons.len());
    let left_pad = button_padding / 2;
    let right_pad = button_padding - left_pad;
    
    let mut button_spans = vec![Span::styled("┃", Style::default().fg(Color::White))];
    button_spans.push(Span::raw(" ".repeat(left_pad)));
    button_spans.push(Span::styled(yes_text, yes_style));
    button_spans.push(Span::raw("  "));
    button_spans.push(Span::styled(no_text, no_style));
    button_spans.push(Span::raw(" ".repeat(right_pad)));
    button_spans.push(Span::styled("┃", Style::default().fg(Color::White)));
    
    popup_lines.push(Line::from(button_spans));
    
    // Empty line - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Bottom border - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┗{}┛", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Ensure popup area height matches the number of lines
    let actual_height = popup_lines.len() as u16;
    let final_popup_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width,
        height: actual_height.min(popup_area.height), // Use actual line count, but don't exceed available space
    };
    
    let popup_widget = Paragraph::new(popup_lines)
        .style(Style::default().bg(hex_color(0x141420))); // Panel background color
    
    f.render_widget(popup_widget, final_popup_area);
}

fn render_input_popup(f: &mut Frame, area: Rect, title: &str, prompt: &str, input: &str, cursor_pos: usize) {
    // Calculate popup dimensions
    let max_line_len = prompt.len().max(title.len()).max(30);
    let popup_width = (max_line_len as u16 + 8)
        .max(40)
        .min((area.width as f32 * 0.60) as u16)
        .min(area.width - 4);
    
    let popup_height = 7u16;
    
    // Center the popup
    let popup_area = centered_rect(
        ((popup_width as f32 / area.width as f32) * 100.0) as u16,
        ((popup_height as f32 / area.height as f32) * 100.0) as u16,
        area,
    );
    
    // Use the actual popup area width for rendering
    let actual_width = popup_area.width as usize;
    
    // Clear popup area
    f.render_widget(Clear, popup_area);
    
    // Build popup content
    let mut popup_lines = Vec::new();
    
    // Top border - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┏{}┓", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Title line - use actual width
    let title_padding = (actual_width - 2).saturating_sub(title.len());
    let title_left_pad = title_padding / 2;
    let title_right_pad = title_padding - title_left_pad;
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}{}{}┃", " ".repeat(title_left_pad), title, " ".repeat(title_right_pad)),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    
    // Empty line - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Prompt line - use actual width
    let prompt_padding = (actual_width - 2).saturating_sub(prompt.len());
    let prompt_left_pad = prompt_padding / 2;
    let prompt_right_pad = prompt_padding - prompt_left_pad;
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}{}{}┃", " ".repeat(prompt_left_pad), prompt, " ".repeat(prompt_right_pad)),
        Style::default().fg(Color::White),
    )));
    
    // Input field with cursor - use actual width
    let cursor = cursor_pos.min(input.len());
    let (head, tail) = input.split_at(cursor);
    let input_display = format!("{}{}{}", head, "█", tail);
    let input_padding = (actual_width - 2).saturating_sub(input_display.len());
    let input_left_pad = input_padding / 2;
    let input_right_pad = input_padding - input_left_pad;
    
    let mut input_spans = vec![Span::styled("┃", Style::default().fg(Color::White))];
    input_spans.push(Span::raw(" ".repeat(input_left_pad)));
    input_spans.push(Span::styled(head, Style::default().fg(Color::White)));
    input_spans.push(Span::styled("█", Style::default().fg(Color::Yellow))); // Yellow cursor for input
    input_spans.push(Span::styled(tail, Style::default().fg(Color::White)));
    input_spans.push(Span::raw(" ".repeat(input_right_pad)));
    input_spans.push(Span::styled("┃", Style::default().fg(Color::White)));
    
    popup_lines.push(Line::from(input_spans));
    
    // Empty line - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Bottom border - use actual width
    popup_lines.push(Line::from(Span::styled(
        format!("┗{}┛", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::White),
    )));
    
    // Ensure popup area height matches the number of lines
    let actual_height = popup_lines.len() as u16;
    let final_popup_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width,
        height: actual_height.min(popup_area.height),
    };
    
    let popup_widget = Paragraph::new(popup_lines)
        .style(Style::default().bg(hex_color(0x141420)));
    
    f.render_widget(popup_widget, final_popup_area);
}

fn render_error_popup(f: &mut Frame, area: Rect, title: &str, message: &str) {
    // Similar to confirm but with error styling
    let wrapped_lines = wrap_text(message, 50);
    let max_line_len = wrapped_lines.iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(title.len())
        .max(title.len())
        .max(30);
    
    let popup_width = (max_line_len as u16 + 8)
        .max(40)
        .min((area.width as f32 * 0.60) as u16)
        .min(area.width - 4);
    
    let popup_height = (wrapped_lines.len() as u16 + 5)
        .min(area.height - 4);
    
    let popup_area = centered_rect(
        ((popup_width as f32 / area.width as f32) * 100.0) as u16,
        ((popup_height as f32 / area.height as f32) * 100.0) as u16,
        area,
    );
    
    // Use the actual popup area width for rendering
    let actual_width = popup_area.width as usize;
    
    f.render_widget(Clear, popup_area);
    
    // Build popup similar to confirm but with error icon
    let mut popup_lines = Vec::new();
    
    popup_lines.push(Line::from(Span::styled(
        format!("┏{}┓", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Red),
    )));
    
    let title_padding = (actual_width - 2).saturating_sub(title.len());
    let title_left_pad = title_padding / 2;
    let title_right_pad = title_padding - title_left_pad;
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}{}{}┃", " ".repeat(title_left_pad), title, " ".repeat(title_right_pad)),
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )));
    
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Red),
    )));
    
    for line in &wrapped_lines {
        let padding = (actual_width - 2).saturating_sub(line.len());
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;
        // Border characters should be red, text should be white
        let spans = vec![
            Span::styled("┃", Style::default().fg(Color::Red)),
            Span::raw(" ".repeat(left_pad)),
            Span::styled(line.clone(), Style::default().fg(Color::White)),
            Span::raw(" ".repeat(right_pad)),
            Span::styled("┃", Style::default().fg(Color::Red)),
        ];
        popup_lines.push(Line::from(spans));
    }
    
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Red),
    )));
    
    popup_lines.push(Line::from(Span::styled(
        format!("┗{}┛", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Red),
    )));
    
    // Ensure popup area height matches the number of lines
    let actual_height = popup_lines.len() as u16;
    let final_popup_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width,
        height: actual_height.min(popup_area.height),
    };
    
    let popup_widget = Paragraph::new(popup_lines)
        .style(Style::default().bg(hex_color(0x141420)));
    
    f.render_widget(popup_widget, final_popup_area);
}

fn render_info_popup(f: &mut Frame, area: Rect, title: &str, message: &str) {
    // Similar to confirm but with info styling
    let wrapped_lines = wrap_text(message, 50);
    let max_line_len = wrapped_lines.iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(title.len())
        .max(title.len())
        .max(30);
    
    let popup_width = (max_line_len as u16 + 8)
        .max(40)
        .min((area.width as f32 * 0.60) as u16)
        .min(area.width - 4);
    
    let popup_height = (wrapped_lines.len() as u16 + 5)
        .min(area.height - 4);
    
    let popup_area = centered_rect(
        ((popup_width as f32 / area.width as f32) * 100.0) as u16,
        ((popup_height as f32 / area.height as f32) * 100.0) as u16,
        area,
    );
    
    // Use the actual popup area width for rendering
    let actual_width = popup_area.width as usize;
    
    f.render_widget(Clear, popup_area);
    
    // Build popup similar to confirm but with info icon
    let mut popup_lines = Vec::new();
    
    popup_lines.push(Line::from(Span::styled(
        format!("┏{}┓", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Cyan),
    )));
    
    let title_padding = (actual_width - 2).saturating_sub(title.len());
    let title_left_pad = title_padding / 2;
    let title_right_pad = title_padding - title_left_pad;
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}{}{}┃", " ".repeat(title_left_pad), title, " ".repeat(title_right_pad)),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Cyan),
    )));
    
    for line in &wrapped_lines {
        let padding = (actual_width - 2).saturating_sub(line.len());
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;
        // Border characters should be cyan, text should be white
        let spans = vec![
            Span::styled("┃", Style::default().fg(Color::Cyan)),
            Span::raw(" ".repeat(left_pad)),
            Span::styled(line.clone(), Style::default().fg(Color::White)),
            Span::raw(" ".repeat(right_pad)),
            Span::styled("┃", Style::default().fg(Color::Cyan)),
        ];
        popup_lines.push(Line::from(spans));
    }
    
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Cyan),
    )));
    
    popup_lines.push(Line::from(Span::styled(
        format!("┗{}┛", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Cyan),
    )));
    
    // Ensure popup area height matches the number of lines
    let actual_height = popup_lines.len() as u16;
    let final_popup_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width,
        height: actual_height.min(popup_area.height),
    };
    
    let popup_widget = Paragraph::new(popup_lines)
        .style(Style::default().bg(hex_color(0x141420)));
    
    f.render_widget(popup_widget, final_popup_area);
}


fn render_warning_popup(f: &mut Frame, area: Rect, title: &str, message: &str) {
    // Similar to info but with warning styling (yellow/orange)
    let wrapped_lines = wrap_text(message, 50);
    let max_line_len = wrapped_lines.iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(title.len())
        .max(title.len())
        .max(30);
    
    let popup_width = (max_line_len as u16 + 8)
        .max(40)
        .min((area.width as f32 * 0.60) as u16)
        .min(area.width - 4);
    
    let popup_height = (wrapped_lines.len() as u16 + 5)
        .min(area.height - 4);
    
    let popup_area = centered_rect(
        ((popup_width as f32 / area.width as f32) * 100.0) as u16,
        ((popup_height as f32 / area.height as f32) * 100.0) as u16,
        area,
    );
    
    // Use the actual popup area width for rendering
    let actual_width = popup_area.width as usize;
    
    f.render_widget(Clear, popup_area);
    
    // Build popup with warning styling (yellow/orange)
    let mut popup_lines = Vec::new();
    
    popup_lines.push(Line::from(Span::styled(
        format!("┏{}┓", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Yellow),
    )));
    
    let title_padding = (actual_width - 2).saturating_sub(title.len());
    let title_left_pad = title_padding / 2;
    let title_right_pad = title_padding - title_left_pad;
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}{}{}┃", " ".repeat(title_left_pad), title, " ".repeat(title_right_pad)),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Yellow),
    )));
    
    for line in &wrapped_lines {
        let padding = (actual_width - 2).saturating_sub(line.len());
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;
        // Border characters should be yellow, text should be white
        let spans = vec![
            Span::styled("┃", Style::default().fg(Color::Yellow)),
            Span::raw(" ".repeat(left_pad)),
            Span::styled(line.clone(), Style::default().fg(Color::White)),
            Span::raw(" ".repeat(right_pad)),
            Span::styled("┃", Style::default().fg(Color::Yellow)),
        ];
        popup_lines.push(Line::from(spans));
    }
    
    popup_lines.push(Line::from(Span::styled(
        format!("┃{}┃", " ".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Yellow),
    )));
    
    popup_lines.push(Line::from(Span::styled(
        format!("┗{}┛", "━".repeat(actual_width.saturating_sub(2))),
        Style::default().fg(Color::Yellow),
    )));
    
    // Ensure popup area height matches the number of lines
    let actual_height = popup_lines.len() as u16;
    let final_popup_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width,
        height: actual_height.min(popup_area.height),
    };
    
    let popup_widget = Paragraph::new(popup_lines)
        .style(Style::default().bg(hex_color(0x141420)));
    
    f.render_widget(popup_widget, final_popup_area);
}
