// Tab Bar Styles Comparison Example
// Shows both Tab and Text styles side by side without bounding boxes
// 
// Usage:
//   cargo run --bin tab-bar-styles-example --package component-examples
//   cargo run --bin tab-bar-styles-example --package component-examples -- Tab1 Tab2 Tab3 Tab4

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Paragraph,
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
            
            // Create vertical layout: Tab, Boxed, Text, BoxStatic, TextStatic styles, instructions
            let chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Percentage(15),  // Top spacer
                    Constraint::Length(3),       // Tab style area (with top decorative line)
                    Constraint::Length(1),       // Spacer
                    Constraint::Length(1),       // Boxed style area
                    Constraint::Length(1),       // Spacer
                    Constraint::Length(1),       // Text style area
                    Constraint::Length(1),       // Spacer
                    Constraint::Length(1),       // BoxStatic style area
                    Constraint::Length(1),       // Spacer
                    Constraint::Length(1),       // TextStatic style area
                    Constraint::Percentage(15),  // Middle spacer
                    Constraint::Length(3),       // Instructions
                ])
                .split(area);
            
            // Create horizontal layout for centering tab bars
            let tab_style_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10),  // Left spacer
                    Constraint::Percentage(80),  // Tab bar area
                    Constraint::Percentage(10),  // Right spacer
                ])
                .split(chunks[1]);
            
            let boxed_style_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10),  // Left spacer
                    Constraint::Percentage(80),  // Tab bar area
                    Constraint::Percentage(10),  // Right spacer
                ])
                .split(chunks[3]);
            
            let text_style_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10),  // Left spacer
                    Constraint::Percentage(80),  // Tab bar area
                    Constraint::Percentage(10),  // Right spacer
                ])
                .split(chunks[5]);
            
            let box_static_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10),  // Left spacer
                    Constraint::Percentage(80),  // Tab bar area
                    Constraint::Percentage(10),  // Right spacer
                ])
                .split(chunks[7]);
            
            let text_static_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10),  // Left spacer
                    Constraint::Percentage(80),  // Tab bar area
                    Constraint::Percentage(10),  // Right spacer
                ])
                .split(chunks[9]);
            
            // Tab style area (no bounding box, just the tab bar)
            let tab_style_area = Rect {
                x: tab_style_chunks[1].x,
                y: tab_style_chunks[1].y,
                width: tab_style_chunks[1].width,
                height: tab_style_chunks[1].height,
            };
            
            // Boxed style area (no bounding box, just the tab bar)
            let boxed_style_area = Rect {
                x: boxed_style_chunks[1].x,
                y: boxed_style_chunks[1].y,
                width: boxed_style_chunks[1].width,
                height: boxed_style_chunks[1].height,
            };
            
            // Text style area (no bounding box, just the tab bar)
            let text_style_area = Rect {
                x: text_style_chunks[1].x,
                y: text_style_chunks[1].y,
                width: text_style_chunks[1].width,
                height: text_style_chunks[1].height,
            };
            
            // BoxStatic style area (no bounding box, just the tab bar)
            let box_static_area = Rect {
                x: box_static_chunks[1].x,
                y: box_static_chunks[1].y,
                width: box_static_chunks[1].width,
                height: box_static_chunks[1].height,
            };
            
            // TextStatic style area (no bounding box, just the tab bar)
            let text_static_area = Rect {
                x: text_static_chunks[1].x,
                y: text_static_chunks[1].y,
                width: text_static_chunks[1].width,
                height: text_static_chunks[1].height,
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
            let tab_bar_tab = TabBar::new(
                tab_items.clone(),
                TabBarStyle::Tab,
                TabBarAlignment::Center,
            )
            .with_color(ratatui::style::Color::Cyan) // Same color as header text
            .with_position(TabBarPosition::Coords {
                x1: tab_style_area.x,
                x2: tab_style_area.x + tab_style_area.width,
                y: tab_style_area.y + 1, // Position in middle of 3-line area (for top decorative line)
            });
            tab_bar_tab.render(f);
            
            // Render Boxed style (square brackets around active tab)
            let tab_bar_boxed = TabBar::new(
                tab_items.clone(),
                TabBarStyle::Boxed,
                TabBarAlignment::Center,
            )
            .with_color(ratatui::style::Color::Green) // Same color as header text
            .with_position(TabBarPosition::Coords {
                x1: boxed_style_area.x,
                x2: boxed_style_area.x + boxed_style_area.width,
                y: boxed_style_area.y,
            });
            tab_bar_boxed.render(f);
            
            // Render Text style (plain text with separators)
            let tab_bar_text = TabBar::new(
                tab_items.clone(),
                TabBarStyle::Text,
                TabBarAlignment::Center,
            )
            .with_color(ratatui::style::Color::Yellow) // Same color as header text
            .with_position(TabBarPosition::Coords {
                x1: text_style_area.x,
                x2: text_style_area.x + text_style_area.width,
                y: text_style_area.y,
            });
            tab_bar_text.render(f);
            
            // Render BoxStatic style (all tabs in brackets, no active state)
            let tab_bar_box_static = TabBar::new(
                tab_items.clone(),
                TabBarStyle::BoxStatic,
                TabBarAlignment::Center,
            )
            .with_position(TabBarPosition::Coords {
                x1: box_static_area.x,
                x2: box_static_area.x + box_static_area.width,
                y: box_static_area.y,
            });
            tab_bar_box_static.render(f);
            
            // Render TextStatic style (all tabs as plain text, no active state)
            let tab_bar_text_static = TabBar::new(
                tab_items,
                TabBarStyle::TextStatic,
                TabBarAlignment::Center,
            )
            .with_position(TabBarPosition::Coords {
                x1: text_static_area.x,
                x2: text_static_area.x + text_static_area.width,
                y: text_static_area.y,
            });
            tab_bar_text_static.render(f);
            
            // Style labels (positioned one line above the tab bars)
            let tab_label = Paragraph::new(Line::from("Tab Style (with decorative brackets)"))
                .style(Style::default().fg(ratatui::style::Color::Cyan));
            f.render_widget(tab_label, Rect {
                x: tab_style_chunks[1].x,
                y: tab_style_chunks[1].y.saturating_sub(1),
                width: tab_style_chunks[1].width,
                height: 1,
            });
            
            let boxed_label = Paragraph::new(Line::from("Boxed Style (square brackets)"))
                .style(Style::default().fg(ratatui::style::Color::Green));
            f.render_widget(boxed_label, Rect {
                x: boxed_style_chunks[1].x,
                y: boxed_style_chunks[1].y.saturating_sub(1),
                width: boxed_style_chunks[1].width,
                height: 1,
            });
            
            let text_label = Paragraph::new(Line::from("Text Style (plain text with separators)"))
                .style(Style::default().fg(ratatui::style::Color::Yellow));
            f.render_widget(text_label, Rect {
                x: text_style_chunks[1].x,
                y: text_style_chunks[1].y.saturating_sub(1),
                width: text_style_chunks[1].width,
                height: 1,
            });
            
            let box_static_label = Paragraph::new(Line::from("BoxStatic Style (all tabs boxed, no active state)"))
                .style(Style::default().fg(ratatui::style::Color::Magenta));
            f.render_widget(box_static_label, Rect {
                x: box_static_chunks[1].x,
                y: box_static_chunks[1].y.saturating_sub(1),
                width: box_static_chunks[1].width,
                height: 1,
            });
            
            let text_static_label = Paragraph::new(Line::from("TextStatic Style (all tabs plain, no active state)"))
                .style(Style::default().fg(ratatui::style::Color::Blue));
            f.render_widget(text_static_label, Rect {
                x: text_static_chunks[1].x,
                y: text_static_chunks[1].y.saturating_sub(1),
                width: text_static_chunks[1].width,
                height: 1,
            });
            
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
            .style(Style::default());
            f.render_widget(instructions, chunks[11]);
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

