// Tab Bar Positioning Demo
// Demonstrates TopOf/BottomOf positioning with Left/Center/Right alignment and offsets
//
// Usage:
//   cargo run --bin tab-bar-positioning-example --package component-examples -- Tab1 Tab2 Tab3 80 tabbed topOf left
//   cargo run --bin tab-bar-positioning-example --package component-examples -- Tab1 Tab2 Tab3 80 boxed topOf center 3
//   cargo run --bin tab-bar-positioning-example --package component-examples -- Tab1 Tab2 Tab3 80 text bottomOf right -4

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use std::io;
use tui_components::{TabBar, TabBarItem, TabBarStyle, TabBarAlignment, TabBarPosition, RectRegistry};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn parse_args() -> Result<(Vec<String>, u16, TabBarStyle, TabBarPosition, TabBarAlignment, i16, Option<String>), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    if args.len() < 5 {
        return Err(format!(
            "Usage: <tab1> <tab2> ... <width> <position> <alignment> [offset] <style> [id]\n\
            \n\
            Examples:\n\
              Tab1 Tab2 Tab3 80 topOf left tabbed\n\
              Tab1 Tab2 Tab3 80 topOf left 3 boxed\n\
              Tab1 Tab2 Tab3 80 topOf left -4 text\n\
              Tab1 Tab2 Tab3 80 topOf left tabbed my-box\n\
            \n\
            Styles: tabbed, boxed, text, boxStatic, textStatic\n\
            Positions: topOf, bottomOf\n\
            Alignments: left, center, right\n\
            Offset: optional integer (default: 0)\n\
            ID: optional identifier for bounding box (enables handle-based positioning)"
        ));
    }
    
    // Find where the width argument is (it's a number)
    let mut width_idx = None;
    for (idx, arg) in args.iter().enumerate() {
        if arg.parse::<u16>().is_ok() {
            width_idx = Some(idx);
            break;
        }
    }
    
    let width_idx = width_idx.ok_or("Width argument not found")?;
    
    // Extract tab names (everything before width)
    let tab_names: Vec<String> = args[..width_idx].iter().cloned().collect();
    if tab_names.is_empty() {
        return Err("At least one tab name is required".to_string());
    }
    
    // Extract width
    let width: u16 = args[width_idx].parse()
        .map_err(|_| format!("Invalid width: {}", args[width_idx]))?;
    
    // Extract remaining arguments: <position> <alignment> [offset] <style>
    let remaining = &args[width_idx + 1..];
    if remaining.len() < 3 {
        return Err("Missing position, alignment, or style arguments".to_string());
    }
    
    // Parse position (first argument after width)
    let position_str = remaining[0].to_lowercase();
    let is_top = match position_str.as_str() {
        "topof" | "top" => true,
        "bottomof" | "bottom" => false,
        _ => return Err(format!("Invalid position: {}. Use: topOf or bottomOf", remaining[0])),
    };
    
    // Parse alignment (second argument after width)
    let alignment = match remaining[1].to_lowercase().as_str() {
        "left" => TabBarAlignment::Left,
        "center" => TabBarAlignment::Center,
        "right" => TabBarAlignment::Right,
        _ => return Err(format!("Invalid alignment: {}. Use: left, center, or right", remaining[1])),
    };
    
    // Parse style, offset, and optional ID
    // Format: <position> <alignment> [offset] <style> [id]
    let (offset, style_str, id) = if remaining.len() == 3 {
        // No offset, no ID: <position> <alignment> <style>
        (0, remaining[2].as_str(), None)
    } else if remaining.len() == 4 {
        // Either offset or ID: check if 3rd arg is a number
        if remaining[2].parse::<i16>().is_ok() {
            // Has offset, no ID: <position> <alignment> <offset> <style>
            let offset_val: i16 = remaining[2].parse()
                .map_err(|_| format!("Invalid offset: {}", remaining[2]))?;
            (offset_val, remaining[3].as_str(), None)
        } else {
            // No offset, has ID: <position> <alignment> <style> <id>
            (0, remaining[2].as_str(), Some(remaining[3].clone()))
        }
    } else if remaining.len() == 5 {
        // Has both offset and ID: <position> <alignment> <offset> <style> <id>
        let offset_val: i16 = remaining[2].parse()
            .map_err(|_| format!("Invalid offset: {}", remaining[2]))?;
        (offset_val, remaining[3].as_str(), Some(remaining[4].clone()))
    } else {
        return Err("Too many arguments after width".to_string());
    };
    
    // Parse style
    let style = match style_str.to_lowercase().as_str() {
        "tabbed" | "tab" => TabBarStyle::Tab,
        "boxed" | "box" => TabBarStyle::Boxed,
        "text" => TabBarStyle::Text,
        "boxstatic" | "box-static" => TabBarStyle::BoxStatic,
        "textstatic" | "text-static" => TabBarStyle::TextStatic,
        _ => return Err(format!("Invalid style: {}. Use: tabbed, boxed, text, boxStatic, textStatic", style_str)),
    };
    
    // Create position (we'll adjust for offset and handle in the render function)
    // If ID is provided, we'll use handle-based positioning
    let position = if is_top {
        TabBarPosition::TopOf(Rect { x: 0, y: 0, width, height: 5 })
    } else {
        TabBarPosition::BottomOf(Rect { x: 0, y: 0, width, height: 5 })
    };
    
    Ok((tab_names, width, style, position, alignment, offset, id))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let (tab_names, width, style, position, alignment, offset, box_id) = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    
    // Create registry for handle-based positioning (if ID is provided)
    let mut registry = RectRegistry::new();
    
    // Track active tab (only for non-static styles)
    let mut active_tab_index = 0;
    let supports_navigation = style != TabBarStyle::BoxStatic && style != TabBarStyle::TextStatic;
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Render loop
    loop {
        terminal.draw(|f| {
            let area = f.area();
            
            // Center the bounding box on screen
            let box_width = width.min(area.width.saturating_sub(4));
            let box_height = 5;
            let box_x = (area.width.saturating_sub(box_width)) / 2;
            let box_y = (area.height.saturating_sub(box_height)) / 2;
            
            // Create bounding box rect (adjust x for offset)
            let bounding_box = Rect {
                x: (box_x as i16 + offset).max(0) as u16,
                y: box_y,
                width: box_width,
                height: box_height,
            };
            
            // Register/update bounding box in registry if ID provided (updates each frame for resize handling)
            let handle_opt = if let Some(id) = &box_id {
                Some(registry.register(Some(id), bounding_box))
            } else {
                None
            };
            
            // Get current HWND metrics if handle exists
            let hwnd_info = if let Some(handle) = handle_opt {
                if let Some(metrics) = registry.get_metrics(handle) {
                    Some(format!("ID: {} | HWND: x:{}, y:{}, w:{}, h:{}", 
                        box_id.as_ref().unwrap(), metrics.x, metrics.y, metrics.width, metrics.height))
                } else {
                    None
                }
            } else {
                None
            };
            
            // Create handle-based position if ID provided
            let adjusted_position = if let Some(handle) = handle_opt {
                // Create handle-based position
                match &position {
                    TabBarPosition::TopOf(_) => TabBarPosition::TopOfHandle(handle),
                    TabBarPosition::BottomOf(_) => TabBarPosition::BottomOfHandle(handle),
                    _ => position.clone(),
                }
            } else {
                // No ID, use direct rect positioning
                match &position {
                    TabBarPosition::TopOf(_) => TabBarPosition::TopOf(bounding_box),
                    TabBarPosition::BottomOf(_) => TabBarPosition::BottomOf(bounding_box),
                    _ => position.clone(),
                }
            };
            
            // Render box dimensions 3 rows above the box (centered)
            // Show ID and HWND metrics if available
            if bounding_box.y >= 3 {
                let dimensions_str = if let Some(hwnd) = &hwnd_info {
                    hwnd.clone()
                } else {
                    format!("{}x{}", width, box_height)
                };
                let dim_width = dimensions_str.len() as u16;
                let dim_x = bounding_box.x + (bounding_box.width.saturating_sub(dim_width.min(bounding_box.width))) / 2;
                let dimensions_text = Paragraph::new(Line::from(dimensions_str))
                    .style(Style::default().fg(ratatui::style::Color::Yellow));
                f.render_widget(dimensions_text, Rect {
                    x: dim_x,
                    y: bounding_box.y.saturating_sub(3),
                    width: dim_width.min(bounding_box.width),
                    height: 1,
                });
            }
            
            // Render bounding box first (so tab bar can render on top of it)
            let block = Block::default()
                .borders(Borders::ALL);
            f.render_widget(block, bounding_box);
            
            // Create tab items (use current active_tab_index for non-static styles)
            let tab_items: Vec<TabBarItem> = tab_names
                .iter()
                .enumerate()
                .map(|(idx, name)| TabBarItem {
                    name: name.clone(),
                    active: idx == active_tab_index && supports_navigation,
                })
                .collect();
            
            // Format position info before moving adjusted_position
            let position_info = if let Some(id) = &box_id {
                format!("Position: {:?} (Handle-based, ID: {}) | Alignment: {:?} | Offset: {} | Style: {:?}", 
                    &adjusted_position, id, alignment, offset, style)
            } else {
                format!("Position: {:?} | Alignment: {:?} | Offset: {} | Style: {:?}", 
                    &adjusted_position, alignment, offset, style)
            };
            
            // Create and render tab bar (after block so it appears on top of border)
            let tab_bar = TabBar::new(tab_items, style, alignment)
                .with_color(ratatui::style::Color::Cyan)
                .with_position(adjusted_position);
            
            // Use render_with_registry if ID provided (for handle-based positioning)
            if box_id.is_some() {
                tab_bar.render_with_registry(f, Some(&mut registry), None);
            } else {
                tab_bar.render(f);
            }
            
            // Instructions
            
            let hwnd_line = if let Some(hwnd) = &hwnd_info {
                Line::from(hwnd.clone())
            } else {
                Line::from("No HWND (bounding box not registered with ID)")
            };
            
            let nav_hint = if supports_navigation {
                " | Arrow keys/h/j/l to navigate tabs | Number keys (1-9) to jump to tab"
            } else {
                " | (Static style - no navigation)"
            };
            
            let instructions = Paragraph::new(vec![
                Line::from(position_info),
                hwnd_line,
                Line::from(format!("Press 'q' to quit{} | Resize terminal to see HWND update", nav_hint)),
            ])
            .style(Style::default());
            f.render_widget(instructions, Rect {
                x: 0,
                y: area.height.saturating_sub(3),
                width: area.width,
                height: 3,
            });
        })?;
        
        // Handle keyboard input (blocking read like other examples)
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    break;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    // Move to previous tab (only for non-static styles)
                    if supports_navigation {
                        active_tab_index = if active_tab_index > 0 {
                            active_tab_index - 1
                        } else {
                            tab_names.len() - 1 // Wrap to last tab
                        };
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    // Move to next tab (only for non-static styles)
                    if supports_navigation {
                        active_tab_index = (active_tab_index + 1) % tab_names.len();
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    // Handle number keys (1-9) to jump to specific tab
                    if supports_navigation {
                        if let Some(digit) = c.to_digit(10) {
                            let tab_num = digit as usize;
                            if tab_num >= 1 && tab_num <= tab_names.len() {
                                active_tab_index = tab_num - 1; // Convert to 0-based index
                            }
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

