// Side-by-Side Diff View
// Renders source and destination files in parallel columns

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::core::{App, ViewMode};
use crate::operations::diff::{align_lines, compute_word_diff_dest, compute_word_diff_source, LineAlignment};
use super::Styles;

/// Render side-by-side diff view
pub fn render_side_by_side(f: &mut Frame, app: &App, area: Rect) {
    if let (Some(source_lines), Some(dest_lines)) =
        (&app.side_by_side_source, &app.side_by_side_dest)
    {
        // Split area into two columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let available_height = columns[0].height.saturating_sub(2) as usize;
        
        // Calculate maximum line number to determine gutter width
        let max_line_num = source_lines.len().max(dest_lines.len());
        let max_line_digits = if max_line_num == 0 {
            1
        } else {
            (max_line_num as f64).log10().floor() as usize + 1
        };
        let gutter_width = max_line_digits + 1; // +1 for the space after the number
        let right_margin = 1; // Single column gap on the right
        // Content area is inside borders: columns[0].width - 2
        // Text should wrap 1 column before right border, so available width is: columns[0].width - 2 - 1
        // This space is divided into: gutter_width + text_width + right_margin
        // So: text_width = (columns[0].width - 2 - 1) - gutter_width - right_margin
        let content_area_width = columns[0].width.saturating_sub(2) as usize; // Inside borders
        let wrap_at = content_area_width.saturating_sub(1); // 1 column before right border
        let text_width = wrap_at.saturating_sub(gutter_width + right_margin);

        // Align lines
        let aligned_lines = align_lines(source_lines, dest_lines);

        // Build visible lines for both panels
        let (mut source_visible, mut dest_visible) =
            build_aligned_lines(&aligned_lines, source_lines, dest_lines, text_width, gutter_width, max_line_digits, app);

        // Apply scroll offset
        let scroll_offset = app
            .diff_scroll_offset
            .min(source_visible.len().saturating_sub(1));
        if scroll_offset > 0 {
            source_visible.drain(..scroll_offset);
            dest_visible.drain(..scroll_offset);
        }

        // Truncate to available height
        source_visible.truncate(available_height);
        dest_visible.truncate(available_height);

        // Panel titles
        let (left_label, right_label) = match app.view_mode {
            ViewMode::SharedToProject => ("Shared", "Project"),
            ViewMode::ProjectToShared => ("Project", "Shared"),
        };

        let source_title = app
            .selected_diff()
            .map(|d| format!("{}: {}", left_label, short_path(&d.source_path)))
            .unwrap_or_else(|| left_label.to_string());

        let dest_title = app
            .selected_diff()
            .map(|d| format!("{}: {}", right_label, short_path(&d.destination_path)))
            .unwrap_or_else(|| right_label.to_string());

        let source_widget = Paragraph::new(source_visible)
            .block(Block::default().borders(Borders::ALL).title(source_title));
        f.render_widget(source_widget, columns[0]);

        let dest_widget = Paragraph::new(dest_visible)
            .block(Block::default().borders(Borders::ALL).title(dest_title));
        f.render_widget(dest_widget, columns[1]);
    } else {
        let loading = Paragraph::new("Loading files...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Side-by-Side Diff"),
            );
        f.render_widget(loading, area);
    }
}

/// Get shortened path for display
fn short_path(path: &std::path::Path) -> String {
    let components: Vec<_> = path.components().rev().take(3).collect();
    let short: std::path::PathBuf = components.into_iter().rev().collect();
    short.display().to_string()
}

