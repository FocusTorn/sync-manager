// Diff List Component
// Renders a list of diff entries

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::operations::{DiffEntry, FileStatus};
use super::Styles;

/// Render a diff list component
pub fn render_diff_list(
    f: &mut Frame,
    diffs: &[DiffEntry],
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
                Styles::list_selected_focused()
            } else if idx == selected_index {
                Styles::list_selected_unfocused()
            } else {
                Styles::list_normal()
            };
            
            let (status_icon, status_style) = match diff.status {
                FileStatus::Added => ("A", Styles::status_added()),
                FileStatus::Modified => ("M", Styles::status_modified()),
                FileStatus::Deleted => ("D", Styles::status_deleted()),
                FileStatus::Untracked => ("?", Styles::status_untracked()),
                FileStatus::Unchanged => (" ", Styles::status_unchanged()),
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", status_icon), status_style),
                Span::styled(diff.path.display().to_string(), style),
            ]))
        })
        .collect();
    
    let title_style = if is_focused {
        Styles::title_focused()
    } else {
        Styles::title_unfocused()
    };
    
    let border_style = if is_focused {
        Styles::border_focused()
    } else {
        Styles::border_unfocused()
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
