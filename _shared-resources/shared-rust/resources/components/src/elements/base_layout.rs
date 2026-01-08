// Base layout component for TUI applications
// Provides title header, tab bar, status bar, and global bindings
use crate::utilities::DimmingContext;
use crate::elements::tab_bar::{TabBar, TabBarItem, TabBarStyle, TabBarAlignment, TabBarPosition};
use crate::core::RectRegistry;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Configuration for the base layout UI elements
#[derive(Debug, Clone)]
pub struct BaseLayoutConfig {
    pub title: String,
    pub tabs: Vec<TabConfig>,
    pub global_bindings: Vec<BindingConfig>,
    pub status_bar: StatusBarConfig,
}

/// Configuration for a tab
#[derive(Debug, Clone)]
pub struct TabConfig {
    pub name: String,
    pub id: String, // Unique identifier for the tab
}

/// Configuration for a keyboard binding display
#[derive(Debug, Clone)]
pub struct BindingConfig {
    pub key: String,        // e.g., "[n]", "[Ctrl+C]"
    pub description: String, // e.g., "New Baseline"
}

/// Configuration for the status bar
#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    pub default_text: String,
    pub modal_text: Option<String>, // Text to show when modal is active
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            default_text: "Status: Ready | Press PgUp/PgDn to switch tabs | 'q' to quit".to_string(),
            modal_text: Some("Modal active - Use arrow keys to navigate, Enter to confirm, Esc to cancel".to_string()),
        }
    }
}

/// Result of rendering the base layout
/// Provides the content area where application-specific content should be rendered
#[derive(Debug, Clone)]
pub struct BaseLayoutResult {
    pub content_area: Rect, // The area where content panels should be rendered
}

/// Base layout component that renders the standard TUI frame structure
pub struct BaseLayout<'a> {
    config: &'a BaseLayoutConfig,
    active_tab_id: Option<&'a str>,
    dimming: &'a DimmingContext,
}

impl<'a> BaseLayout<'a> {
    pub fn new(
        config: &'a BaseLayoutConfig,
        active_tab_id: Option<&'a str>,
        dimming: &'a DimmingContext,
    ) -> Self {
        Self {
            config,
            active_tab_id,
            dimming,
        }
    }

    /// Render the base layout and return the content area
    /// Requires a RectRegistry to register all components with their HWND IDs
    pub fn render(&self, f: &mut Frame, area: Rect, registry: &mut RectRegistry) -> BaseLayoutResult {
        // Minimum terminal height needed: title(3) + gap(1) + main(1) + context(1) + footer(1) + status(1) = 8
        // If terminal is too small, we'll adjust but ensure no panic
        
        // Early return if terminal is too small to render anything safely
        if area.width == 0 || area.height < 6 {
            // Return a minimal safe rect
            let safe_rect = Rect {
                x: 0,
                y: 0,
                width: area.width,
                height: area.height.max(1),
            };
            registry.register(Some("hwndMainContainer"), safe_rect);
            return BaseLayoutResult { content_area: safe_rect };
        }
        
        // 1. Title Banner (hwndTitleBanner): x=0, y=0, h=3, w=100% - anchored to top at y=0
        let title_banner = Rect {
            x: 0,
            y: 0,
            width: area.width,
            height: 3.min(area.height), // Don't exceed terminal height
        };
        registry.register(Some("hwndTitleBanner"), title_banner);
        if title_banner.width > 0 && title_banner.height > 0 {
            self.render_title(f, title_banner);
        }

        // 2. Status Bar (hwndStatusBar): anchored to bottom left, w=100%, h=1, dim grey
        let status_bar = Rect {
            x: 0,
            y: area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        };
        registry.register(Some("hwndStatusBar"), status_bar);
        if status_bar.width > 0 && status_bar.height > 0 && status_bar.y < area.height {
            // Status bar will be rendered later
        }

        // 3. Footer Divider (hwndFooterDiv): x=0, y=hwndStatusBar.y-1, w=100%, h=1
        let footer_div = Rect {
            x: 0,
            y: status_bar.y.saturating_sub(1),
            width: area.width,
            height: 1,
        };
        registry.register(Some("hwndFooterDiv"), footer_div);
        if footer_div.width > 0 && footer_div.height > 0 && footer_div.y < area.height {
            self.render_footer_divider(f, footer_div);
        }

        // 4. Context Bindings (hwndContextBindings): x=0, y=hwndFooterDiv.y-1, w=100%, h=1
        let context_bindings = Rect {
            x: 0,
            y: footer_div.y.saturating_sub(1),
            width: area.width,
            height: 1,
        };
        registry.register(Some("hwndContextBindings"), context_bindings);
        if context_bindings.width > 0 && context_bindings.height > 0 && context_bindings.y < area.height {
            self.render_context_bindings(f, context_bindings);
        }

        // 5. Main Container (hwndMainContainer): x=0, y=hwndTitleBanner.y + hwndTitleBanner.h + 1 - 1, w=100%, h=hwndContextBindings.y - main_container.y
        // Title banner: y=0, height=3 (rows 0, 1, 2) - anchored to top at y=0
        // Single gap: row 3 (one row of spacing after title banner)
        // Main container starts at: row 3 (y=3) - moved y-1 from original y=4
        let calculated_y = title_banner.y + title_banner.height + 1 - 1; // 0 + 3 + 1 - 1 = 3 (moved up one row)
        
        // Ensure main_container_y doesn't exceed context_bindings.y (terminal too small case)
        // If context_bindings.y is 0 or very small, ensure we don't create invalid rect
        let main_container_y = if context_bindings.y > 0 {
            calculated_y.min(context_bindings.y.saturating_sub(1))
        } else {
            // Terminal extremely small - place at calculated position but ensure valid
            calculated_y.min(area.height.saturating_sub(1))
        };
        
        // Main container should be between main_container_y and context_bindings.y
        // But if context_bindings.y is too small, use area.height as fallback
        let main_container_height = if context_bindings.y > main_container_y {
            context_bindings.y - main_container_y
        } else if area.height > main_container_y {
            // Fallback: use remaining space to bottom of terminal
            area.height - main_container_y
        } else {
            // Terminal too small - ensure at least 1 row
            1
        };
        
        // Ensure all values are valid
        // Main container should be full width of terminal (x=0, width=area.width)
        let main_container = Rect {
            x: 0,
            y: main_container_y.min(area.height.saturating_sub(1)),
            width: area.width, // Full terminal width
            height: main_container_height.max(1).min(area.height.saturating_sub(main_container_y)),
        };
        registry.register(Some("hwndMainContainer"), main_container);

        // Render tab bar on top of main container's top border (if tabs exist)
        if !self.config.tabs.is_empty() && main_container.width > 0 && main_container.height > 0 {
            self.render_tab_bar(f, main_container);
        }

        // Render status bar (dim grey)
        if status_bar.width > 0 && status_bar.height > 0 && status_bar.y < area.height {
            self.render_status_bar(f, status_bar);
        }

        BaseLayoutResult { content_area: main_container }
    }

