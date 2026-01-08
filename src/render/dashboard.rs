// Dashboard panel rendering

use crate::dashboard::{DashboardState, SCROLL_TO_BOTTOM};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

/// Render dashboard panel
#[allow(dead_code)]  // Reserved for future dashboard functionality
pub fn render_dashboard(
    f: &mut Frame,
    area: Rect,
    dashboard_state: &mut DashboardState,
) {
    // Ensure area is valid
    if area.width == 0 || area.height == 0 {
        return;
    }
    
    // Split into status bar and output
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status bar (3 lines: border + text + border)
            Constraint::Min(0),     // Output (remaining space)
        ])
        .split(area);
    
    // Status bar box
    let status_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Status ", Style::default().fg(Color::White)))
        .border_style(Style::default().fg(Color::Rgb(102, 102, 102)))
        .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));
    
    // Show regular status text
    let status_para = Paragraph::new(dashboard_state.status_text.as_ref())
        .block(status_block)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(status_para, chunks[0]);
    
    // Output box with scrolling
    let output_area = chunks[1];
    let output_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Output ", Style::default().fg(Color::White)))
        .border_style(Style::default().fg(Color::Rgb(102, 102, 102)))
        .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));
    let output_inner = output_block.inner(output_area);
    
    // Calculate visible lines
    let visible_height = output_inner.height as usize;
    let total_lines = dashboard_state.output_lines.len();
    
    // Calculate maximum scroll position (0-based index of first visible line when at bottom)
    // If total_lines <= visible_height, max_scroll is 0 (no scrolling needed)
    let max_scroll = if total_lines > visible_height {
        total_lines - visible_height
    } else {
        0
    };
    
    // Handle auto-scroll: if scroll position is SCROLL_TO_BOTTOM sentinel, scroll to bottom
    if dashboard_state.output_scroll == SCROLL_TO_BOTTOM {
        dashboard_state.scroll_to_bottom(visible_height);
    }
    
    // Ensure scroll position is valid (clamp to valid range [0, max_scroll])
    dashboard_state.output_scroll = dashboard_state.output_scroll.min(max_scroll);
    
    // Get visible lines
    let start_line = dashboard_state.output_scroll;
    let end_line = (start_line + visible_height).min(total_lines);
    
    // Convert lines to ratatui Lines
    let visible_lines: Vec<Line> = if dashboard_state.output_lines.is_empty() {
        vec![Line::from(Span::styled(
            "No output yet.",
            Style::default().fg(Color::Rgb(128, 128, 128)),
        ))]
    } else {
        dashboard_state.output_lines[start_line..end_line]
            .iter()
            .map(|line| Line::from(Span::raw(line.clone())))
            .collect()
    };
    
    // Render the block (borders and title) to the full area
    f.render_widget(output_block.clone(), output_area);
    
    // Create content area that's one column narrower to leave space for scrollbar
    // This ensures content doesn't overlap with the scrollbar
    let content_area = if total_lines > visible_height {
        // Leave one column for scrollbar
        Rect {
            x: output_inner.x,
            y: output_inner.y,
            width: output_inner.width.saturating_sub(1),
            height: output_inner.height,
        }
    } else {
        // No scrollbar, use full width
        output_inner
    };
    
    // Render content without block (block already rendered above)
    let output_para = Paragraph::new(visible_lines)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(output_para, content_area);
    
    // Render scrollbar if there are more lines than visible
    if total_lines > visible_height {
        // Position scrollbar on the right edge of the inner content area
        let scrollbar_area = Rect {
            x: output_inner.x + output_inner.width.saturating_sub(1),
            y: output_inner.y,
            width: 1,
            height: output_inner.height,
        };
        
        // Create scrollbar state
        let content_length = total_lines;
        let viewport_length = visible_height;
        let position = dashboard_state.output_scroll.min(max_scroll);
        
        let mut scrollbar_state = ScrollbarState::new(content_length)
            .viewport_content_length(viewport_length)
            .position(position);
        
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .thumb_symbol("█")
            .track_symbol(Some("│"));
        
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}
