// Content rendering

use ratatui::{
    Frame,
    layout::Rect,
    widgets::Block,
    widgets::Borders,
    style::Style,
};
use tui_components::DimmingContext;

/// Render main content box border
pub fn render_content(f: &mut Frame, area: Rect, dimming: &DimmingContext) {
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(dimming.border_color(true)));
    
    f.render_widget(content_block, area);
}
