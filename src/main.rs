// Sync Manager
// TUI application for managing file synchronization across projects

// MODULES ------------------>> 

mod config;
mod config_validation;
mod dashboard;
mod constants;
mod render;

//--------------------------------------------------------<<
// IMPORTS ------------------>> 

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::Rect,
};
use std::io;
use tui_components::{
    BaseLayout, BaseLayoutConfig, BaseLayoutResult,
    BindingConfig, StatusBarConfig,
    DimmingContext, RectRegistry, Popup, render_popup,
    TabBar, TabBarStyle, RectHandle,
    Toast, render_toasts,
    TabBarManager,
    get_box_by_name,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Module imports
use config_validation::load_and_validate_config;
use constants::*;
use dashboard::DashboardState;
use render::{render_content, render_sync_manager_content};
use sync_manager::core::App;

//--------------------------------------------------------<<

// ┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
// │                                                 MAIN ENTRY POINT                                                 │
// └──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let popup: Option<Popup> = None;
    let toasts: Vec<Toast> = Vec::new();
    let mut registry = RectRegistry::new();
    
    // Initialize dashboard state
    let mut dashboard_state = DashboardState::new();
    
    // Initialize sync manager application state
    let mut sync_app = App::new()?;
    
    // Load and validate configuration from YAML file
    let app_config = load_and_validate_config(None)?;
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Convert YAML bindings > BaseLayoutConfig bindings
    let global_bindings: Vec<BindingConfig> = app_config.application.bindings
        .iter()
        .map(|b| BindingConfig {
            key: b.key.clone(),
            description: b.description.clone(),
        })
        .collect();
    
    // Create base layout configuration from YAML
    let config = BaseLayoutConfig {
        title: app_config.application.title.clone(),
        tabs: vec![],
        global_bindings,
        status_bar: StatusBarConfig {
            default_text: app_config.application.status_bar.default_text.clone(),
            modal_text: app_config.application.status_bar.modal_text.clone(),
        },
    };
    
    // Look up tab bar config by handle name (HWND)
    let tab_bar_config = app_config.tab_bars.values()
        .find(|config| config.hwnd == HWND_MAIN_CONTENT_TAB_BAR)
        .ok_or_else(|| format!(
            "Tab bar with hwnd '{}' not found in config. Available tab bars: {}",
            HWND_MAIN_CONTENT_TAB_BAR,
            app_config.tab_bars.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
        ))?;
    
    // Create and initialize tab bar manager
    let main_content_tab_bar = TabBarManager::create(&mut registry, HWND_MAIN_CONTENT_TAB_BAR, tab_bar_config);
    
    // Store current tab bar instance for click detection
    let mut current_tab_bar: Option<(TabBar, RectHandle)> = None;
    
    // Get tab style for style switching
    let tab_bar_state = registry.get_tab_bar_state(main_content_tab_bar.handle())
        .expect("Tab bar state should be initialized");
    let tab_style = TabBarStyle::from_str(&tab_bar_state.config.style);
    
    let main_content_box_handle_name = HWND_MAIN_CONTENT_BOX;
    let mut original_anchor_metrics: Option<Rect> = None;
    
    // ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
    // │                                           MAIN LOOP                                            │
    // └────────────────────────────────────────────────────────────────────────────────────────────────┘ 
          
    loop {
        // Ensure diff content is cached for the current selection
        ensure_diff_cached(&mut sync_app);
        
        terminal.draw(|f| {
            let area = f.area();
            
            // Create dimming context based on popup state
            let dimming = DimmingContext::new(popup.is_some());
            
            // Render base layout
            let base_layout = BaseLayout::new(&config, None, &dimming);
            let layout_result: BaseLayoutResult = base_layout.render(f, area, &mut registry);
            let content_area = layout_result.content_area;
            
            // Store original anchor metrics after first render
            if original_anchor_metrics.is_none() {
                if let Some(anchor_handle) = registry.get_handle(main_content_box_handle_name) {
                    if let Some(metrics) = registry.get_metrics(anchor_handle) {
                        original_anchor_metrics = Some(metrics.into());
                    }
                } else {
                    original_anchor_metrics = Some(content_area);
                }
            }
            
            // Restore anchor position
            if let (Some(anchor_handle), Some(original_metrics)) = (registry.get_handle(main_content_box_handle_name), original_anchor_metrics) {
                registry.update(anchor_handle, original_metrics);
            }
            
            // Initialize or update main content bounding box
            if let Some(handle) = registry.get_handle(main_content_box_handle_name) {
                registry.update(handle, content_area);
            } else {
                let _handle = registry.register(Some(main_content_box_handle_name), content_area);
            }
            
            // Prepare tab bar
            let tab_bar_result = main_content_tab_bar.prepare(&mut registry, Some(tab_style));
            
            // Get render area
            let render_area = if let Some(box_manager) = get_box_by_name(&registry, main_content_box_handle_name) {
                box_manager.prepare(&mut registry).unwrap_or(content_area)
            } else {
                content_area
            };
            
            // Render content box border
            render_content(f, render_area, &dimming);
            
            // Render tab content
            if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                    if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                        if tab_config.id == "dashboard" {
                            // Create nested area for tab content (account for borders)
                            let nested_area = Rect {
                                x: render_area.x.saturating_add(1),
                                y: render_area.y.saturating_add(1),
                                width: render_area.width.saturating_sub(2),
                                height: render_area.height.saturating_sub(2),
                            };
                            
                            // Render sync manager content (diff lists) in the dashboard
                            render_sync_manager_content(f, &sync_app, nested_area);
                        }
                    }
                }
            }
            
            // Render tab bar
            if let Some((tab_bar, anchor_handle, tab_bar_state)) = tab_bar_result {
                tab_bar.render_with_state(f, &mut registry, &tab_bar_state, Some(&dimming));
                
                // Store tab bar for click detection
                current_tab_bar = Some((tab_bar, anchor_handle));
            }
            
            // Render popups and toasts
            if let Some(ref popup) = popup {
                render_popup(f, area, popup);
            }
            
            render_toasts(f, area, &toasts);
        })?;
        
        // ┌──────────────────────────────────────────────────────────────────────────────────────────────┐
        // │                              Handle events (keyboard and mouse)                              │
        // └──────────────────────────────────────────────────────────────────────────────────────────────┘                
        
        match crossterm::event::poll(std::time::Duration::from_millis(50)) {
            Ok(true) => {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
                        
                        // Handle global bindings
                        match key.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            _ => {}
                        }
                        
                        // Handle sync manager navigation
                        if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                            if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                                if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                                    if tab_config.id == "dashboard" {
                                        // Handle sync manager navigation keys
                                        match key.code {
                                            KeyCode::Up => {
                                                if sync_app.show_side_by_side {
                                                    sync_app.scroll_up(1);
                                                } else {
                                                    sync_app.select_previous();
                                                }
                                            }
                                            KeyCode::Down => {
                                                if sync_app.show_side_by_side {
                                                    sync_app.scroll_down(1);
                                                } else {
                                                    sync_app.select_next();
                                                }
                                            }
                                            KeyCode::PageUp => {
                                                if sync_app.show_side_by_side {
                                                    sync_app.scroll_up(10);
                                                }
                                            }
                                            KeyCode::PageDown => {
                                                if sync_app.show_side_by_side {
                                                    sync_app.scroll_down(10);
                                                }
                                            }
                                            KeyCode::Enter | KeyCode::Char(' ') => {
                                                if !sync_app.show_side_by_side {
                                                    sync_app.toggle_side_by_side();
                                                }
                                            }
                                            KeyCode::Esc => {
                                                if sync_app.show_side_by_side {
                                                    sync_app.show_side_by_side = false;
                                                    sync_app.side_by_side_source = None;
                                                    sync_app.side_by_side_dest = None;
                                                }
                                            }
                                            KeyCode::Tab => {
                                                if !sync_app.show_side_by_side {
                                                    sync_app.toggle_view_mode();
                                                }
                                            }
                                            KeyCode::Char('r') => {
                                                if !sync_app.show_side_by_side {
                                                    let _ = sync_app.refresh_diffs();
                                                }
                                            }
                                            KeyCode::Char('q') => {
                                                sync_app.quit();
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        // Handle mouse scrolling for dashboard output
                        if matches!(mouse_event.kind, MouseEventKind::ScrollUp | MouseEventKind::ScrollDown) {
                            if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                                if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                                    if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                                        if tab_config.id == "dashboard" {
                                            match mouse_event.kind {
                                                MouseEventKind::ScrollUp => {
                                                    dashboard_state.scroll_output_up(3);
                                                }
                                                MouseEventKind::ScrollDown => {
                                                    dashboard_state.scroll_output_down(3);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        if mouse_event.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                            // Handle mouse clicks on tabs
                            if let Some((ref tab_bar, _handle)) = &current_tab_bar {
                                let clicked_tab: Option<usize> = tab_bar.get_tab_at(mouse_event.column, mouse_event.row, Some(&registry));
                                if let Some(clicked_tab_idx) = clicked_tab {
                                    if tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic {
                                        main_content_tab_bar.set_active(&mut registry, clicked_tab_idx);
                                    }
                                }
                            }
                        }
                    }
                    Event::Resize(_, _) => {
                        // Terminal resize - will be handled on next draw
                    }
                    _ => {}
                }
            }
            Ok(false) => {
                // No event available
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
                use sync_manager::operations::DiffEngine;
                app.cached_diff_content = DiffEngine::load_diff_content(diff);
                app.cached_diff_path = Some(diff_path);
                app.diff_scroll_offset = 0;
            }
        }
    } else {
        app.cached_diff_content = None;
        app.cached_diff_path = None;
    }
}
