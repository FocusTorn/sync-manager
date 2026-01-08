// Rendering module
// Contains logic for computing render-ready data from aligned lines

use crate::elements::{SplitDiffViewConfig, SplitDiffViewState};
use crate::utilities::SyntaxHighlighter;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use super::diff_algorithms::{LineAlignment, align_lines};
use super::line_wrapping::{create_wrapped_lines, create_wrapped_lines_with_syntax};

/// Render-ready data for split diff view
pub struct SplitDiffRenderData {
    pub source_lines: Vec<Line<'static>>,
    pub dest_lines: Vec<Line<'static>>,
}

/// Parameters for rendering split diff view
/// Groups all rendering parameters into a single struct for better API design
/// Note: Does not implement Clone because it contains mutable references
#[derive(Debug)]
pub struct RenderParams<'a> {
    /// Configuration for the split diff view
    pub config: &'a SplitDiffViewConfig,
    /// State for the split diff view (mutable for cache updates)
    pub state: &'a mut SplitDiffViewState,
    /// Source lines to render
    pub source_lines: &'a [String],
    /// Destination lines to render
    pub dest_lines: &'a [String],
    /// Width available for text (excluding gutter and borders)
    pub text_width: usize,
    /// Width of the gutter (line numbers + spacing)
    pub gutter_width: usize,
    /// Maximum number of digits in line numbers
    pub max_line_digits: usize,
    /// Available height for rendering
    pub available_height: usize,
    /// Optional syntax highlighter for syntax highlighting
    pub syntax_highlighter: Option<&'a SyntaxHighlighter>,
}

impl<'a> RenderParams<'a> {
    /// Create a new RenderParams struct
    pub fn new(
        config: &'a SplitDiffViewConfig,
        state: &'a mut SplitDiffViewState,
        source_lines: &'a [String],
        dest_lines: &'a [String],
        text_width: usize,
        gutter_width: usize,
        max_line_digits: usize,
        available_height: usize,
    ) -> Self {
        Self {
            config,
            state,
            source_lines,
            dest_lines,
            text_width,
            gutter_width,
            max_line_digits,
            available_height,
            syntax_highlighter: None,
        }
    }

    /// Create a new RenderParams struct with syntax highlighter
    pub fn with_syntax_highlighter(
        config: &'a SplitDiffViewConfig,
        state: &'a mut SplitDiffViewState,
        source_lines: &'a [String],
        dest_lines: &'a [String],
        text_width: usize,
        gutter_width: usize,
        max_line_digits: usize,
        available_height: usize,
        syntax_highlighter: Option<&'a SyntaxHighlighter>,
    ) -> Self {
        Self {
            config,
            state,
            source_lines,
            dest_lines,
            text_width,
            gutter_width,
            max_line_digits,
            available_height,
            syntax_highlighter,
        }
    }
}

