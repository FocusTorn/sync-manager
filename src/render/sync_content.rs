// Sync Manager content rendering
// Renders the diff lists and sync manager UI

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    text::{Line, Span},
    style::{Color, Modifier, Style},
};
use sync_manager::core::{App, ViewMode};
use sync_manager::operations::FileStatus;

/// Render sync manager content (diff lists or side-by-side view)
pub fn render_sync_manager_content(f: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    
    if app.show_side_by_side {
        // Render side-by-side diff view
        render_side_by_side(f, app, area);
    } else {
        // Render normal split view (lists + info)
        render_split_view(f, app, area);
    }
}

/// Render split view (diff lists + info panel)
fn render_split_view(f: &mut Frame, app: &App, area: Rect) {
    // Split into left (lists) and right (info)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);
    
    // Left side: Two diff lists stacked vertically
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[0]);
    
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
    
    // Right side: Info panel
    let info_text = if let Some(diff) = app.selected_diff() {
        format!(
            "File: {}\nStatus: {:?}\n\nPress Enter/Space to view\nside-by-side diff\n\nTab: Switch views\n↑/↓: Navigate",
            diff.path.display(),
            diff.status
        )
    } else {
        "No file selected\n\nUse Tab to switch views\n↑/↓ to navigate\nEnter/Space: Side-by-Side diff\nr: Refresh diffs".to_string()
    };
    
    let info_panel = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("File Info"));
    f.render_widget(info_panel, chunks[1]);
}

/// Render side-by-side diff view
fn render_side_by_side(f: &mut Frame, app: &App, area: Rect) {
    // Use the existing side-by-side renderer from the UI module
    sync_manager::ui::render_side_by_side(f, app, area);
}

/// Render a diff list
fn render_diff_list(
    f: &mut Frame,
    diffs: &[sync_manager::operations::DiffEntry],
    selected_index: usize,
    is_focused: bool,
    area: Rect,
    title: &str,
) {
    let items: Vec<ListItem> = diffs
        .iter()
        .enumerate()
        .map(|(idx, diff)| {
            let style = if idx == selected_index && is_focused {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if idx == selected_index {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };
            
            let (status_icon, status_color) = match diff.status {
                FileStatus::Added => ("A", Color::Green),
                FileStatus::Modified => ("M", Color::Yellow),
                FileStatus::Deleted => ("D", Color::Red),
                FileStatus::Untracked => ("?", Color::Magenta),
                FileStatus::Unchanged => (" ", Color::White),
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
                Span::styled(diff.path.display().to_string(), style),
            ]))
        })
        .collect();
    
    let title_style = if is_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Rgb(102, 102, 102))
    };
    
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(title, title_style)),
    );
    
    let mut list_state = ListState::default();
    list_state.select(Some(selected_index));
    f.render_stateful_widget(list, area, &mut list_state);
}
