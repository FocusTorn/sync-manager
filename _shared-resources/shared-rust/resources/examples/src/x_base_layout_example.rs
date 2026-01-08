// Layout Demo - Tabbed Manager
// Demonstrates the BaseLayout component with title, tabs, status bar, and content area
//
// Usage:
//   cargo run --bin layout-tabbed-manager-demo --package component-examples

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
};
use std::io;
use tui_components::{
    BaseLayout, BaseLayoutConfig, BaseLayoutResult,
    BindingConfig, StatusBarConfig,
    DimmingContext, RectRegistry, Popup, render_popup,
    TabBar, TabBarItem, TabBarStyle, TabBarAlignment, TabBarPosition, RectHandle,
    Toast, ToastType, render_toasts,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create base layout configuration
    let config = BaseLayoutConfig {
        title: "Layout Demo - Tabbed Manager".to_string(),
        tabs: vec![], // Tab bar disabled for now
        global_bindings: vec![
            BindingConfig { key: "[7]".to_string(), description: "Tabbed style".to_string() },
            BindingConfig { key: "[8]".to_string(), description: "Boxed style".to_string() },
            BindingConfig { key: "[9]".to_string(), description: "Text style".to_string() },
            BindingConfig { key: "[Alt+8]".to_string(), description: "BoxStatic style".to_string() },
            BindingConfig { key: "[Alt+9]".to_string(), description: "TextStatic style".to_string() },
            BindingConfig { key: "[p]".to_string(), description: "Cycle popups".to_string() },
            BindingConfig { key: "[t]".to_string(), description: "Show toast".to_string() },
            BindingConfig { key: "[q]".to_string(), description: "Quit".to_string() },
        ],
        status_bar: StatusBarConfig {
            default_text: "Status: Ready | [7] Tabbed | [8] Boxed | [9] Text | [Alt+8] BoxStatic | [Alt+9] TextStatic | [p] Cycle Popups | [t] Toast | [q] Quit".to_string(),
            modal_text: Some("Modal active - Press 'm' to close | Use arrow keys to navigate | Enter to confirm | Esc to cancel".to_string()),
        },
    };
    
    // Application state
    let mut popup: Option<Popup> = None;
    let mut popup_cycle_state = 0u8; // 0=info, 1=warning, 2=error, then cycles
    let mut toasts: Vec<Toast> = Vec::new();
    let mut toast_counter = 0u32; // Counter for unique toast messages
    let mut tab_style = TabBarStyle::Tab;
    let mut active_tab_index = 0;
    
    // Tab names (defined outside loop for event handler access)
    let tab_names = vec!["Dashboard".to_string(), "Changes".to_string(), "Baselines".to_string()];
    
    // Create registry for handle-based positioning
    let mut registry = RectRegistry::new();
    
    // Store tab bar for click detection (needs to persist across frames)
    let mut current_tab_bar: Option<(TabBar, RectHandle)> = None;
    
    // Render loop
    loop {
        terminal.draw(|f| {
            let area = f.area();
            
            // Create dimming context based on popup state
            let dimming = DimmingContext::new(popup.is_some());
            
            // Render base layout (no tabs, so pass None)
            let base_layout = BaseLayout::new(
                &config,
                None,
                &dimming,
            );
            let result: BaseLayoutResult = base_layout.render(f, area, &mut registry);
            
            // Get the content area
            let mut content_area = result.content_area;
            let main_container_handle = registry.get_handle("hwndMainContainer");
            
            // If Tab style, move the container's top border down by one row and reduce height by 1
            // This creates space for the decorative line above the tab bar
            if tab_style == TabBarStyle::Tab {
                if let Some(handle) = main_container_handle {
                    if let Some(metrics) = registry.get_metrics(handle) {
                        let mut updated_metrics = metrics;
                        updated_metrics.y = updated_metrics.y.saturating_add(1);
                        updated_metrics.height = updated_metrics.height.saturating_sub(1).max(1);
                        registry.update(handle, updated_metrics.into());
                        content_area = updated_metrics.into();
                    }
                }
            }
            
            // Render content Block FIRST (so tab bar can render on top of it)
            render_content(f, content_area, &dimming);
            
            // Render tab bar AFTER content Block (so it appears on top)
            // Store tab bar for click detection
            if let Some(main_container_handle) = main_container_handle {
                let tab_items: Vec<TabBarItem> = tab_names
                    .iter()
                    .enumerate()
                    .map(|(idx, name)| TabBarItem {
                        name: name.clone(),
                        active: idx == active_tab_index && tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic,
                    })
                    .collect();
                
                // Always use TopOfHandle to get the current container position from registry
                // For Tab style, this will be the moved position; for others, the original position
                let tab_bar = TabBar::new(tab_items, tab_style, TabBarAlignment::Center)
                    .with_color(ratatui::style::Color::Cyan)
                    .with_position(TabBarPosition::TopOfHandle(main_container_handle));
                tab_bar.render_with_registry(f, Some(&mut registry), Some(&dimming));
                
                // Store for click detection
                current_tab_bar = Some((tab_bar, main_container_handle));
            }
            
            // Render popup if active
            if let Some(ref popup) = popup {
                render_popup(f, area, popup);
            }
            
            // Filter out expired toasts (older than 1.5 seconds)
            let now = std::time::SystemTime::now();
            toasts.retain(|toast| {
                if let Ok(duration) = now.duration_since(toast.shown_at) {
                    duration.as_secs_f64() < 1.5
                } else {
                    false // Remove if time calculation fails
                }
            });
            
            // Render toasts (stacked in bottom-left)
            render_toasts(f, area, &toasts);
        })?;
        
        // Handle events (keyboard and mouse) - use non-blocking poll to allow toast timeout checking
        match crossterm::event::poll(std::time::Duration::from_millis(50)) {
            Ok(true) => {
                // Event available, read it
                match event::read()? {
                    Event::Key(key) => {
                match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    // Quit application
                    break;
                }
                KeyCode::Esc => {
                    // Close popup on Esc
                    popup = None;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    // Cycle through popups: info -> warning -> error -> repeat
                    popup = match popup_cycle_state {
                        0 => {
                            popup_cycle_state = 1;
                            Some(Popup::info(
                                "Information".to_string(),
                                "This is an informational message. Everything is working correctly.".to_string()
                            ))
                        }
                        1 => {
                            popup_cycle_state = 2;
                            Some(Popup::warning(
                                "Warning".to_string(),
                                "This is a warning message. Please review your settings before proceeding.".to_string()
                            ))
                        }
                        _ => {
                            popup_cycle_state = 0;
                            Some(Popup::error(
                                "Error".to_string(),
                                "An error has occurred. Please check your configuration and try again.".to_string()
                            ))
                        }
                    };
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    // Add a new toast (no cooldown, can stack multiple)
                    toast_counter += 1;
                    let toast_type = match toast_counter % 3 {
                        0 => ToastType::Success,
                        1 => ToastType::Info,
                        _ => ToastType::Error,
                    };
                    let message = format!("Toast notification #{}", toast_counter);
                    toasts.push(Toast::new(message, toast_type));
                }
                KeyCode::Char('7') => {
                    // Tabbed style
                    tab_style = TabBarStyle::Tab;
                }
                KeyCode::Char('8') => {
                    // Check for Alt modifier
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        // BoxStatic style (Alt+8)
                        tab_style = TabBarStyle::BoxStatic;
                    } else {
                        // Boxed style
                        tab_style = TabBarStyle::Boxed;
                    }
                }
                KeyCode::Char('9') => {
                    // Check for Alt modifier
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        // TextStatic style (Alt+9)
                        tab_style = TabBarStyle::TextStatic;
                    } else {
                        // Text style
                        tab_style = TabBarStyle::Text;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    // Navigate tabs (only for non-static styles)
                    if tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic {
                        active_tab_index = if active_tab_index > 0 {
                            active_tab_index - 1
                        } else {
                            tab_names.len() - 1
                        };
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    // Navigate tabs (only for non-static styles)
                    if tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic {
                        active_tab_index = (active_tab_index + 1) % tab_names.len();
                    }
                }
                _ => {}
                }
            }
            Event::Mouse(mouse_event) => {
                // Handle mouse clicks on tabs
                if mouse_event.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                    if let Some((ref tab_bar, _handle)) = current_tab_bar {
                        let clicked_tab: Option<usize> = tab_bar.get_tab_at(mouse_event.column, mouse_event.row, Some(&registry));
                        if let Some(clicked_tab_idx) = clicked_tab {
                            // Only switch tabs for non-static styles
                            if tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic {
                                active_tab_index = clicked_tab_idx;
                            }
                        }
                    }
                }
            }
            Event::Resize(_, _) => {
                // Terminal resize - will be handled on next draw
            }
            _ => {
                // Ignore other event types
            }
                }
            }
            Ok(false) => {
                // No event available, but render loop will continue to check toast timeouts
            }
            Err(_) => {
                // Error polling, continue anyway
            }
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}

/// Render main content
fn render_content(f: &mut ratatui::Frame, area: Rect, dimming: &DimmingContext) {
    // Use full area - no padding (main container should be full width)
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(dimming.border_color(true)));
    
    // Render just the block with no text
    f.render_widget(content_block, area);
}