/// Compute render-ready data for the split diff view (static method)
/// This is the main business logic method that performs all diff computation
/// Can be called without a manager instance
/// 
/// Uses RenderParams struct to reduce parameter count and improve maintainability
pub fn compute_render_data_static(params: RenderParams<'_>) -> SplitDiffRenderData {
    let RenderParams {
        config,
        state,
        source_lines,
        dest_lines,
        text_width,
        gutter_width,
        max_line_digits,
        available_height,
        syntax_highlighter,
    } = params;
    
    // Helper function to create wrapped lines with optional syntax highlighting
    let create_wrapped = |idx: usize, line: &str, other_line: Option<&str>,
                          base_style: Style, highlight_style: Style,
                          is_first: bool, is_dest: bool| -> Vec<Line<'static>> {
        if let (Some(highlighter), Some(ext)) = (syntax_highlighter, config.file_extension.as_deref()) {
            create_wrapped_lines_with_syntax(
                idx, line, other_line, base_style, highlight_style,
                is_first, is_dest, text_width, gutter_width, max_line_digits,
                Some(highlighter), Some(ext),
            )
        } else {
            create_wrapped_lines(
                idx, line, other_line, base_style, highlight_style,
                is_first, is_dest, text_width, gutter_width, max_line_digits,
            )
        }
    };
    // Align lines using LCS-like algorithm
    let aligned_lines = align_lines(source_lines, dest_lines);
    
    // Process aligned lines and create render-ready data
    let mut source_visible: Vec<Line<'static>> = Vec::new();
    let mut dest_visible: Vec<Line<'static>> = Vec::new();
    
    const CONTEXT_LINES: usize = 3;
    
    // Helper function to create blank padding lines (no line numbers)
    let create_blank_padding = |count: usize| -> Vec<Line<'static>> {
        (0..count).map(|_| {
            Line::from(vec![
                Span::styled("     ", Style::default().fg(Color::Rgb(100,107,121))),
                Span::styled(" ".repeat(text_width), Style::default()),
            ])
        }).collect()
    };
    
    // Helper function to pad shorter side to match longer side's height
    let pad_to_match = |source_lines: &mut Vec<Line>, dest_lines: &mut Vec<Line>| {
        let src_height = source_lines.len();
        let dest_height = dest_lines.len();
        if src_height < dest_height {
            let padding = create_blank_padding(dest_height - src_height);
            source_lines.extend(padding);
        } else if dest_height < src_height {
            let padding = create_blank_padding(src_height - dest_height);
            dest_lines.extend(padding);
        }
    };
    
    // Helper function to check if a line has changes
    let has_changes = |line_type: &LineAlignment| -> bool {
        match line_type {
            LineAlignment::Both(src_idx, dest_idx) => {
                source_lines[*src_idx] != dest_lines[*dest_idx]
            }
            LineAlignment::SourceOnly(_) | LineAlignment::DestOnly(_) => true,
        }
    };
    
    let mut i = 0;
    while i < aligned_lines.len() {
        // Check if we should fold unchanged regions
        if state.fold_unchanged {
            // Count consecutive unchanged lines
            let mut unchanged_count = 0;
            let mut j = i;
            while j < aligned_lines.len() {
                match &aligned_lines[j] {
                    LineAlignment::Both(src_idx, dest_idx) => {
                        let src_line = &source_lines[*src_idx];
                        let dest_line = &dest_lines[*dest_idx];
                        if src_line == dest_line {
                            unchanged_count += 1;
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            
            // Check if there are changes before and after this unchanged region
            let has_change_before = i > 0 && has_changes(&aligned_lines[i - 1]);
            let has_change_after = (i + unchanged_count) < aligned_lines.len() &&
                                   has_changes(&aligned_lines[i + unchanged_count]);
            
            // Only fold if we have enough unchanged lines and there are changes nearby
            let min_lines_for_fold = if has_change_before && has_change_after {
                CONTEXT_LINES * 2 + 1
            } else if has_change_before || has_change_after {
                CONTEXT_LINES + 1
            } else {
                usize::MAX
            };
            
            if unchanged_count > min_lines_for_fold {
                let mut context_before = 0;
                let mut context_after = 0;
                let mut hidden_count = unchanged_count;
                
                if has_change_before {
                    context_before = CONTEXT_LINES.min(unchanged_count);
                    hidden_count -= context_before;
                }
                
                if has_change_after {
                    context_after = CONTEXT_LINES.min(hidden_count);
                    hidden_count -= context_after;
                }
                
                // Show context lines before the fold
                if context_before > 0 {
                    let context_start = i;
                    let context_end = i + context_before;
                    for k in context_start..context_end {
                        if let LineAlignment::Both(src_idx, dest_idx) = &aligned_lines[k] {
                            let src_line = &source_lines[*src_idx];
                            let dest_line = &dest_lines[*dest_idx];
                            
                            let mut src_wrapped = create_wrapped(
                                *src_idx, src_line, Some(dest_line),
                                Style::default(), Style::default(),
                                true, false,
                            );
                            let mut dest_wrapped = create_wrapped(
                                *dest_idx, dest_line, Some(src_line),
                                Style::default(), Style::default(),
                                true, true,
                            );
                            
                            // Pad to match heights
                            pad_to_match(&mut src_wrapped, &mut dest_wrapped);
                            
                            source_visible.extend(src_wrapped);
                            dest_visible.extend(dest_wrapped);
                        }
                    }
                }
                
                // Add "🡙 X lines hidden" indicator
                if hidden_count > 0 {
                    let hidden_text = format!("{} lines hidden", hidden_count);
                    let gutter_spaces = if max_line_digits > 1 {
                        " ".repeat(max_line_digits - 1)
                    } else {
                        String::new()
                    };
                    let indicator_line = Line::from(vec![
                        Span::styled(gutter_spaces, Style::default().fg(Color::Rgb(100,107,121))),
                        Span::styled("🡙 ", Style::default().fg(Color::Rgb(160, 160, 160))),
                        Span::styled(
                            hidden_text.clone(),
                            Style::default().fg(Color::Rgb(160, 160, 160))
                        ),
                        Span::styled(" ".repeat(text_width.saturating_sub(hidden_text.len())), Style::default()),
                    ]);
                    source_visible.push(indicator_line.clone());
                    dest_visible.push(indicator_line);
                }
                
                // Show context lines after the fold
                if context_after > 0 {
                    let after_context_start = i + unchanged_count.saturating_sub(context_after);
                    let after_context_end = i + unchanged_count;
                    
                    for k in after_context_start..after_context_end {
                        if let LineAlignment::Both(src_idx, dest_idx) = &aligned_lines[k] {
                            let src_line = &source_lines[*src_idx];
                            let dest_line = &dest_lines[*dest_idx];
                            
                            let mut src_wrapped = create_wrapped(
                                *src_idx, src_line, Some(dest_line),
                                Style::default(), Style::default(),
                                true, false,
                            );
                            let mut dest_wrapped = create_wrapped(
                                *dest_idx, dest_line, Some(src_line),
                                Style::default(), Style::default(),
                                true, true,
                            );
                            
                            // Pad to match heights
                            pad_to_match(&mut src_wrapped, &mut dest_wrapped);
                            
                            source_visible.extend(src_wrapped);
                            dest_visible.extend(dest_wrapped);
                        }
                    }
                }
                
                // Skip all the unchanged lines
                i += unchanged_count;
                continue;
            }
        }
        
        // Process the current line normally
        let aligned_line = &aligned_lines[i];
        match aligned_line {
            LineAlignment::Both(src_idx, dest_idx) => {
                let src_line = &source_lines[*src_idx];
                let dest_line = &dest_lines[*dest_idx];
                let is_same = src_line == dest_line;
                
                let base_style = if !is_same {
                    Style::default().bg(Color::Rgb(55, 4, 4))
                } else {
                    Style::default()
                };
                
                let highlight_style = if !is_same {
                    Style::default().bg(Color::Rgb(95, 3, 3))
                } else {
                    Style::default()
                };
                
                let mut src_wrapped = create_wrapped(
                    *src_idx, src_line, Some(dest_line),
                    base_style, highlight_style,
                    true, false,
                );
                
                let base_style_dest = if !is_same {
                    Style::default().bg(Color::Rgb(35, 41, 21))
                } else {
                    Style::default()
                };
                
                let highlight_style_dest = if !is_same {
                    Style::default().bg(Color::Rgb(80, 102, 31))
                } else {
                    Style::default()
                };
                
                let mut dest_wrapped = create_wrapped(
                    *dest_idx, dest_line, Some(src_line),
                    base_style_dest, highlight_style_dest,
                    true, true,
                );
                
                // Pad to match heights so lines stay vertically aligned
                pad_to_match(&mut src_wrapped, &mut dest_wrapped);
                
                source_visible.extend(src_wrapped);
                dest_visible.extend(dest_wrapped);
                
                i += 1;
            }
            LineAlignment::SourceOnly(_s_idx) => {
                // Process all consecutive SourceOnly lines together
                let mut total_height = 0;
                let mut k = i;
                while k < aligned_lines.len() {
                    if let LineAlignment::SourceOnly(s_idx) = &aligned_lines[k] {
                        let src_line = &source_lines[*s_idx];
                        let highlight_style = Style::default().bg(Color::Rgb(95, 3, 3));
                        
                        let src_wrapped = create_wrapped(
                            *s_idx, src_line, None,
                            highlight_style, highlight_style,
                            true, false,
                        );
                        total_height += src_wrapped.len();
                        source_visible.extend(src_wrapped);
                        k += 1;
                    } else {
                        break;
                    }
                }
                
                // Add padding to destination side for all removed lines at once
                let padding = create_blank_padding(total_height);
                dest_visible.extend(padding);
                
                i = k;
            }
            LineAlignment::DestOnly(_d_idx) => {
                // Process all consecutive DestOnly lines together
                let mut total_height = 0;
                let mut k = i;
                while k < aligned_lines.len() {
                    if let LineAlignment::DestOnly(d_idx) = &aligned_lines[k] {
                        let dest_line = &dest_lines[*d_idx];
                        let base_style = Style::default().bg(Color::Rgb(35, 41, 21));
                        let highlight_style = Style::default().bg(Color::Rgb(80, 102, 31));
                        
                        let dest_wrapped = create_wrapped(
                            *d_idx, dest_line, None,
                            base_style, highlight_style,
                            true, true,
                        );
                        total_height += dest_wrapped.len();
                        dest_visible.extend(dest_wrapped);
                        k += 1;
                    } else {
                        break;
                    }
                }
                
                // Add padding to source side for all added lines at once
                let padding = create_blank_padding(total_height);
                source_visible.extend(padding);
                
                i = k;
            }
        }
    }
    
    // Apply scroll offset to the rendered lines
    let scroll_offset = state.scroll_offset.min(source_visible.len().saturating_sub(1));
    if scroll_offset > 0 {
        source_visible.drain(..scroll_offset);
        dest_visible.drain(..scroll_offset);
    }
    
    // Truncate to available height
    if source_visible.len() > available_height {
        source_visible.truncate(available_height);
    }
    if dest_visible.len() > available_height {
        dest_visible.truncate(available_height);
    }
    
    SplitDiffRenderData {
        source_lines: source_visible,
        dest_lines: dest_visible,
    }
}
