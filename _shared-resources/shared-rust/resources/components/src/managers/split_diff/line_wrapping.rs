// Line wrapping module
// Contains logic for wrapping lines to fit available width with word-level highlighting

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use super::diff_algorithms::{compute_word_diff_source, compute_word_diff_dest};
use crate::utilities::SyntaxHighlighter;

/// Create wrapped lines with word-level highlighting
pub fn create_wrapped_lines(
    idx: usize,
    line: &str,
    other_line: Option<&str>,
    base_style: Style,
    highlight_style: Style,
    is_first: bool,
    is_dest: bool,
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
) -> Vec<Line<'static>> {
    let line_num = idx + 1;
    let mut wrapped_lines = Vec::new();
    
    // Compute word-level diff if other line exists
    let word_diffs = if let Some(other) = other_line {
        if is_dest {
            compute_word_diff_dest(line, other)
        } else {
            compute_word_diff_source(line, other)
        }
    } else {
        vec![(line.to_string(), true)]
    };
    
    // Build spans with appropriate styling
    let mut spans: Vec<Span> = Vec::new();
    for (word, is_changed) in word_diffs {
        let style = if is_changed { highlight_style } else { base_style };
        spans.push(Span::styled(word, style));
    }
    
    // Wrap the spans to fit available width
    let mut current_line_spans: Vec<Span> = Vec::new();
    let mut current_width = 0;
    let mut is_first_wrap = true;
    
    // Helper to create gutter string
    let create_gutter = |is_first_wrap_val: bool| -> String {
        if is_first && is_first_wrap_val {
            format!("{:width$} ", line_num, width = max_line_digits)
        } else {
            " ".repeat(gutter_width)
        }
    };
    
    // Helper to finish current line (inline to avoid closure lifetime issues)
    macro_rules! finish_line {
        () => {{
            let gutter = create_gutter(is_first_wrap);
            let line_width = current_line_spans.iter()
                .map(|s| s.content.chars().count())
                .sum::<usize>();
            let remaining_width = text_width.saturating_sub(line_width);
            let padding = " ".repeat(remaining_width);
            
            let mut line_spans = vec![
                Span::styled(gutter, Style::default().fg(Color::Rgb(100,107,121))),
            ];
            line_spans.extend(current_line_spans.drain(..));
            if remaining_width > 0 {
                let padding_style = if other_line.is_none() { highlight_style } else { base_style };
                line_spans.push(Span::styled(padding, padding_style));
            }
            
            wrapped_lines.push(Line::from(line_spans));
            current_width = 0;
            is_first_wrap = false;
        }};
    }
    
    for span in spans {
        let span_text = span.content.clone();
        let span_width = span_text.chars().count();
        let span_style = span.style;
        
        // Handle case where a single span is longer than text_width
        if span_width > text_width {
            let words: Vec<String> = span_text.split_inclusive(char::is_whitespace)
                .map(|s| s.to_string())
                .collect();
            
            for word in words {
                let word_width = word.chars().count();
                
                if !current_line_spans.is_empty() && current_width + word_width > text_width {
                    finish_line!();
                }
                
                if word_width > text_width {
                    let mut remaining_word = word.to_string();
                    while !remaining_word.is_empty() {
                        if !current_line_spans.is_empty() && current_width >= text_width {
                            finish_line!();
                        }
                        
                        let chars: Vec<char> = remaining_word.chars().collect();
                        let chunk_size = text_width.min(chars.len());
                        let chunk_text: String = chars[..chunk_size].iter().collect();
                        remaining_word = chars[chunk_size..].iter().collect();
                        
                        let chunk_span = Span::styled(chunk_text, span_style);
                        current_line_spans.push(chunk_span);
                        current_width += chunk_size;
                        
                        if !remaining_word.is_empty() {
                            finish_line!();
                        }
                    }
                } else {
                    let word_span = Span::styled(word.clone(), span_style);
                    current_line_spans.push(word_span);
                    current_width += word_width;
                    
                    if current_width >= text_width {
                        finish_line!();
                    }
                }
            }
        } else {
            // Normal case: span fits or would fit with current line
            if (!current_line_spans.is_empty() && current_width + span_width > text_width) ||
               (current_width >= text_width && !current_line_spans.is_empty()) {
                finish_line!();
            }
            
            current_line_spans.push(span);
            current_width += span_width;
            
            if current_width >= text_width && !current_line_spans.is_empty() {
                finish_line!();
            }
        }
    }
    
    // Add final line
    if !current_line_spans.is_empty() {
        let gutter = create_gutter(is_first_wrap);
        let line_width = current_line_spans.iter()
            .map(|s| s.content.chars().count())
            .sum::<usize>();
        let remaining_width = text_width.saturating_sub(line_width);
        let padding = " ".repeat(remaining_width);
        
        let mut line_spans = vec![
            Span::styled(gutter, Style::default().fg(Color::Rgb(100,107,121))),
        ];
        line_spans.extend(current_line_spans.drain(..));
        if remaining_width > 0 {
            let padding_style = if other_line.is_none() { highlight_style } else { base_style };
            line_spans.push(Span::styled(padding, padding_style));
        }
        
        wrapped_lines.push(Line::from(line_spans));
    }
    
    wrapped_lines
}

