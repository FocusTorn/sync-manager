// Application View
// Main application layout and rendering

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::core::{App, ViewMode};
use super::{render_diff_list, render_side_by_side, Styles};

/// Render the entire application
pub fn render_app(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());
    
    render_header(f, chunks[0]);
    render_main_content(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

/// Render the header bar
fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new("Sync Manager TUI")
        .style(Styles::header())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

/// Render the main content area
fn render_main_content(f: &mut Frame, app: &App, area: Rect) {
    if app.show_side_by_side {
        render_side_by_side(f, app, area);
    } else {
        render_split_view(f, app, area);
    }
}

/// Render the split view (diff lists + diff view)
fn render_split_view(f: &mut Frame, app: &App, area: Rect) {
    // Split into left (lists) and right (diff view)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // Left side: Two diff lists stacked vertically
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);
    
    // Top list: shared -> project
    render_diff_list(
        f,
        &app.shared_to_project_diffs,
        app.shared_to_project_index,
        app.view_mode == ViewMode::SharedToProject,
        left_chunks[0],
        "_shared → .project",
    );
    
    // Bottom list: project -> shared
    render_diff_list(
        f,
        &app.project_to_shared_diffs,
        app.project_to_shared_index,
        app.view_mode == ViewMode::ProjectToShared,
        left_chunks[1],
        ".project → _shared",
    );
    
    // Right side: Info panel (diff view disabled)
    let info_text = if let Some(diff) = app.selected_diff() {
        format!(
            "File: {}\nStatus: {:?}\n\nPress Enter/Space to view\nside-by-side diff",
            diff.path.display(),
            diff.status
        )
    } else {
        "No file selected\n\nUse Tab to switch between views\n↑/↓ to navigate\nEnter/Space: Side-by-Side diff".to_string()
    };
    
    let info_panel = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("File Info"));
    f.render_widget(info_panel, main_chunks[1]);
}

/// Render the footer bar
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.show_side_by_side {
        if app.fold_unchanged {
            "q: Quit | Esc: Back | ↑/↓: Scroll | F: Unfold | PgUp/PgDn: Scroll | Mouse Wheel: Scroll"
        } else {
            "q: Quit | Esc: Back | ↑/↓: Scroll | F: Fold | PgUp/PgDn: Scroll | Mouse Wheel: Scroll"
        }
    } else {
        "q: Quit | Tab: Switch View | ↑/↓: Navigate | Enter/Space: Side-by-Side | PgUp/PgDn: Scroll | r: Refresh"
    };
    
    let footer = Paragraph::new(help_text)
        .style(Styles::footer())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}