    /// Render the title header with borders
    fn render_title(&self, f: &mut Frame, area: Rect) {
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.dimming.border_color(true)));
        
        let title_text = format!(" {}", self.config.title);
        let title = Paragraph::new(Line::from(title_text))
            .block(title_block)
            .style(Style::default()
                .fg(self.dimming.text_color(true))
                .add_modifier(Modifier::BOLD))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(title, area);
    }

    /// Render the tab bar
    fn render_tab_bar(&self, f: &mut Frame, content_area: Rect) {
        let tab_items: Vec<TabBarItem> = self
            .config
            .tabs
            .iter()
            .map(|tab| TabBarItem {
                name: tab.name.clone(),
                active: self.active_tab_id.map_or(false, |id| id == tab.id),
                state: None,
            })
            .collect();

        // Position tab bar on top of the content area's top border
        let tab_bar = TabBar::new(tab_items, TabBarStyle::Tab, TabBarAlignment::Center)
            .with_position(TabBarPosition::TopOf(content_area));

        tab_bar.render(f);
    }

    /// Render the status bar (dim grey colored)
    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.dimming.modal_visible {
            self.config.status_bar.modal_text.as_deref()
                .unwrap_or(&self.config.status_bar.default_text)
        } else {
            &self.config.status_bar.default_text
        };

        // Dim grey color for status bar
        let dim_grey = Color::Rgb(0x44, 0x44, 0x44);
        let status = Paragraph::new(Line::from(status_text))
            .style(Style::default().fg(dim_grey));
        f.render_widget(status, area);
    }

    /// Render footer divider
    fn render_footer_divider(&self, f: &mut Frame, area: Rect) {
        // Prevent panic if width is 0
        if area.width == 0 || area.height == 0 {
            return;
        }
        let divider = Paragraph::new(Line::from("─".repeat(area.width as usize)))
            .style(Style::default().fg(self.dimming.border_color(false)));
        f.render_widget(divider, area);
    }

    /// Render context bindings (if applicable)
    fn render_context_bindings(&self, f: &mut Frame, area: Rect) {
        if !self.config.global_bindings.is_empty() {
            let mut spans = Vec::new();
            for (idx, binding) in self.config.global_bindings.iter().enumerate() {
                if idx > 0 {
                    spans.push(Span::styled(" | ", Style::default().fg(self.dimming.text_color(false))));
                }
                spans.push(Span::styled(
                    binding.key.clone(),
                    Style::default()
                        .fg(self.dimming.text_color(true))
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    format!(" {}", binding.description),
                    Style::default().fg(self.dimming.text_color(false)),
                ));
            }
            let bindings_text = vec![Line::from(spans)];
            let paragraph = Paragraph::new(bindings_text)
                .style(Style::default().fg(self.dimming.text_color(false)));
            f.render_widget(paragraph, area);
        }
    }
}

/// Render global bindings box
/// This can be called from within content views to show global keyboard shortcuts
pub fn render_global_bindings(
    f: &mut Frame,
    area: Rect,
    bindings: &[BindingConfig],
    dimming: &DimmingContext,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("─ Bindings ─")
        .title_alignment(ratatui::layout::Alignment::Left)
        .border_style(Style::default().fg(dimming.border_color(true)));

    // Build bindings text line
    let mut spans = Vec::new();
    for (idx, binding) in bindings.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(dimming.text_color(false))));
        }
        spans.push(Span::styled(
            binding.key.clone(),
            Style::default()
                .fg(dimming.text_color(true))
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", binding.description),
            Style::default().fg(dimming.text_color(false)),
        ));
    }

    let bindings_text = vec![Line::from(spans)];
    let paragraph = Paragraph::new(bindings_text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(paragraph, area);
}