/// Build aligned lines for source and destination
fn build_aligned_lines(
    aligned: &[LineAlignment],
    source_lines: &[String],
    dest_lines: &[String],
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
    app: &App,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let mut source_visible: Vec<Line<'static>> = Vec::new();
    let mut dest_visible: Vec<Line<'static>> = Vec::new();

    const CONTEXT_LINES: usize = 3;

    // Helper to normalize empty/whitespace-only lines for comparison
    fn normalize_for_comparison(line: &str) -> &str {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            ""
        } else {
            line
        }
    }
    
    let has_changes = |line_type: &LineAlignment| -> bool {
        match line_type {
            LineAlignment::Both(src_idx, dest_idx) => {
                let src_norm = normalize_for_comparison(&source_lines[*src_idx]);
                let dest_norm = normalize_for_comparison(&dest_lines[*dest_idx]);
                src_norm != dest_norm
            }
            LineAlignment::SourceOnly(_) | LineAlignment::DestOnly(_) => true,
        }
    };

    let mut i = 0;
    while i < aligned.len() {
        // Check for foldable unchanged regions
        if app.fold_unchanged {
            let mut unchanged_count = 0;
            let mut j = i;
            while j < aligned.len() {
                match &aligned[j] {
                    LineAlignment::Both(src_idx, dest_idx) => {
                        let src_norm = normalize_for_comparison(&source_lines[*src_idx]);
                        let dest_norm = normalize_for_comparison(&dest_lines[*dest_idx]);
                        if src_norm == dest_norm {
                            unchanged_count += 1;
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            let has_change_before = i > 0 && has_changes(&aligned[i - 1]);
            let has_change_after =
                (i + unchanged_count) < aligned.len() && has_changes(&aligned[i + unchanged_count]);

            let min_lines_for_fold = if has_change_before && has_change_after {
                CONTEXT_LINES * 2 + 1
            } else if has_change_before || has_change_after {
                CONTEXT_LINES + 1
            } else {
                usize::MAX
            };

            if unchanged_count > min_lines_for_fold {
                let context_before = if has_change_before {
                    CONTEXT_LINES.min(unchanged_count)
                } else {
                    0
                };
                let context_after = if has_change_after {
                    CONTEXT_LINES.min(unchanged_count - context_before)
                } else {
                    0
                };
                let hidden_count = unchanged_count - context_before - context_after;

                // Show context before
                for k in i..(i + context_before) {
                    if let LineAlignment::Both(src_idx, dest_idx) = &aligned[k] {
                        add_unchanged_line(
                            &mut source_visible,
                            &mut dest_visible,
                            *src_idx,
                            *dest_idx,
                            source_lines,
                            dest_lines,
                            text_width,
                            gutter_width,
                            max_line_digits,
                        );
                    }
                }

                // Show fold indicator
                if hidden_count > 0 {
                    let indicator = create_fold_indicator(hidden_count, text_width, gutter_width);
                    source_visible.push(indicator.clone());
                    dest_visible.push(indicator);
                }

                // Show context after
                let after_start = i + unchanged_count - context_after;
                for k in after_start..(i + unchanged_count) {
                    if let LineAlignment::Both(src_idx, dest_idx) = &aligned[k] {
                        add_unchanged_line(
                            &mut source_visible,
                            &mut dest_visible,
                            *src_idx,
                            *dest_idx,
                            source_lines,
                            dest_lines,
                            text_width,
                            gutter_width,
                            max_line_digits,
                        );
                    }
                }

                i += unchanged_count;
                continue;
            }
        }

        // Process the current line normally
        match &aligned[i] {
            LineAlignment::Both(src_idx, dest_idx) => {
                let src_line = &source_lines[*src_idx];
                let dest_line = &dest_lines[*dest_idx];
                // Normalize empty/whitespace-only lines for comparison
                let src_normalized = src_line.trim();
                let dest_normalized = dest_line.trim();
                // If both are empty after normalization, treat as same
                // Otherwise, compare the original lines
                let is_same = if src_normalized.is_empty() && dest_normalized.is_empty() {
                    true
                } else {
                    src_line == dest_line
                };

                if is_same {
                    add_unchanged_line(
                        &mut source_visible,
                        &mut dest_visible,
                        *src_idx,
                        *dest_idx,
                        source_lines,
                        dest_lines,
                        text_width,
                        gutter_width,
                        max_line_digits,
                    );
                } else {
                    add_modified_line(
                        &mut source_visible,
                        &mut dest_visible,
                        *src_idx,
                        *dest_idx,
                        source_lines,
                        dest_lines,
                        text_width,
                        gutter_width,
                        max_line_digits,
                    );
                }
            }
            LineAlignment::SourceOnly(src_idx) => {
                // Check if next line is a blank line that should be shown as changed
                let (show_next_blank_as_changed, next_src_idx, next_dest_idx) = if i + 1 < aligned.len() {
                    if let LineAlignment::Both(ns_idx, nd_idx) = &aligned[i + 1] {
                        let next_src_norm = normalize_for_comparison(&source_lines[*ns_idx]);
                        let next_dest_norm = normalize_for_comparison(&dest_lines[*nd_idx]);
                        // If next line is blank in both, show it as changed below the added line
                        if next_src_norm.is_empty() && next_dest_norm.is_empty() {
                            (true, *ns_idx, *nd_idx)
                        } else {
                            (false, 0, 0)
                        }
                    } else {
                        (false, 0, 0)
                    }
                } else {
                    (false, 0, 0)
                };
                
                add_source_only_line(
                    &mut source_visible,
                    &mut dest_visible,
                    *src_idx,
                    source_lines,
                    text_width,
                    gutter_width,
                    max_line_digits,
                    if show_next_blank_as_changed { Some((next_src_idx, next_dest_idx)) } else { None },
                );
                
                // If we're showing the next blank as changed, skip it in the main loop
                if show_next_blank_as_changed {
                    i += 1; // Skip the next Both alignment since we handled it
                }
            }
            LineAlignment::DestOnly(dest_idx) => {
                // Check if next line is a blank line that should be shown as changed
                let (show_next_blank_as_changed, next_src_idx, next_dest_idx) = if i + 1 < aligned.len() {
                    if let LineAlignment::Both(ns_idx, nd_idx) = &aligned[i + 1] {
                        let next_src_norm = normalize_for_comparison(&source_lines[*ns_idx]);
                        let next_dest_norm = normalize_for_comparison(&dest_lines[*nd_idx]);
                        // If next line is blank in both, show it as changed below the added line
                        if next_src_norm.is_empty() && next_dest_norm.is_empty() {
                            (true, *ns_idx, *nd_idx)
                        } else {
                            (false, 0, 0)
                        }
                    } else {
                        (false, 0, 0)
                    }
                } else {
                    (false, 0, 0)
                };
                
                add_dest_only_line(
                    &mut source_visible,
                    &mut dest_visible,
                    *dest_idx,
                    dest_lines,
                    text_width,
                    gutter_width,
                    max_line_digits,
                    if show_next_blank_as_changed { Some((next_src_idx, next_dest_idx)) } else { None },
                );
                
                // If we're showing the next blank as changed, skip it in the main loop
                if show_next_blank_as_changed {
                    i += 1; // Skip the next Both alignment since we handled it
                }
            }
        }

        i += 1;
    }

    (source_visible, dest_visible)
}

fn add_unchanged_line(
    source_visible: &mut Vec<Line<'static>>,
    dest_visible: &mut Vec<Line<'static>>,
    src_idx: usize,
    dest_idx: usize,
    source_lines: &[String],
    dest_lines: &[String],
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
) {
    let src_line = &source_lines[src_idx];
    let dest_line = &dest_lines[dest_idx];

    // Create source line (may wrap to multiple lines)
    let src_wrapped = create_highlighted_lines(
        src_idx + 1,
        &[(src_line.clone(), false)],
        text_width,
        gutter_width,
        max_line_digits,
        ratatui::style::Style::default(),
        ratatui::style::Style::default(),
    );
    
    // Create destination line (may wrap to multiple lines)
    let dest_wrapped = create_highlighted_lines(
        dest_idx + 1,
        &[(dest_line.clone(), false)],
        text_width,
        gutter_width,
        max_line_digits,
        ratatui::style::Style::default(),
        ratatui::style::Style::default(),
    );
    
    source_visible.extend(src_wrapped.clone());
    dest_visible.extend(dest_wrapped.clone());
    
    // Ensure both sides have the same number of lines by padding with blank lines
    let src_count = src_wrapped.len();
    let dest_count = dest_wrapped.len();
    
    if src_count > dest_count {
        // Source has more lines, pad destination
        for _ in dest_count..src_count {
            dest_visible.push(create_blank_line(text_width, gutter_width));
        }
    } else if dest_count > src_count {
        // Destination has more lines, pad source
        for _ in src_count..dest_count {
            source_visible.push(create_blank_line(text_width, gutter_width));
        }
    }
}

fn add_modified_line(
    source_visible: &mut Vec<Line<'static>>,
    dest_visible: &mut Vec<Line<'static>>,
    src_idx: usize,
    dest_idx: usize,
    source_lines: &[String],
    dest_lines: &[String],
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
) {
    let src_line = &source_lines[src_idx];
    let dest_line = &dest_lines[dest_idx];

    // Source line with word-level highlighting
    let src_diffs = compute_word_diff_source(src_line, dest_line);
    let src_wrapped = create_highlighted_lines(
        src_idx + 1,
        &src_diffs,
        text_width,
        gutter_width,
        max_line_digits,
        Styles::side_by_side_source_modified_bg(),
        Styles::side_by_side_source_highlight(),
    );
    source_visible.extend(src_wrapped.clone());

    // Destination line with word-level highlighting
    let dest_diffs = compute_word_diff_dest(dest_line, src_line);
    let dest_wrapped = create_highlighted_lines(
        dest_idx + 1,
        &dest_diffs,
        text_width,
        gutter_width,
        max_line_digits,
        Styles::side_by_side_dest_modified_bg(),
        Styles::side_by_side_dest_highlight(),
    );
    dest_visible.extend(dest_wrapped.clone());

    // Ensure both sides have the same number of lines by padding with blank lines
    let src_count = src_wrapped.len();
    let dest_count = dest_wrapped.len();
    
    if src_count > dest_count {
        // Source has more lines, pad destination
        for _ in dest_count..src_count {
            dest_visible.push(create_blank_line(text_width, gutter_width));
        }
    } else if dest_count > src_count {
        // Destination has more lines, pad source
        for _ in src_count..dest_count {
            source_visible.push(create_blank_line(text_width, gutter_width));
        }
    }
}

fn add_source_only_line(
    source_visible: &mut Vec<Line<'static>>,
    dest_visible: &mut Vec<Line<'static>>,
    src_idx: usize,
    source_lines: &[String],
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
    next_blank_indices: Option<(usize, usize)>,
) {
    let src_line = &source_lines[src_idx];
    
    // Create source line (may wrap to multiple lines)
    // Added lines should be fully bright (like VSCode)
    let src_wrapped = create_highlighted_lines(
        src_idx + 1,
        &[(src_line.clone(), true)],
        text_width,
        gutter_width,
        max_line_digits,
        Styles::side_by_side_source_highlight(), // Bright background for added lines
        Styles::side_by_side_source_highlight(), // Same for highlight
    );
    
    source_visible.extend(src_wrapped.clone());
    
    // Add matching number of blank lines to destination
    for _ in 0..src_wrapped.len() {
        dest_visible.push(create_blank_line(text_width, gutter_width));
    }
    
    // If next line is blank, show it as changed below the added line (bright, like VSCode)
    if let Some((next_src_idx, next_dest_idx)) = next_blank_indices {
        let next_src_line = &source_lines[next_src_idx];
        
        // Create blank line with bright highlighting (like VSCode shows blank lines below added lines)
        let blank_wrapped = create_highlighted_lines(
            next_src_idx + 1,
            &[(next_src_line.clone(), true)], // Bright highlighting for blank line below added line
            text_width,
            gutter_width,
            max_line_digits,
            Styles::side_by_side_source_highlight(), // Bright background (like VSCode)
            Styles::side_by_side_source_highlight(), // Same for highlight
        );
        
        source_visible.extend(blank_wrapped.clone());
        
        // Add matching blank lines to destination (unchanged, no background)
        for _ in 0..blank_wrapped.len() {
            dest_visible.push(create_blank_line(text_width, gutter_width));
        }
    }
}

fn add_dest_only_line(
    source_visible: &mut Vec<Line<'static>>,
    dest_visible: &mut Vec<Line<'static>>,
    dest_idx: usize,
    dest_lines: &[String],
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
    next_blank_indices: Option<(usize, usize)>,
) {
    let dest_line = &dest_lines[dest_idx];
    
    // Create destination line (may wrap to multiple lines)
    // Added lines should be fully bright (like VSCode)
    let dest_wrapped = create_highlighted_lines(
        dest_idx + 1,
        &[(dest_line.clone(), true)],
        text_width,
        gutter_width,
        max_line_digits,
        Styles::side_by_side_dest_highlight(), // Bright background for added lines
        Styles::side_by_side_dest_highlight(), // Same for highlight
    );
    
    dest_visible.extend(dest_wrapped.clone());
    
    // Add matching number of blank lines to source
    for _ in 0..dest_wrapped.len() {
        source_visible.push(create_blank_line(text_width, gutter_width));
    }
    
    // If next line is blank, show it as changed below the added line (bright, like VSCode)
    if let Some((next_src_idx, next_dest_idx)) = next_blank_indices {
        let next_dest_line = &dest_lines[next_dest_idx];
        
        // Create blank line with bright highlighting (like VSCode shows blank lines below added lines)
        let blank_wrapped = create_highlighted_lines(
            next_dest_idx + 1,
            &[(next_dest_line.clone(), true)], // Bright highlighting for blank line below added line
            text_width,
            gutter_width,
            max_line_digits,
            Styles::side_by_side_dest_highlight(), // Bright background (like VSCode)
            Styles::side_by_side_dest_highlight(), // Same for highlight
        );
        
        dest_visible.extend(blank_wrapped.clone());
        
        // Add matching blank lines to source (unchanged, no background)
        for _ in 0..blank_wrapped.len() {
            source_visible.push(create_blank_line(text_width, gutter_width));
        }
    }
}

/// Split text into "word+whitespace" units where whitespace is attached to the preceding word
fn split_into_word_units(text: &str) -> Vec<String> {
    let mut units = Vec::new();
    let mut current_unit = String::new();
    let mut in_word = false;
    
    for c in text.chars() {
        if c.is_whitespace() {
            if in_word {
                // Add whitespace to current word unit
                current_unit.push(c);
                units.push(current_unit.clone());
                current_unit.clear();
                in_word = false;
            } else {
                // Continue whitespace (shouldn't happen at start, but handle it)
                current_unit.push(c);
            }
        } else {
            if !in_word && !current_unit.is_empty() {
                // We had whitespace, start new word
                units.push(current_unit.clone());
                current_unit.clear();
            }
            current_unit.push(c);
            in_word = true;
        }
    }
    
    // Add final unit if any
    if !current_unit.is_empty() {
        units.push(current_unit);
    }
    
    units
}

fn create_highlighted_lines(
    line_num: usize,
    diffs: &[(String, bool)],
    text_width: usize,
    gutter_width: usize,
    max_line_digits: usize,
    base_style: ratatui::style::Style,
    highlight_style: ratatui::style::Style,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let gutter = format!("{:width$} ", line_num, width = max_line_digits);
    // Show line number on continuation lines too (for wrapped lines)
    let continuation_gutter = format!("{:width$} ", line_num, width = max_line_digits);
    
    let mut current_line_spans: Vec<Span> = Vec::new();
    let mut current_width = 0;
    let mut is_first_line = true;

    for (text, is_changed) in diffs {
        let style = if *is_changed { highlight_style } else { base_style };
        
        // Split text into "word+whitespace" units
        let word_units = split_into_word_units(text);
        
        for unit in word_units {
            let unit_width = unit.chars().count();
            let remaining_width = text_width.saturating_sub(current_width);
            
            // If this unit doesn't fit on current line, wrap to next line
            if unit_width > remaining_width && !current_line_spans.is_empty() {
                // Current line is full, wrap to next line
                let gutter_span = if is_first_line {
                    Span::styled(gutter.clone(), Styles::gutter())
                } else {
                    Span::styled(continuation_gutter.clone(), Styles::gutter())
                };
                
                // Calculate content width before moving
                let line_content_width: usize = current_line_spans.iter()
                    .map(|s| s.content.chars().count())
                    .sum();
                
                let mut line_spans = vec![gutter_span];
                line_spans.append(&mut current_line_spans);
                
                // Add padding to fill the line
                let padding_len = text_width.saturating_sub(line_content_width);
                if padding_len > 0 {
                    line_spans.push(Span::styled(" ".repeat(padding_len), base_style));
                }
                
                // Add right margin
                line_spans.push(Span::styled(" ", base_style));
                
                lines.push(Line::from(line_spans));
                current_line_spans = Vec::new();
                current_width = 0;
                is_first_line = false;
            }
            
            // If unit is too long for even an empty line, we need to break it (shouldn't happen often)
            let remaining_width = text_width.saturating_sub(current_width);
            if unit_width > remaining_width && current_line_spans.is_empty() {
                // Unit is longer than line width, break it character by character
                let unit_chars: Vec<char> = unit.chars().collect();
                let mut char_idx = 0;
                
                while char_idx < unit_chars.len() {
                    let remaining = text_width.saturating_sub(current_width);
                    if remaining == 0 {
                        // Wrap to next line
                        let gutter_span = if is_first_line {
                            Span::styled(gutter.clone(), Styles::gutter())
                        } else {
                            Span::styled(continuation_gutter.clone(), Styles::gutter())
                        };
                        
                        let line_content_width: usize = current_line_spans.iter()
                            .map(|s| s.content.chars().count())
                            .sum();
                        
                        let mut line_spans = vec![gutter_span];
                        line_spans.append(&mut current_line_spans);
                        
                        let padding_len = text_width.saturating_sub(line_content_width);
                        if padding_len > 0 {
                            line_spans.push(Span::styled(" ".repeat(padding_len), base_style));
                        }
                        
                        line_spans.push(Span::styled(" ", base_style));
                        
                        lines.push(Line::from(line_spans));
                        current_line_spans = Vec::new();
                        current_width = 0;
                        is_first_line = false;
                        continue;
                    }
                    
                    let take_count = remaining.min(unit_chars.len() - char_idx);
                    let segment: String = unit_chars[char_idx..char_idx + take_count].iter().collect();
                    
                    current_line_spans.push(Span::styled(segment, style));
                    current_width += take_count;
                    char_idx += take_count;
                }
            } else {
                // Unit fits, add it to current line
                current_line_spans.push(Span::styled(unit, style));
                current_width += unit_width;
            }
        }
    }

    // Add final line
    if !current_line_spans.is_empty() {
        let gutter_span = if is_first_line {
            Span::styled(gutter.clone(), Styles::gutter())
        } else {
            Span::styled(continuation_gutter.clone(), Styles::gutter())
        };
        
        // Calculate content width before moving
        let line_content_width: usize = current_line_spans.iter()
            .map(|s| s.content.chars().count())
            .sum();
        
        let mut line_spans = vec![gutter_span];
        line_spans.append(&mut current_line_spans);
        
        // Add padding to fill the line
        let padding_len = text_width.saturating_sub(line_content_width);
        if padding_len > 0 {
            line_spans.push(Span::styled(" ".repeat(padding_len), base_style));
        }
        
        // Add right margin
        line_spans.push(Span::styled(" ", base_style));
        
        lines.push(Line::from(line_spans));
    }

    // If no lines were created, create at least one empty line
    if lines.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(gutter.clone(), Styles::gutter()),
            Span::styled(" ".repeat(text_width), base_style),
            Span::styled(" ", base_style), // Right margin
        ]));
    }

    lines
}

fn create_blank_line(text_width: usize, gutter_width: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(" ".repeat(gutter_width), Styles::gutter()),
        Span::raw(" ".repeat(text_width)),
        Span::raw(" "), // Right margin
    ])
}

fn create_blank_line_with_style(
    text_width: usize,
    gutter_width: usize,
    background_style: ratatui::style::Style,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(" ".repeat(gutter_width), Styles::gutter()),
        Span::styled(" ".repeat(text_width), background_style),
        Span::styled(" ", background_style), // Right margin
    ])
}

fn create_fold_indicator(hidden_count: usize, text_width: usize, gutter_width: usize) -> Line<'static> {
    let text = format!("{} lines hidden", hidden_count);
    let padding_len = text_width.saturating_sub(text.len());

    Line::from(vec![
        Span::styled(" ".repeat(gutter_width), Styles::gutter()),
        Span::styled(text, Styles::fold_indicator()),
        Span::raw(" ".repeat(padding_len)),
        Span::raw(" "), // Right margin
    ])
}
