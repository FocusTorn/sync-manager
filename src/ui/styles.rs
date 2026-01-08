// UI Styles
// Color schemes and styling for the TUI

use ratatui::style::{Color, Modifier, Style};

/// Application color scheme and styles
pub struct Styles;

impl Styles {
    // === Header / Footer ===
    
    pub fn header() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn footer() -> Style {
        Style::default().fg(Color::Yellow)
    }
    
    // === List Items ===
    
    pub fn list_selected_focused() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    }
    
    pub fn list_selected_unfocused() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn list_normal() -> Style {
        Style::default()
    }
    
    // === File Status Colors ===
    
    pub fn status_added() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn status_modified() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn status_deleted() -> Style {
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn status_untracked() -> Style {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn status_unchanged() -> Style {
        Style::default().fg(Color::Gray)
    }
    
    // === Diff View Colors ===
    
    pub fn diff_added() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn diff_removed() -> Style {
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn diff_hunk_header() -> Style {
        Style::default().fg(Color::Cyan)
    }
    
    pub fn diff_file_header() -> Style {
        Style::default().fg(Color::Gray)
    }
    
    pub fn diff_context() -> Style {
        Style::default()
    }
    
    // === Side-by-Side Diff Colors ===
    // Colors are compiled from config.yaml
    
    /// Background for modified source lines (dim red)
    pub fn side_by_side_source_modified_bg() -> Style {
        let (r, g, b) = crate::core::app_config::compiled::SOURCE_DIM_BG;
        Style::default().bg(Color::Rgb(r, g, b))
    }
    
    /// Highlight for changed parts in source (bright red)
    pub fn side_by_side_source_highlight() -> Style {
        let (r, g, b) = crate::core::app_config::compiled::SOURCE_BRIGHT_BG;
        Style::default().bg(Color::Rgb(r, g, b))
    }
    
    /// Background for modified destination lines (dim green)
    pub fn side_by_side_dest_modified_bg() -> Style {
        let (r, g, b) = crate::core::app_config::compiled::DEST_DIM_BG;
        Style::default().bg(Color::Rgb(r, g, b))
    }
    
    /// Highlight for changed parts in destination (bright green)
    pub fn side_by_side_dest_highlight() -> Style {
        let (r, g, b) = crate::core::app_config::compiled::DEST_BRIGHT_BG;
        Style::default().bg(Color::Rgb(r, g, b))
    }
    
    /// Gutter (line numbers) style
    pub fn gutter() -> Style {
        Style::default().fg(Color::Rgb(68, 68, 68))
    }
    
    /// Fold indicator style
    pub fn fold_indicator() -> Style {
        Style::default()
            .fg(Color::Rgb(150, 150, 150))
            .add_modifier(Modifier::ITALIC)
    }
    
    // === Border Styles ===
    
    pub fn border_focused() -> Style {
        Style::default().fg(Color::Cyan)
    }
    
    pub fn border_unfocused() -> Style {
        Style::default().fg(Color::Gray)
    }
    
    pub fn title_focused() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn title_unfocused() -> Style {
        Style::default().fg(Color::Gray)
    }
}
