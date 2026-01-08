// Diff algorithms module
// Contains line alignment and character/word-level diff computation logic

/// Line alignment type for diff computation
#[derive(Debug, Clone)]
pub enum LineAlignment {
    Both(usize, usize),  // (source_idx, dest_idx)
    SourceOnly(usize),   // source_idx
    DestOnly(usize),     // dest_idx
}

/// Align lines using LCS-like algorithm
pub fn align_lines(source: &[String], dest: &[String]) -> Vec<LineAlignment> {
    let mut aligned = Vec::new();
    let mut src_idx = 0;
    let mut dest_idx = 0;
    
    const LOOKAHEAD_LIMIT: usize = 5;
    
    while src_idx < source.len() || dest_idx < dest.len() {
        if src_idx < source.len() && dest_idx < dest.len() && source[src_idx] == dest[dest_idx] {
            // Lines match - align them
            aligned.push(LineAlignment::Both(src_idx, dest_idx));
            src_idx += 1;
            dest_idx += 1;
        } else if src_idx < source.len() && dest_idx < dest.len() {
            // Lines don't match - check if we can find a match ahead
            let mut found_match = false;
            
            // Look ahead in source for a match with current dest line
            for ahead in 1..=LOOKAHEAD_LIMIT.min(source.len() - src_idx) {
                if src_idx + ahead < source.len() && source[src_idx + ahead] == dest[dest_idx] {
                    // Found match ahead - mark intermediate source lines as removed
                    for i in 0..ahead {
                        aligned.push(LineAlignment::SourceOnly(src_idx + i));
                    }
                    src_idx += ahead;
                    found_match = true;
                    break;
                }
            }
            
            if !found_match {
                // Look ahead in dest for a match with current source line
                for ahead in 1..=LOOKAHEAD_LIMIT.min(dest.len() - dest_idx) {
                    if dest_idx + ahead < dest.len() && source[src_idx] == dest[dest_idx + ahead] {
                        // Found match ahead - mark intermediate dest lines as added
                        for i in 0..ahead {
                            aligned.push(LineAlignment::DestOnly(dest_idx + i));
                        }
                        dest_idx += ahead;
                        found_match = true;
                        break;
                    }
                }
            }
            
            if !found_match {
                // No match found - mark as changed
                aligned.push(LineAlignment::Both(src_idx, dest_idx));
                src_idx += 1;
                dest_idx += 1;
            }
        } else if src_idx < source.len() {
            // Only source has more lines
            aligned.push(LineAlignment::SourceOnly(src_idx));
            src_idx += 1;
        } else {
            // Only dest has more lines
            aligned.push(LineAlignment::DestOnly(dest_idx));
            dest_idx += 1;
        }
    }
    
    aligned
}

/// Character-level diff for source side
pub fn diff_chars_source(line: &str, other: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let line_chars: Vec<char> = line.chars().collect();
    let other_chars: Vec<char> = other.chars().collect();
    
    // Find common prefix
    let mut prefix_len = 0;
    let min_len = line_chars.len().min(other_chars.len());
    for i in 0..min_len {
        if line_chars[i] == other_chars[i] {
            prefix_len = i + 1;
        } else {
            break;
        }
    }
    
    // Find common suffix
    let mut suffix_len = 0;
    let max_suffix_check = (min_len - prefix_len)
        .min(line_chars.len() - prefix_len)
        .min(other_chars.len() - prefix_len);
    for i in 0..max_suffix_check {
        let line_idx = line_chars.len() - 1 - i;
        let other_idx = other_chars.len() - 1 - i;
        if line_idx >= prefix_len && other_idx >= prefix_len && line_chars[line_idx] == other_chars[other_idx] {
            suffix_len = i + 1;
        } else {
            break;
        }
    }
    
    // Add unchanged prefix
    if prefix_len > 0 {
        result.push((line_chars[..prefix_len].iter().collect(), false));
    }
    
    let line_middle = &line_chars[prefix_len..line_chars.len() - suffix_len];
    let other_middle = &other_chars[prefix_len..other_chars.len() - suffix_len];
    
    let line_middle_str: String = line_middle.iter().collect();
    let other_middle_str: String = other_middle.iter().collect();
    
    if line_middle.is_empty() && !other_middle.is_empty() {
        // Pure insertion - nothing in line (source) is changed
    } else if !line_middle.is_empty() {
        // For source side, we want to show what's removed/changed
        if other_middle.is_empty() {
            result.push((line_middle_str, true));
        } else if line_middle_str == other_middle_str {
            result.push((line_middle_str, false));
        } else {
            result.push((line_middle_str, true));
        }
    }
    
    // Add unchanged suffix
    if suffix_len > 0 {
        result.push((line_chars[line_chars.len() - suffix_len..].iter().collect(), false));
    }
    
    result
}

