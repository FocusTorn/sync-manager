// Layout Demo - Tabbed Manager
// Demonstrates the BaseLayout component with title, tabs, status bar, and content area
//
// Usage:
//   cargo build --bin layout-tabbed-manager-demo --package component-examples
//   cargo run   --bin layout-tabbed-manager-demo --package component-examples

// IMPORTS ------------------>> 

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
};
use std::io;
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;
use tui_components::{
    BaseLayout, BaseLayoutConfig, BaseLayoutResult,
    BindingConfig, StatusBarConfig,
    DimmingContext, RectRegistry, Popup, render_popup,
    TabBar, TabBarStyle, RectHandle,
    Toast, ToastType, render_toasts,
    TabBarConfigYaml, TabBarManager,
    get_box_by_name, BoundingBox,
    SplitDiffView, SplitDiffViewConfig, SplitDiffViewState,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

//--------------------------------------------------------<<

// Static handle names (HWND IDs)
const HWND_MAIN_CONTENT_BOX: &str = "hwndMainContentBox";
const HWND_MAIN_CONTENT_TAB_BAR: &str = "hwndMainContentTabBar";
const HWND_DIFF_VIEW: &str = "hwndDiffView";


// Type aliases for cleaner code (use library's YAML structures)
type TabBarConfig = TabBarConfigYaml;

#[derive(Debug, Clone, Deserialize)]
struct AppConfig { //>
    application: ApplicationConfig,
    #[serde(rename = "tab_bars")]
    tab_bars: std::collections::HashMap<String, TabBarConfig>,
    // content: Option<ContentConfig>,  // Commented out - not currently used
} //<

#[derive(Debug, Clone, Deserialize)]
struct ApplicationConfig { //>
    title: String,
    bindings: Vec<BindingConfigYaml>,
    status_bar: StatusBarConfigYaml,
} //<

#[derive(Debug, Clone, Deserialize)]
struct BindingConfigYaml { //>
    key: String,
    description: String,
} //<

#[derive(Debug, Clone, Deserialize)]
struct StatusBarConfigYaml { //>
    default_text: String,
    modal_text: String,
} //<


// #[derive(Debug, Clone, Deserialize)]
// struct NavigationConfig { //>
//     left: Vec<String>,
//     right: Vec<String>,
// } //<

// #[derive(Debug, Clone, Deserialize)]
// struct TabContentConfig { //>
//     #[serde(rename = "type")]
//     content_type: String,
//     value: Option<String>,
// } //<

// #[derive(Debug, Clone, Deserialize)]
// struct ContentConfig { //>
//     render_borders: Option<bool>,
//     border_style: Option<String>,
//     widgets: Option<std::collections::HashMap<String, WidgetConfig>>,
// } //<

// #[derive(Debug, Clone, Deserialize)]
// struct WidgetConfig { //>
//     #[serde(rename = "type")]
//     widget_type: String,
//     text: Option<String>,
// } //<

fn load_config(config_path: Option<PathBuf>) -> Result<AppConfig, Box<dyn std::error::Error>> { //>
    let path = config_path.unwrap_or_else(|| {
        // Default to config.yaml in the examples directory
        let mut default_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        default_path.push("config.yaml");
        default_path
    });
    
    let contents = fs::read_to_string(&path)?;
    let config: AppConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
} //<


// ┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
// │                                                 MAIN ENTRY POINT                                                 │
// └──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut popup: Option<Popup> = None;     //
    let mut popup_cycle_state = 0u8;         // 0=info, 1=warning, 2=error, then cycles
    let mut toasts: Vec<Toast> = Vec::new(); //
    let mut toast_counter = 0u32;            // Counter for unique toast messages
    let mut registry = RectRegistry::new();  // Create registry for handle-based positioning
    
    // Split diff view state
    let mut diff_view_state = SplitDiffViewState::default();
    
    // Load configuration from YAML file -------------------------------------->> 
    let app_config = match load_config(None) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config.yaml: {}", e);
            eprintln!("Using default configuration");
            // Return error or use defaults - for now, return error to ensure config is set up
            return Err(e);
        }
    };
    //--------------------------------------------------------------------------------------------<<
    // Setup terminal --------------------------------------------------------->> 
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    //--------------------------------------------------------------------------------------------<<
    // Convert YAML bindings > BaseLayoutConfig bindings ---------------------->> 
    let global_bindings: Vec<BindingConfig> = app_config.application.bindings
        .iter()
        .map(|b| BindingConfig {
            key: b.key.clone(),
            description: b.description.clone(),
        })
        .collect();
    //--------------------------------------------------------------------------------------------<<
    // Create base layout configuration from YAML ----------------------------->> 
    let config = BaseLayoutConfig {
        title: app_config.application.title.clone(),
        tabs: vec![], // Tab bar disabled for now
        global_bindings,
        status_bar: StatusBarConfig {
            default_text: app_config.application.status_bar.default_text.clone(),
            modal_text: Some(app_config.application.status_bar.modal_text.clone()),
        },
    };
    //--------------------------------------------------------------------------------------------<<
    
    // Look up tab bar config by handle name (HWND) --------------------------->> 
    let tab_bar_config = app_config.tab_bars.values()
        .find(|config| config.hwnd == HWND_MAIN_CONTENT_TAB_BAR)
        .ok_or_else(|| format!(
            "Tab bar with hwnd '{}' not found in config. Available tab bars: {}",
            HWND_MAIN_CONTENT_TAB_BAR,
            app_config.tab_bars.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
        ))?;
        
    //--------------------------------------------------------------------------------------------<<
    
    
        
    // Create and initialize tab bar manager (OOP-like: object associated with handle)
    // All tab bar operations are now accessed through this manager object
    let main_content_tab_bar = TabBarManager::create(&mut registry, HWND_MAIN_CONTENT_TAB_BAR, tab_bar_config);
    
    // Store current tab bar instance for click detection (associated with the manager's handle)
    let mut current_tab_bar: Option<(TabBar, RectHandle)> = None;
    
    // Create main content bounding box handle (will be initialized with content area in render loop)
    // Note: We store the handle name instead of the BoundingBox since the box can be retrieved from registry
    let main_content_box_handle_name = HWND_MAIN_CONTENT_BOX;
    
    // Get tab style for style switching -------------------------------------->> 
    let tab_bar_state = registry.get_tab_bar_state(main_content_tab_bar.handle())
        .expect("Tab bar state should be initialized");
    let mut tab_style = TabBarStyle::from_str(&tab_bar_state.config.style);
    // Note: The tab bar's anchor should be "hwndMainContentBox" (HWND_MAIN_CONTENT_BOX) as specified in config
    
    //--------------------------------------------------------------------------------------------<<
    
    let mut original_anchor_metrics: Option<Rect> = None; // Store before bounding box adjustment
    
    
    // ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
    // │                                           MAIN LOOP                                            │
    // └────────────────────────────────────────────────────────────────────────────────────────────────┘ 
          
    loop {
        terminal.draw(|f| {
            let area = f.area();
            
            // Create dimming context based on popup state
            let dimming = DimmingContext::new(popup.is_some());
            
            
            // Render base layout
            let base_layout = BaseLayout::new(
                &config,
                None, // no tabs, so pass None 
                &dimming,
            );
            let result: BaseLayoutResult = base_layout.render(f, area, &mut registry);
            let content_area = result.content_area; // Get the content area
            
            // Store original anchor metrics after first render (if not already stored)
            // Note: This stores the unadjusted metrics from the first render
            if original_anchor_metrics.is_none() {
                if let Some(anchor_handle) = registry.get_handle(main_content_box_handle_name) {
                    if let Some(metrics) = registry.get_metrics(anchor_handle) {
                        original_anchor_metrics = Some(metrics.into());
                    }
                } else {
                    // If handle doesn't exist yet, store the content_area as original
                    original_anchor_metrics = Some(content_area);
                }
            }
            
            // Restore anchor position (HWND_MAIN_CONTENT_BOX) ---------------->> 
            // This clears any previous adjustments from the last frame
            if let (Some(anchor_handle), Some(original_metrics)) = (registry.get_handle(main_content_box_handle_name), original_anchor_metrics) {
                registry.update(anchor_handle, original_metrics);
            }
            
            //------------------------------------------------------------------------------------<<
            
            // Initialize or update main content bounding box (HWND_MAIN_CONTENT_BOX) with current content area
            // This is the same box that the tab bar uses as its anchor
            // If handle doesn't exist, create it; otherwise update it with current content_area
            if let Some(handle) = registry.get_handle(main_content_box_handle_name) {
                // Update existing bounding box with current content area
                registry.update(handle, content_area);
            } else {
                // Create new bounding box
                let _handle = registry.register(Some(main_content_box_handle_name), content_area);
            }

            // Prepare tab bar ------------------------------------------------>> 
            // This will adjust the anchor box (y+1, height-1) for Tab style
            // The adjustment happens in from_registry() AFTER the box has been updated with content_area
            // This ensures the anchor box is updated with current size, then adjusted for Tab style
            let tab_bar_result = main_content_tab_bar.prepare(&mut registry, Some(tab_style));
            
            //------------------------------------------------------------------------------------<<
            
            // Render content block ------------------------------------------->> 
            // Uses the adjusted anchor (if Tab style was applied)
            // The anchor box has been adjusted by prepare(), so content renders with correct position
            let render_area = if let Some(box_manager) = get_box_by_name(&registry, main_content_box_handle_name) {
                box_manager.prepare(&mut registry).unwrap_or(content_area)
            } else {
                content_area
            };
            
            // Always render the content box border first
            render_content(f, render_area, &dimming);
            
            // Check if diff tab is active and render split diff view
            if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                    if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                        if tab_config.id == "diff" {
                            // Create nested area for diff view (x+1, y+1, width-2, height-2 to account for borders)
                            let nested_area = Rect {
                                x: render_area.x.saturating_add(1),
                                y: render_area.y.saturating_add(1),
                                width: render_area.width.saturating_sub(2),
                                height: render_area.height.saturating_sub(2),
                            };
                            
                            // Create or update diff view bounding box
                            let diff_view_box = if let Some(existing_box) = get_box_by_name(&registry, HWND_DIFF_VIEW) {
                                // Update existing box with nested area
                                existing_box.update(&mut registry, nested_area);
                                existing_box
                            } else {
                                // Create new bounding box for diff view
                                BoundingBox::create(&mut registry, HWND_DIFF_VIEW, nested_area)
                            };
                            
                            // Load files for comparison
                            let file1_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("file1.md");
                            let file2_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("file2.md");
                            
                            let source_lines: Vec<String> = fs::read_to_string(&file1_path)
                                .unwrap_or_else(|_| "Error loading file1.md".to_string())
                                .lines()
                                .map(|l| l.to_string())
                                .collect();
                            
                            let dest_lines: Vec<String> = fs::read_to_string(&file2_path)
                                .unwrap_or_else(|_| "Error loading file2.md".to_string())
                                .lines()
                                .map(|l| l.to_string())
                                .collect();
                            
                            let diff_config = SplitDiffViewConfig::new()
                                .with_source_title(format!("Source: {}", file1_path.file_name().unwrap_or_default().to_string_lossy()))
                                .with_dest_title(format!("Destination: {}", file2_path.file_name().unwrap_or_default().to_string_lossy()));
                            
                            let mut diff_view = SplitDiffView::new(
                                &diff_config,
                                &mut diff_view_state,
                                &source_lines,
                                &dest_lines,
                            );
                            
                            // Render diff view using its own bounding box (nested inside content box)
                            if let Err(e) = diff_view.render(f, &diff_view_box, &mut registry) {
                                // Error rendering diff view - could log or show error message
                                eprintln!("Error rendering diff view: {}", e);
                            }
                        }
                        // Note: Other tabs don't need special handling - content box is already rendered
                    }
                }
            }

            //------------------------------------------------------------------------------------<<
            
            // Render tab bar ------------------------------------------------->> 
            if let Some((tab_bar, anchor_handle, tab_bar_state)) = tab_bar_result {
                tab_bar.render_with_state(f, &mut registry, &tab_bar_state, Some(&dimming));
                
                // Store tab bar for click detection
                current_tab_bar = Some((tab_bar, anchor_handle));
            }
            
            //------------------------------------------------------------------------------------<<
            
            
            
            
            
            
            
            
            
            
            if let Some(ref popup) = popup { //> Render popup if active
                render_popup(f, area, popup);
            } //<
            
            //> Filter out expired toasts (older than 1.5 seconds)
            let now = std::time::SystemTime::now();
            toasts.retain(|toast| {
                if let Ok(duration) = now.duration_since(toast.shown_at) {
                    duration.as_secs_f64() < 1.5
                } else {
                    false // Remove if time calculation fails
                }
            });
            
            //<<----------------------------------------------------------------------
            
            render_toasts(f, area, &toasts); // Render toasts (stacked in bottom-left)
        })?;
        
        
        // ┌──────────────────────────────────────────────────────────────────────────────────────────────┐
        // │                              Handle events (keyboard and mouse)                              │
        // └──────────────────────────────────────────────────────────────────────────────────────────────┘                
        
        // use non-blocking poll to allow toast timeout checking
        match crossterm::event::poll(std::time::Duration::from_millis(50)) {
            Ok(true) => {
                // Event available, read it
                match event::read()? {
                    Event::Key(key) => {
                        // Only process key press events to avoid key repeats
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
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
                    // Debouncing is handled by the component's navigate_tab method
                    if tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic {
                        main_content_tab_bar.navigate_previous(&mut registry); // Navigate left/previous
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    // Navigate tabs (only for non-static styles)
                    // Debouncing is handled by the component's navigate_tab method
                    if tab_style != TabBarStyle::BoxStatic && tab_style != TabBarStyle::TextStatic {
                        main_content_tab_bar.navigate_next(&mut registry); // Navigate right/next
                    }
                }
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    // Toggle fold unchanged regions in diff view
                    if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                        if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                            if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                                if tab_config.id == "diff" {
                                    diff_view_state.fold_unchanged = !diff_view_state.fold_unchanged;
                                    diff_view_state.scroll_offset = 0; // Reset scroll when toggling fold
                                }
                            }
                        }
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    // Scroll diff view up
                    if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                        if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                            if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                                if tab_config.id == "diff" && diff_view_state.scroll_offset > 0 {
                                    diff_view_state.scroll_offset -= 1;
                                }
                            }
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Scroll diff view down
                    if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                        if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                            if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                                if tab_config.id == "diff" {
                                    diff_view_state.scroll_offset += 1;
                                }
                            }
                        }
                    }
                }
                KeyCode::PageUp => {
                    // Scroll diff view up by page
                    if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                        if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                            if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                                if tab_config.id == "diff" && diff_view_state.scroll_offset > 0 {
                                    diff_view_state.scroll_offset = diff_view_state.scroll_offset.saturating_sub(10);
                                }
                            }
                        }
                    }
                }
                KeyCode::PageDown => {
                    // Scroll diff view down by page
                    if let Some(active_tab_idx) = registry.get_active_tab(main_content_tab_bar.handle()) {
                        if let Some(tab_bar_state) = registry.get_tab_bar_state(main_content_tab_bar.handle()) {
                            if let Some(tab_config) = tab_bar_state.tab_configs.get(active_tab_idx) {
                                if tab_config.id == "diff" {
                                    diff_view_state.scroll_offset += 10;
                                }
                            }
                        }
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
                                main_content_tab_bar.set_active(&mut registry, clicked_tab_idx);
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


