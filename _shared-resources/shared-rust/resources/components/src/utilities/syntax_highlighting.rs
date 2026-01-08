// Syntax highlighting module
// Provides syntax highlighting based on file extensions using Syntastica (tree-sitter)
// Requires the "syntax-highlighting" feature to be enabled

use ratatui::text::Span;

#[cfg(feature = "syntax-highlighting")]
use ratatui::style::{Color, Style};

#[cfg(feature = "syntax-highlighting")]
use syntastica::{process::Processor, renderer::TerminalRenderer};
#[cfg(feature = "syntax-highlighting")]
use syntastica_parsers::LanguageSetImpl;
#[cfg(feature = "syntax-highlighting")]
use syntastica_themes::gruvbox;

/// Syntax highlighting state (caches processor and renderer)
/// Only available when the "syntax-highlighting" feature is enabled
#[derive(Debug)]
pub struct SyntaxHighlighter {
    #[cfg(feature = "syntax-highlighting")]
    language_set: LanguageSetImpl,
    #[cfg(feature = "syntax-highlighting")]
    processor: Processor<'static>,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with default theme
    /// Requires the "syntax-highlighting" feature to be enabled
    #[cfg(feature = "syntax-highlighting")]
    pub fn new() -> Self {
        let language_set = LanguageSetImpl::new();
        let processor = Processor::new(&language_set);
        
        Self {
            language_set,
            processor,
        }
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    pub fn new() -> Self {
        Self {}
    }

    /// Create a new syntax highlighter with a specific theme
    /// Note: Syntastica themes are applied during rendering, not initialization
    /// This method currently uses the default theme
    #[cfg(feature = "syntax-highlighting")]
    pub fn with_theme(_theme_name: &str) -> Option<Self> {
        // Themes are applied during rendering, so initialization is the same
        Some(Self::new())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    pub fn with_theme(_theme_name: &str) -> Option<Self> {
        Some(Self::new())
    }

    /// Get language from file extension
    #[cfg(feature = "syntax-highlighting")]
    fn get_language_for_extension(&self, extension: &str) -> Option<&str> {
        // Remove leading dot if present
        let ext = extension.strip_prefix('.').unwrap_or(extension);
        
        // Map common extensions to language names (syntastica uses string names)
        match ext.to_lowercase().as_str() {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" | "jsx" => Some("javascript"),
            "ts" | "tsx" => Some("typescript"),
            "go" => Some("go"),
            "java" => Some("java"),
            "c" => Some("c"),
            "cpp" | "cc" | "cxx" => Some("cpp"),
            "cs" => Some("csharp"),
            "rb" => Some("ruby"),
            "php" => Some("php"),
            "swift" => Some("swift"),
            "kt" => Some("kotlin"),
            "scala" => Some("scala"),
            "sh" | "bash" => Some("bash"),
            "yaml" | "yml" => Some("yaml"),
            "json" => Some("json"),
            "toml" => Some("toml"),
            "xml" => Some("xml"),
            "html" => Some("html"),
            "css" => Some("css"),
            "sql" => Some("sql"),
            "md" | "markdown" => Some("markdown"),
            _ => None,
        }
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[allow(dead_code)]
    fn get_language_for_extension(&self, _extension: &str) -> Option<()> {
        None
    }

    /// Highlight a line of code and return ratatui Spans
    #[cfg(feature = "syntax-highlighting")]
    pub fn highlight_line(&self, line: &str, extension: &str) -> Vec<Span<'static>> {
        // If no language found, return plain text
        let Some(lang) = self.get_language_for_extension(extension) else {
            return vec![Span::raw(line.to_string())];
        };

        // Process the line to get highlights
        let highlights = match self.processor.process(line, lang) {
            Ok(highlights) => highlights,
            Err(_) => {
                // On error, return plain text
                return vec![Span::raw(line.to_string())];
            }
        };

        // Create renderer with theme
        let mut renderer = TerminalRenderer::new(gruvbox::dark());

        // Render highlights (render takes 2 args: highlights and renderer)
        let output = syntastica::render(&highlights, &mut renderer);

        // Parse the ANSI-colored output and convert to ratatui spans
        // Syntastica outputs ANSI escape codes, we need to parse them
        self.parse_ansi_to_spans(&output)
    }

    /// Parse ANSI escape codes from Syntastica output and convert to ratatui spans
    #[cfg(feature = "syntax-highlighting")]
    fn parse_ansi_to_spans(&self, ansi_text: &str) -> Vec<Span<'static>> {
        use regex::Regex;
        
        // Regex to match ANSI escape sequences
        let ansi_re = Regex::new(r"\x1b\[([0-9;]*?)m").unwrap();
        let mut spans = Vec::new();
        let mut last_end = 0;
        let mut current_style = Style::default();

        for cap in ansi_re.find_iter(ansi_text) {
            // Add text before this escape sequence
            if cap.start() > last_end {
                let text = &ansi_text[last_end..cap.start()];
                if !text.is_empty() {
                    spans.push(Span::styled(text.to_string(), current_style));
                }
            }

            // Parse the ANSI code
            let code = cap.as_str();
            if code == "\x1b[0m" || code == "\x1b[m" {
                // Reset
                current_style = Style::default();
            } else if code.starts_with("\x1b[38;2;") {
                // RGB color: \x1b[38;2;r;g;bm
                let rgb_part = &code[7..code.len()-1]; // Remove \x1b[38;2; and m
                let parts: Vec<&str> = rgb_part.split(';').collect();
                if parts.len() >= 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[0].parse::<u8>(),
                        parts[1].parse::<u8>(),
                        parts[2].parse::<u8>(),
                    ) {
                        current_style = current_style.fg(Color::Rgb(r, g, b));
                    }
                }
            } else if code.starts_with("\x1b[38;5;") {
                // 256-color: \x1b[38;5;nm
                let color_part = &code[7..code.len()-1];
                if let Ok(color_code) = color_part.parse::<u8>() {
                    // Convert 256-color code to RGB (simplified mapping)
                    current_style = current_style.fg(self.ansi256_to_rgb(color_code));
                }
            }

            last_end = cap.end();
        }