/// Character-level diff for destination side
pub fn diff_chars_dest(line: &str, other: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let line_chars: Vec<char> = line.chars().collect();
    let other_chars: Vec<char> = other.chars().collect();
    
    // Find common prefix
    let mut prefix_len = 0;
    let min_len = line_chars.len().min(other_chars.len());
    for i in 0..min_len {
        if line_chars[i] == other_chars[i] {
            prefix_len = i + 1;
        } else {
            break;
        }
    }
    
    // Find common suffix
    let mut suffix_len = 0;
    let max_suffix_check = (min_len - prefix_len)
        .min(line_chars.len() - prefix_len)
        .min(other_chars.len() - prefix_len);
    for i in 0..max_suffix_check {
        let line_idx = line_chars.len() - 1 - i;
        let other_idx = other_chars.len() - 1 - i;
        if line_idx >= prefix_len && other_idx >= prefix_len && line_chars[line_idx] == other_chars[other_idx] {
            suffix_len = i + 1;
        } else {
            break;
        }
    }
    
    // Add unchanged prefix
    if prefix_len > 0 {
        result.push((line_chars[..prefix_len].iter().collect(), false));
    }
    
    let line_middle = &line_chars[prefix_len..line_chars.len() - suffix_len];
    let other_middle = &other_chars[prefix_len..other_chars.len() - suffix_len];
    
    let line_middle_str: String = line_middle.iter().collect();
    let other_middle_str: String = other_middle.iter().collect();
    
    if line_middle.is_empty() && !other_middle.is_empty() {
        result.push((other_middle_str, true));
    } else if !line_middle.is_empty() {
        if let Some(pos) = other_middle_str.find(&line_middle_str) {
            if pos == 0 {
                result.push((line_middle_str.clone(), false));
                let insertion = &other_middle_str[line_middle_str.len()..];
                if !insertion.is_empty() {
                    result.push((insertion.to_string(), true));
                }
            } else {
                let insertion = &other_middle_str[..pos];
                result.push((insertion.to_string(), true));
                result.push((line_middle_str.clone(), false));
                let after = &other_middle_str[pos + line_middle_str.len()..];
                if !after.is_empty() {
                    result.push((after.to_string(), true));
                }
            }
        } else if other_middle_str.starts_with(&line_middle_str) {
            result.push((line_middle_str.clone(), false));
            let insertion = &other_middle_str[line_middle_str.len()..];
            if !insertion.is_empty() {
                result.push((insertion.to_string(), true));
            }
        } else if line_middle_str.starts_with(&other_middle_str) {
            result.push((other_middle_str.clone(), false));
            let extra = &line_middle_str[other_middle_str.len()..];
            if !extra.is_empty() {
                result.push((extra.to_string(), true));
            }
        } else if other_middle_str.ends_with(&line_middle_str) {
            let insertion = &other_middle_str[..other_middle_str.len() - line_middle_str.len()];
            result.push((insertion.to_string(), true));
            result.push((line_middle_str, false));
        } else {
            result.push((line_middle_str, true));
        }
    }
    
    // Add unchanged suffix
    if suffix_len > 0 {
        result.push((line_chars[line_chars.len() - suffix_len..].iter().collect(), false));
    }
    
    result
}

