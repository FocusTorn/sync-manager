// Tab Bar Component Example
// Standalone example showing the tab bar component output
// 
// Expected output:
//                        ╭───────────╮  
// ── DASHBOARD ─ CHANGES ─╯ BASELINES ╰──
//
// This shows the Tab style with BASELINES as the active tab

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use std::io;
use tui_components::{TabBar, TabBarItem, TabBarStyle, TabBarAlignment, TabBarPosition};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments for tab names
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    // Default tab names if no arguments provided
    let default_tabs = vec!["DASHBOARD", "CHANGES", "BASELINES"];
    let tab_names: Vec<String> = if args.is_empty() {
        default_tabs.iter().map(|s| s.to_string()).collect()
    } else {
        args
    };
    
    if tab_names.is_empty() {
        eprintln!("Error: At least one tab name is required");
        std::process::exit(1);
    }
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // State: which tab is active (default to last tab, or first if only one tab)
    let mut active_tab_index = if tab_names.len() > 1 {
        tab_names.len() - 1 // Start with last tab active
    } else {
        0
    };
    
    // Render loop
    loop {
        terminal.draw(|f| {
            let area = f.area();
            
            // Create a centered box to show the tab bar
            let chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),  // Top spacer
                    Constraint::Length(5),       // Tab bar box
                    Constraint::Percentage(40),  // Bottom spacer
                    Constraint::Length(3),       // Instructions
                ])
                .split(area);
            
            // Create a centered horizontal layout within the tab bar box
            let tab_box_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),  // Left spacer
                    Constraint::Percentage(60),  // Tab bar area
                    Constraint::Percentage(20),  // Right spacer
                ])
                .split(chunks[1]);
            
            // Tab bar container block
            let tab_bar_container = Rect {
                x: tab_box_chunks[1].x,
                y: chunks[1].y,
                width: tab_box_chunks[1].width,
                height: chunks[1].height,
            };
            
            let tab_bar_block = Block::default()
                .borders(Borders::ALL)
                .title(" Tab Bar Component (Tab Style) ");
            f.render_widget(tab_bar_block, tab_bar_container);
            
            // Inner area for tab bar (accounting for border)
            let tab_bar_area = Rect {
                x: tab_bar_container.x + 1,
                y: tab_bar_container.y + 1,
                width: tab_bar_container.width.saturating_sub(2),
                height: tab_bar_container.height.saturating_sub(2),
            };
            
            // Create tabs from command-line arguments or defaults
            let tab_items: Vec<TabBarItem> = tab_names
                .iter()
                .enumerate()
                .map(|(idx, name)| TabBarItem {
                    name: name.clone(),
                    active: active_tab_index == idx,
                })
                .collect();
            
            // Render Tab style (shows the decorative brackets and top line)
            let tab_bar = TabBar::new(
                tab_items,
                TabBarStyle::Tab,
                TabBarAlignment::Center,
            )
            .with_position(TabBarPosition::TopOf(tab_bar_area));
            tab_bar.render(f);
            
            // Instructions
            let active_tab_name = tab_names
                .get(active_tab_index)
                .map(|s| s.as_str())
                .unwrap_or("UNKNOWN");
            
            // Build keyboard shortcut hint
            let key_hints: Vec<String> = (1..=tab_names.len())
                .map(|i| i.to_string())
                .collect();
            let key_hint = if key_hints.len() <= 9 {
                format!("Press ↑/↓ or {} to change active tab | Press 'q' to quit", 
                    key_hints.join("/"))
            } else {
                "Press ↑/↓ to change active tab | Press 'q' to quit".to_string()
            };
            
            let instructions = Paragraph::new(vec![
                Line::from(key_hint),
                Line::from(format!("Active tab: {} ({}/{})", active_tab_name, active_tab_index + 1, tab_names.len())),
            ])
            .block(Block::default().borders(Borders::ALL).title(" Controls "))
            .style(Style::default());
            f.render_widget(instructions, chunks[3]);
        })?;
        
        // Handle keyboard input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    // Move to previous tab
                    active_tab_index = if active_tab_index > 0 {
                        active_tab_index - 1
                    } else {
                        tab_names.len() - 1 // Wrap to last tab
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Move to next tab
                    active_tab_index = (active_tab_index + 1) % tab_names.len();
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    // Handle number keys (1-9) to jump to specific tab
                    if let Some(digit) = c.to_digit(10) {
                        let tab_num = digit as usize;
                        if tab_num >= 1 && tab_num <= tab_names.len() {
                            active_tab_index = tab_num - 1; // Convert to 0-based index
                        }
                    }
                }
                _ => {}
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

