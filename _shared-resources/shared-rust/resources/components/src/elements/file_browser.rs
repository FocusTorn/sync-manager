// File browser component
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub visible_height: usize,
}

impl FileBrowser {
    pub fn new(initial_path: PathBuf) -> Self {
        let mut browser = Self {
            current_dir: initial_path.clone(),
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            visible_height: 0,
        };
        browser.load_directory();
        browser
    }

    pub fn load_directory(&mut self) {
        self.entries.clear();
        
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            
            for entry in entries.flatten() {
                let path = entry.path();
                let metadata = entry.metadata().ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let is_dir = metadata.map(|m| m.is_dir()).unwrap_or(false);
                
                let file_entry = FileEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: path.clone(),
                    is_dir,
                    size,
                };
                
                if is_dir {
                    dirs.push(file_entry);
                } else {
                    files.push(file_entry);
                }
            }
            
            // Sort: directories first, then files, both alphabetically
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));
            
            self.entries = dirs;
            self.entries.append(&mut files);
            
            // Maintain selection context when navigating
            self.adjust_selection_after_load();
        }
        
        self.selected_index = self.selected_index.min(self.entries.len().saturating_sub(1));
        self.adjust_scroll_to_selection();
    }

    fn adjust_selection_after_load(&mut self) {
        // When navigating to parent, maintain selection context
        // This is a placeholder - full implementation would track previous selection
    }

    pub fn navigate_into(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                let current_name = entry.name.clone();
                self.current_dir = entry.path.clone();
                self.load_directory();
                
                // Find and select the directory we just came from (if navigating back)
                for (i, e) in self.entries.iter().enumerate() {
                    if e.name == current_name {
                        self.selected_index = i;
                        self.adjust_scroll_to_selection();
                        break;
                    }
                }
            }
        }
    }

    pub fn navigate_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let current_name = self.current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            
            self.current_dir = parent.to_path_buf();
            self.load_directory();
            
            // Find and select the directory we just came from
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.name == current_name {
                    self.selected_index = i;
                    self.adjust_scroll_to_selection();
                    break;
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll_to_selection();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
            self.adjust_scroll_to_selection();
        }
    }

    fn adjust_scroll_to_selection(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_height {
            self.scroll_offset = self.selected_index.saturating_sub(self.visible_height - 1);
        }
    }

    pub fn render(
        &mut self,
        f: &mut Frame,
        area: Rect,
        title: &str,
        is_active: bool,
        modal_visible: bool,
    ) {
        use crate::utilities::{get_border_style, get_selection_style, get_selection_style_modal, get_text_color};
        use ratatui::widgets::ListState;
        
        self.visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        
        let (border_style, border_type) = get_border_style(is_active, modal_visible);
        let text_color = get_text_color(is_active, modal_visible);
        
        // Build list items
        let visible_entries: Vec<_> = self.entries
            .iter()
            .skip(self.scroll_offset)
            .take(self.visible_height)
            .collect();
        
        let items: Vec<ListItem> = if visible_entries.is_empty() {
            vec![ListItem::new(" No files")
                .style(Style::default().fg(Color::DarkGray))]
        } else {
            visible_entries.iter().map(|entry| {
                let prefix = if entry.is_dir { "📁 " } else { "📄 " };
                let display = format!("{}{}", prefix, entry.name);
                ListItem::new(display).style(Style::default().fg(text_color))
            }).collect()
        };
        
        let highlight_style = if modal_visible {
            get_selection_style_modal()
        } else {
            get_selection_style(is_active)
        };
        
        let list = List::new(items)
            .block(Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(border_style))
            .highlight_style(highlight_style);
        
        // Create stateful list state
        let mut state = ListState::default();
        let relative_index = self.selected_index.saturating_sub(self.scroll_offset);
        state.select(Some(relative_index));
        
        f.render_stateful_widget(list, area, &mut state);
        
        // Render scrollbar if needed
        if self.entries.len() > self.visible_height {
            self.render_scrollbar(f, area);
        }
    }

    fn render_scrollbar(&self, f: &mut Frame, area: Rect) {
        use crate::utilities::hex_color;
        
        let scrollbar_x = area.x + area.width - 1;
        let scrollbar_height = area.height.saturating_sub(2);
        let total_items = self.entries.len();
        
        if total_items == 0 {
            return;
        }
        
        let scrollbar_position = (self.scroll_offset * scrollbar_height as usize) / total_items;
        let scrollbar_size = ((scrollbar_height as usize * scrollbar_height as usize) / total_items.max(1)).max(1);
        
        for i in 0..scrollbar_height {
            let y = area.y + 1 + i;
            let is_scrollbar = (i as usize) >= scrollbar_position && (i as usize) < (scrollbar_position + scrollbar_size);
            let symbol = if is_scrollbar { "█" } else { "│" };
            let color = if is_scrollbar { Color::Cyan } else { hex_color(0x333333) };
            
            let scrollbar_widget = Paragraph::new(symbol)
                .style(Style::default().fg(color));
            
            f.render_widget(scrollbar_widget, Rect {
                x: scrollbar_x,
                y,
                width: 1,
                height: 1,
            });
        }
    }
}