/// Word-level diff for source side
pub fn compute_word_diff_source(line: &str, other: &str) -> Vec<(String, bool)> {
    if line == other {
        return vec![(line.to_string(), false)];
    }
    
    let line_words: Vec<&str> = line.split_inclusive(char::is_whitespace).collect();
    let other_words: Vec<&str> = other.split_inclusive(char::is_whitespace).collect();
    
    const WORD_DIFF_THRESHOLD: usize = 2;
    if line_words.len().abs_diff(other_words.len()) > WORD_DIFF_THRESHOLD {
        return diff_chars_source(line, other);
    }
    
    let mut result = Vec::new();
    let mut line_idx = 0;
    let mut other_idx = 0;
    
    const LOOKAHEAD_LIMIT: usize = 5;
    
    while line_idx < line_words.len() || other_idx < other_words.len() {
        if line_idx < line_words.len() && other_idx < other_words.len() {
            if line_words[line_idx] == other_words[other_idx] {
                result.push((line_words[line_idx].to_string(), false));
                line_idx += 1;
                other_idx += 1;
            } else {
                let mut found_match = false;
                for ahead in 1..=LOOKAHEAD_LIMIT.min(other_words.len() - other_idx) {
                    if other_idx + ahead < other_words.len() && line_words[line_idx] == other_words[other_idx + ahead] {
                        result.push((line_words[line_idx].to_string(), true));
                        line_idx += 1;
                        other_idx += ahead;
                        found_match = true;
                        break;
                    }
                }
                
                if !found_match {
                    for ahead in 1..=LOOKAHEAD_LIMIT.min(line_words.len() - line_idx) {
                        if line_idx + ahead < line_words.len() && other_words[other_idx] == line_words[line_idx + ahead] {
                            for i in 0..ahead {
                                result.push((line_words[line_idx + i].to_string(), true));
                            }
                            line_idx += ahead;
                            found_match = true;
                            break;
                        }
                    }
                }
                
                if !found_match {
                    let char_diffs = diff_chars_source(line_words[line_idx], other_words[other_idx]);
                    result.extend(char_diffs);
                    line_idx += 1;
                    other_idx += 1;
                }
            }
        } else if line_idx < line_words.len() {
            result.push((line_words[line_idx].to_string(), true));
            line_idx += 1;
        } else {
            other_idx += 1;
        }
    }
    
    result
}

/// Word-level diff for destination side
pub fn compute_word_diff_dest(line: &str, other: &str) -> Vec<(String, bool)> {
    if line == other {
        return vec![(line.to_string(), false)];
    }
    
    let line_words: Vec<&str> = line.split_inclusive(char::is_whitespace).collect();
    let other_words: Vec<&str> = other.split_inclusive(char::is_whitespace).collect();
    
    const WORD_DIFF_THRESHOLD: usize = 2;
    if line_words.len().abs_diff(other_words.len()) > WORD_DIFF_THRESHOLD {
        return diff_chars_dest(line, other);
    }
    
    let mut result = Vec::new();
    let mut line_idx = 0;
    let mut other_idx = 0;
    
    const LOOKAHEAD_LIMIT: usize = 5;
    
    while line_idx < line_words.len() || other_idx < other_words.len() {
        if line_idx < line_words.len() && other_idx < other_words.len() {
            if line_words[line_idx] == other_words[other_idx] {
                result.push((line_words[line_idx].to_string(), false));
                line_idx += 1;
                other_idx += 1;
            } else {
                let mut found_in_other = None;
                for ahead in 1..=LOOKAHEAD_LIMIT.min(other_words.len() - other_idx) {
                    if other_idx + ahead < other_words.len() && line_words[line_idx] == other_words[other_idx + ahead] {
                        found_in_other = Some(ahead);
                        break;
                    }
                }
                
                let mut found_in_line = None;
                for ahead in 1..=LOOKAHEAD_LIMIT.min(line_words.len() - line_idx) {
                    if line_idx + ahead < line_words.len() && other_words[other_idx] == line_words[line_idx + ahead] {
                        found_in_line = Some(ahead);
                        break;
                    }
                }
                
                match (found_in_other, found_in_line) {
                    (Some(ahead), _) => {
                        result.push((line_words[line_idx].to_string(), false));
                        line_idx += 1;
                        other_idx += ahead;
                    }
                    (_, Some(ahead)) => {
                        for i in 0..ahead {
                            result.push((line_words[line_idx + i].to_string(), true));
                        }
                        line_idx += ahead;
                        other_idx += 1;
                    }
                    (None, None) => {
                        let char_diffs = diff_chars_dest(line_words[line_idx], other_words[other_idx]);
                        result.extend(char_diffs);
                        line_idx += 1;
                        other_idx += 1;
                    }
                }
            }
        } else if line_idx < line_words.len() {
            result.push((line_words[line_idx].to_string(), true));
            line_idx += 1;
        } else {
            other_idx += 1;
        }
    }
    
    result
}
