// Event Handling
// Application event types and handler infrastructure

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Application events that can be handled
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Quit the application
    Quit,
    
    /// Move selection up
    SelectPrevious,
    
    /// Move selection down
    SelectNext,
    
    /// Toggle between view modes
    ToggleViewMode,
    
    /// Toggle side-by-side diff view
    ToggleSideBySide,
    
    /// Toggle fold unchanged regions
    ToggleFold,
    
    /// Scroll up by amount
    ScrollUp(usize),
    
    /// Scroll down by amount
    ScrollDown(usize),
    
    /// Page up
    PageUp,
    
    /// Page down
    PageDown,
    
    /// Go back / escape current mode
    Back,
    
    /// Refresh data
    Refresh,
    
    /// Sync selected file
    SyncSelected,
    
    /// Sync all files
    SyncAll,
    
    /// No operation
    None,
}

/// Event handler that converts terminal events to application events
pub struct EventHandler;

impl EventHandler {
    /// Convert a crossterm event to an application event
    pub fn handle(event: Event) -> AppEvent {
        match event {
            Event::Key(key) => Self::handle_key(key),
            Event::Mouse(mouse) => Self::handle_mouse(mouse),
            _ => AppEvent::None,
        }
    }
    
    /// Handle keyboard events
    fn handle_key(key: KeyEvent) -> AppEvent {
        // Only handle key press events
        if key.kind != crossterm::event::KeyEventKind::Press {
            return AppEvent::None;
        }
        
        match key.code {
            // Quit
            KeyCode::Char('q') => AppEvent::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => AppEvent::Quit,
            
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => AppEvent::SelectPrevious,
            KeyCode::Down | KeyCode::Char('j') => AppEvent::SelectNext,
            
            // Scrolling
            KeyCode::PageUp => AppEvent::PageUp,
            KeyCode::PageDown => AppEvent::PageDown,
            
            // View toggles
            KeyCode::Tab => AppEvent::ToggleViewMode,
            KeyCode::Enter | KeyCode::Char(' ') => AppEvent::ToggleSideBySide,
            KeyCode::Char('f') => AppEvent::ToggleFold,
            
            // Back / Escape
            KeyCode::Esc => AppEvent::Back,
            
            // Refresh
            KeyCode::Char('r') => AppEvent::Refresh,
            
            // Sync operations
            KeyCode::Char('s') => AppEvent::SyncSelected,
            KeyCode::Char('S') => AppEvent::SyncAll,
            
            _ => AppEvent::None,
        }
    }
    
    /// Handle mouse events
    fn handle_mouse(mouse: MouseEvent) -> AppEvent {
        match mouse.kind {
            MouseEventKind::ScrollUp => AppEvent::ScrollUp(1),
            MouseEventKind::ScrollDown => AppEvent::ScrollDown(1),
            _ => AppEvent::None,
        }
    }
}