        // Add remaining text
        if last_end < ansi_text.len() {
            let text = &ansi_text[last_end..];
            if !text.is_empty() {
                spans.push(Span::styled(text.to_string(), current_style));
            }
        }

        // If no spans were created (no ANSI codes), return plain text
        if spans.is_empty() {
            // Remove ANSI codes and return plain text
            let clean_text = ansi_re.replace_all(ansi_text, "");
            spans.push(Span::raw(clean_text.to_string()));
        }

        spans
    }

    /// Convert ANSI 256-color code to RGB Color
    #[cfg(feature = "syntax-highlighting")]
    fn ansi256_to_rgb(&self, code: u8) -> Color {
        // Simplified 256-color to RGB mapping
        // For a full implementation, you'd need a complete color table
        if code < 16 {
            // Standard colors
            match code {
                0 => Color::Rgb(0, 0, 0),       // Black
                1 => Color::Rgb(128, 0, 0),     // Red
                2 => Color::Rgb(0, 128, 0),     // Green
                3 => Color::Rgb(128, 128, 0),   // Yellow
                4 => Color::Rgb(0, 0, 128),     // Blue
                5 => Color::Rgb(128, 0, 128),   // Magenta
                6 => Color::Rgb(0, 128, 128),   // Cyan
                7 => Color::Rgb(192, 192, 192), // White
                8 => Color::Rgb(128, 128, 128), // Bright Black
                9 => Color::Rgb(255, 0, 0),     // Bright Red
                10 => Color::Rgb(0, 255, 0),    // Bright Green
                11 => Color::Rgb(255, 255, 0),  // Bright Yellow
                12 => Color::Rgb(0, 0, 255),     // Bright Blue
                13 => Color::Rgb(255, 0, 255),  // Bright Magenta
                14 => Color::Rgb(0, 255, 255),  // Bright Cyan
                15 => Color::Rgb(255, 255, 255), // Bright White
                _ => Color::Reset,
            }
        } else if code < 232 {
            // 6x6x6 color cube
            let code = code - 16;
            let r = (code / 36) % 6;
            let g = (code / 6) % 6;
            let b = code % 6;
            Color::Rgb(
                if r == 0 { 0 } else { (r - 1) * 40 + 95 },
                if g == 0 { 0 } else { (g - 1) * 40 + 95 },
                if b == 0 { 0 } else { (b - 1) * 40 + 95 },
            )
        } else {
            // Grayscale
            let gray = (code - 232) * 10 + 8;
            Color::Rgb(gray, gray, gray)
        }
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    pub fn highlight_line(&self, line: &str, _extension: &str) -> Vec<Span<'static>> {
        // Return plain text when syntax highlighting is not available
        vec![Span::raw(line.to_string())]
    }

    /// Check if syntax highlighting is available for a given extension
    pub fn has_syntax_for_extension(&self, _extension: &str) -> bool {
        #[cfg(feature = "syntax-highlighting")]
        {
            self.get_language_for_extension(_extension).is_some()
        }
        #[cfg(not(feature = "syntax-highlighting"))]
        {
            false
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Get file extension from a file path
pub fn get_file_extension(file_path: &str) -> Option<String> {
    std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("file.rs"), Some("rs".to_string()));
        assert_eq!(get_file_extension("file.py"), Some("py".to_string()));
        assert_eq!(get_file_extension("file.js"), Some("js".to_string()));
        assert_eq!(get_file_extension("file"), None);
        assert_eq!(get_file_extension("file."), None);
    }

    #[test]
    fn test_syntax_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        // Test that highlighter can be created (actual language support depends on feature)
        let _ = highlighter.has_syntax_for_extension("rs");
    }

    #[test]
    fn test_highlight_line() {
        let highlighter = SyntaxHighlighter::new();
        let spans = highlighter.highlight_line("fn main() {}", "rs");
        // Should return at least one span (even if plain text)
        assert!(!spans.is_empty());
    }
}
