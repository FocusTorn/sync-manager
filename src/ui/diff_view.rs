// Diff View Component
// Renders unified diff content

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::core::App;
use crate::operations::DiffEntry;
use super::Styles;

/// Render the diff view panel
pub fn render_diff_view(f: &mut Frame, diff: &DiffEntry, app: &App, area: Rect) {
    if let Some(content) = &app.cached_diff_content {
        // Parse and style all lines
        let all_lines: Vec<Line> = content
            .lines()
            .map(|line| style_diff_line(line))
            .collect();
        
        // Calculate visible area
        let available_height = area.height.saturating_sub(2) as usize;
        let max_offset = all_lines.len().saturating_sub(available_height);
        let scroll_offset = app.diff_scroll_offset.min(max_offset);
        
        // Get visible lines
        let visible_lines: Vec<Line> = all_lines
            .into_iter()
            .skip(scroll_offset)
            .take(available_height)
            .collect();
        
        let diff_widget = Paragraph::new(visible_lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Diff: {} (PgUp/PgDn: Scroll)", diff.path.display())),
            );
        
        f.render_widget(diff_widget, area);
    } else {
        let loading = Paragraph::new("Loading diff...")
            .block(Block::default().borders(Borders::ALL).title("Diff View"));
        f.render_widget(loading, area);
    }
}

/// Style a single diff line based on its prefix
fn style_diff_line(line: &str) -> Line<'static> {
    let style = if line.starts_with('+') && !line.starts_with("+++") {
        Styles::diff_added()
    } else if line.starts_with('-') && !line.starts_with("---") {
        Styles::diff_removed()
    } else if line.starts_with('@') {
        Styles::diff_hunk_header()
    } else if line.starts_with("+++") || line.starts_with("---") {
        Styles::diff_file_header()
    } else {
        Styles::diff_context()
    };
    
    Line::from(Span::styled(line.to_string(), style))
}