/// Merge two styles, prioritizing diff highlighting background over syntax foreground
fn merge_styles(syntax_style: Style, diff_style: Style) -> Style {
    // Diff highlighting takes precedence for background (changed lines)
    // Syntax highlighting provides foreground color
    let mut merged = diff_style;
    
    // If diff style doesn't have a foreground, use syntax foreground
    if merged.fg.is_none() {
        if let Some(fg_color) = syntax_style.fg {
            merged = merged.fg(fg_color);
        }
    }
    
    merged
}

/// Create wrapped lines with syntax highlighting and word-level diff highlighting
/// Syntax highlighting is applied first, then diff highlighting is overlaid
/// This version merges syntax foreground colors with diff background colors
pub fn create_wrapped_lines_with_syntax(
    idx: usize,
    line: &str,
    other_line: Option<&str>,
    base_style: Style,
    highlight_style: Style,
    is_first: bool,
    is_dest: bool,
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
    syntax_highlighter: Option<&SyntaxHighlighter>,
    file_extension: Option<&str>,
) -> Vec<Line<'static>> {
    // If no syntax highlighting, fall back to regular function
    if syntax_highlighter.is_none() || file_extension.is_none() {
        return create_wrapped_lines(
            idx, line, other_line, base_style, highlight_style,
            is_first, is_dest, text_width, gutter_width, max_line_digits,
        );
    }
    
    let line_num = idx + 1;
    let mut wrapped_lines = Vec::new();
    
    // Get syntax-highlighted spans for the entire line
    let syntax_spans = syntax_highlighter.unwrap().highlight_line(line, file_extension.unwrap());
    
    // Compute word-level diff if other line exists
    let word_diffs = if let Some(other) = other_line {
        if is_dest {
            compute_word_diff_dest(line, other)
        } else {
            compute_word_diff_source(line, other)
        }
    } else {
        vec![(line.to_string(), true)]
    };
    
    // Build a character-by-character map of syntax styles
    // This allows us to efficiently look up syntax style for any character position
    let mut syntax_style_map: Vec<(usize, Style)> = Vec::new();
    let mut char_pos = 0;
    for syntax_span in &syntax_spans {
        let span_len = syntax_span.content.chars().count();
        for _ in 0..span_len {
            syntax_style_map.push((char_pos, syntax_span.style));
            char_pos += 1;
        }
    }
    
    // Build merged spans by processing word diffs and applying syntax colors
    let mut spans: Vec<Span> = Vec::new();
    let mut line_char_pos = 0;
    
    for (word, is_changed) in word_diffs {
        let word_chars: Vec<char> = word.chars().collect();
        let word_start = line_char_pos;
        let word_end = line_char_pos + word_chars.len();
        
        // Determine diff style for this word
        let diff_style = if is_changed { highlight_style } else { base_style };
        
        // Process word character by character to merge with syntax highlighting
        let mut current_chunk = String::new();
        let mut current_syntax_style: Option<Style> = None;
        
        for (i, ch) in word_chars.iter().enumerate() {
            let char_pos = word_start + i;
            
            // Get syntax style for this character position
            let syntax_style = syntax_style_map.get(char_pos)
                .map(|(_, style)| *style)
                .unwrap_or(Style::default());
            
            // If syntax style changed, finish current chunk and start new one
            if current_syntax_style != Some(syntax_style) {
                if !current_chunk.is_empty() {
                    let merged_style = merge_styles(
                        current_syntax_style.unwrap_or(Style::default()),
                        diff_style
                    );
                    spans.push(Span::styled(current_chunk.clone(), merged_style));
                    current_chunk.clear();
                }
                current_syntax_style = Some(syntax_style);
            }
            
            current_chunk.push(*ch);
        }
        
        // Add final chunk for this word
        if !current_chunk.is_empty() {
            let merged_style = merge_styles(
                current_syntax_style.unwrap_or(Style::default()),
                diff_style
            );
            spans.push(Span::styled(current_chunk, merged_style));
        }
        
        line_char_pos = word_end;
    }
    
    // Use the same wrapping logic as the original function
    let mut current_line_spans: Vec<Span> = Vec::new();
    let mut current_width = 0;
    let mut is_first_wrap = true;
    
    // Helper to create gutter string
    let create_gutter = |is_first_wrap_val: bool| -> String {
        if is_first && is_first_wrap_val {
            format!("{:width$} ", line_num, width = max_line_digits)
        } else {
            " ".repeat(gutter_width)
        }
    };
    
    // Helper to finish current line
    macro_rules! finish_line {
        () => {{
            let gutter = create_gutter(is_first_wrap);
            let line_width = current_line_spans.iter()
                .map(|s| s.content.chars().count())
                .sum::<usize>();
            let remaining_width = text_width.saturating_sub(line_width);
            let padding = " ".repeat(remaining_width);
            
            let mut line_spans = vec![
                Span::styled(gutter, Style::default().fg(Color::Rgb(100,107,121))),
            ];
            line_spans.extend(current_line_spans.drain(..));
            if remaining_width > 0 {
                let padding_style = if other_line.is_none() { highlight_style } else { base_style };
                line_spans.push(Span::styled(padding, padding_style));
            }
            
            wrapped_lines.push(Line::from(line_spans));
            current_width = 0;
            is_first_wrap = false;
        }};
    }
    
    for span in spans {
        let span_text = span.content.clone();
        let span_width = span_text.chars().count();
        let span_style = span.style;
        
        // Handle case where a single span is longer than text_width
        if span_width > text_width {
            let words: Vec<String> = span_text.split_inclusive(char::is_whitespace)
                .map(|s| s.to_string())
                .collect();
            
            for word in words {
                let word_width = word.chars().count();
                
                if !current_line_spans.is_empty() && current_width + word_width > text_width {
                    finish_line!();
                }
                
                if word_width > text_width {
                    let mut remaining_word = word.to_string();
                    while !remaining_word.is_empty() {
                        if !current_line_spans.is_empty() && current_width >= text_width {
                            finish_line!();
                        }
                        
                        let chars: Vec<char> = remaining_word.chars().collect();
                        let chunk_size = text_width.min(chars.len());
                        let chunk_text: String = chars[..chunk_size].iter().collect();
                        remaining_word = chars[chunk_size..].iter().collect();
                        
                        let chunk_span = Span::styled(chunk_text, span_style);
                        current_line_spans.push(chunk_span);
                        current_width += chunk_size;
                        
                        if !remaining_word.is_empty() {
                            finish_line!();
                        }
                    }
                } else {
                    let word_span = Span::styled(word.clone(), span_style);
                    current_line_spans.push(word_span);
                    current_width += word_width;
                    
                    if current_width >= text_width {
                        finish_line!();
                    }
                }
            }
        } else {
            // Normal case: span fits or would fit with current line
            if (!current_line_spans.is_empty() && current_width + span_width > text_width) ||
               (current_width >= text_width && !current_line_spans.is_empty()) {
                finish_line!();
            }
            
            current_line_spans.push(span);
            current_width += span_width;
            
            if current_width >= text_width && !current_line_spans.is_empty() {
                finish_line!();
            }
        }
    }
    
    // Add final line
    if !current_line_spans.is_empty() {
        let gutter = create_gutter(is_first_wrap);
        let line_width = current_line_spans.iter()
            .map(|s| s.content.chars().count())
            .sum::<usize>();
        let remaining_width = text_width.saturating_sub(line_width);
        let padding = " ".repeat(remaining_width);
        
        let mut line_spans = vec![
            Span::styled(gutter, Style::default().fg(Color::Rgb(100,107,121))),
        ];
        line_spans.extend(current_line_spans.drain(..));
        if remaining_width > 0 {
            let padding_style = if other_line.is_none() { highlight_style } else { base_style };
            line_spans.push(Span::styled(padding, padding_style));
        }
        
        wrapped_lines.push(Line::from(line_spans));
    }
    
    wrapped_lines
}
