// UI module
// TUI components and views for the sync manager

pub mod app_view;
pub mod diff_list;
pub mod diff_view;
pub mod side_by_side;
pub mod styles;

use anyhow::Result;
use crossterm::event;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;
use std::time::Duration;

use crate::core::{App, AppEvent, EventHandler};

pub use app_view::render_app;
pub use diff_list::render_diff_list;
pub use diff_view::render_diff_view;
pub use side_by_side::render_side_by_side;
pub use styles::Styles;

/// Run the main application event loop
pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Ensure diff is cached before rendering
        ensure_diff_cached(app);
        
        // Render the UI
        terminal.draw(|f| render_app(f, app))?;
        
        // Handle events
        if event::poll(Duration::from_millis(250))? {
            let event = event::read()?;
            let app_event = EventHandler::handle(event);
            
            handle_event(app, app_event);
        }
        
        // Check if we should quit
        if app.should_quit {
            return Ok(());
        }
    }
}

/// Ensure diff content is cached for the current selection
fn ensure_diff_cached(app: &mut App) {
    let current_path = app.selected_diff().map(|d| d.path.clone());
    
    if let Some(diff_path) = current_path {
        let needs_reload = match &app.cached_diff_path {
            Some(cached_path) => cached_path != &diff_path,
            None => true,
        };
        
        if needs_reload {
            if let Some(diff) = app.selected_diff() {
                app.cached_diff_content = crate::operations::DiffEngine::load_diff_content(diff);
                app.cached_diff_path = Some(diff_path);
                app.diff_scroll_offset = 0;
            }
        }
    } else {
        app.cached_diff_content = None;
        app.cached_diff_path = None;
    }
}

/// Handle an application event
fn handle_event(app: &mut App, event: AppEvent) {
    match event {
        AppEvent::Quit => app.quit(),
        AppEvent::SelectPrevious => {
            if app.show_side_by_side {
                app.scroll_up(1);
            } else {
                app.select_previous();
            }
        }
        AppEvent::SelectNext => {
            if app.show_side_by_side {
                app.scroll_down(1);
            } else {
                app.select_next();
            }
        }
        AppEvent::ToggleViewMode => app.toggle_view_mode(),
        AppEvent::ToggleSideBySide => app.toggle_side_by_side(),
        AppEvent::ToggleFold => app.toggle_fold(),
        AppEvent::ScrollUp(amount) => app.scroll_up(amount),
        AppEvent::ScrollDown(amount) => app.scroll_down(amount),
        AppEvent::PageUp => app.scroll_up(10),
        AppEvent::PageDown => app.scroll_down(10),
        AppEvent::Back => {
            if app.show_side_by_side {
                app.show_side_by_side = false;
                app.side_by_side_source = None;
                app.side_by_side_dest = None;
                app.fold_unchanged = false;
            } else {
                app.quit();
            }
        }
        AppEvent::Refresh => {
            let _ = app.refresh_diffs();
        }
        AppEvent::SyncSelected => {
            // TODO: Implement sync selected
        }
        AppEvent::SyncAll => {
            // TODO: Implement sync all
        }
        AppEvent::None => {}
    }
}
