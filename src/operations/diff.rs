// Diff Engine
// Computes differences between source and destination directories

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Type of diff comparison being made
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffType {
    /// Comparing shared resources to project (shared -> project)
    SharedToProject,
    /// Comparing project to shared resources (project -> shared)
    ProjectToShared,
}

/// Status of a file in the diff
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    /// File exists only in source (will be added to destination)
    Added,
    /// File exists in both but content differs
    Modified,
    /// File exists only in destination (deleted from source)
    Deleted,
    /// File is not tracked
    Untracked,
    /// File is identical in both locations
    Unchanged,
}

/// A single diff entry representing a file difference
#[derive(Debug, Clone)]
pub struct DiffEntry {
    /// Relative path of the file
    pub path: PathBuf,
    /// Full path to source file
    pub source_path: PathBuf,
    /// Full path to destination file
    pub destination_path: PathBuf,
    /// Status of the file
    pub status: FileStatus,
    /// Type of diff this entry belongs to
    pub diff_type: DiffType,
}

/// Engine for computing directory differences
pub struct DiffEngine {
    /// Global exclude patterns
    exclude_patterns: Vec<String>,
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffEngine {
    /// Create a new diff engine with default exclude patterns
    pub fn new() -> Self {
        Self {
            exclude_patterns: vec![
                ".git".to_string(),
                "__pycache__".to_string(),
                ".pyc".to_string(),
                ".pyo".to_string(),
                ".pyd".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.swp".to_string(),
                "*.swo".to_string(),
                "*~".to_string(),
                ".uv".to_string(),
                "uv.lock".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
            ],
        }
    }
    
    /// Create with custom exclude patterns
    pub fn with_excludes(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns.extend(patterns);
        self
    }
    
    /// Compute differences between two directories
    pub fn compute_diff(
        &self,
        source_dir: &Path,
        dest_dir: &Path,
        diff_type: DiffType,
        additional_excludes: &[String],
    ) -> Result<Vec<DiffEntry>> {
        let mut diffs = Vec::new();
        
        // Combine all exclude patterns
        let all_excludes: Vec<&str> = self
            .exclude_patterns
            .iter()
            .chain(additional_excludes.iter())
            .map(|s| s.as_str())
            .collect();
        
        // Walk through source directory
        if source_dir.exists() {
            for entry in walkdir::WalkDir::new(source_dir)
                .into_iter()
                .filter_entry(|e| !Self::should_exclude(e.path(), &all_excludes))
                .filter_map(|e| e.ok())
            {
                let source_path = entry.path();
                
                if source_path.is_file() {
                    let relative_path = source_path
                        .strip_prefix(source_dir)
                        .context("Failed to calculate relative path")?;
                    
                    let dest_path = dest_dir.join(relative_path);
                    let status = Self::determine_status(source_path, &dest_path)?;
                    
                    // Only include files that need syncing
                    if status != FileStatus::Unchanged {
                        diffs.push(DiffEntry {
                            path: relative_path.to_path_buf(),
                            source_path: source_path.to_path_buf(),
                            destination_path: dest_path,
                            status,
                            diff_type: diff_type.clone(),
                        });
                    }
                }
            }
        }
        
        // Sort and deduplicate
        diffs.sort_by(|a, b| a.path.cmp(&b.path));
        diffs.dedup_by(|a, b| a.path == b.path);
        
        Ok(diffs)
    }
    
    /// Check if a path should be excluded
    fn should_exclude(path: &Path, patterns: &[&str]) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        patterns.iter().any(|pattern| {
            if pattern.starts_with('*') {
                path_str.ends_with(&pattern[1..])
            } else {
                path_str.contains(pattern)
            }
        })
    }
    
    /// Determine the status of a file
    fn determine_status(source: &Path, dest: &Path) -> Result<FileStatus> {
        let source_exists = source.exists();
        let dest_exists = dest.exists();
        
        match (source_exists, dest_exists) {
            (false, true) => Ok(FileStatus::Deleted),
            (true, false) => Ok(FileStatus::Added),
            (true, true) => {
                if Self::files_need_sync(source, dest)? {
                    Ok(FileStatus::Modified)
                } else {
                    Ok(FileStatus::Unchanged)
                }
            }
            (false, false) => Ok(FileStatus::Untracked),
        }
    }
    
    /// Check if files need to be synchronized
    fn files_need_sync(source: &Path, dest: &Path) -> Result<bool> {
        let source_meta = fs::metadata(source)?;
        let dest_meta = fs::metadata(dest)?;
        
        // Compare file sizes
        if source_meta.len() != dest_meta.len() {
            return Ok(true);
        }
        
        // Compare modification times
        let source_mtime = source_meta.modified()?;
        let dest_mtime = dest_meta.modified()?;
        
        if source_mtime > dest_mtime {
            return Ok(true);
        }
        
        // Compare content if times differ significantly
        let time_diff = source_mtime
            .duration_since(dest_mtime)
            .or_else(|_| dest_mtime.duration_since(source_mtime))
            .unwrap_or_default();
        
        if time_diff.as_secs_f64() > 1.0 {
            let source_content = fs::read(source)?;
            let dest_content = fs::read(dest)?;
            return Ok(source_content != dest_content);
        }
        
        // Final content check
        let source_content = fs::read(source)?;
        let dest_content = fs::read(dest)?;
        
        Ok(source_content != dest_content)
    }
    
    /// Load unified diff content for a diff entry
    pub fn load_diff_content(diff: &DiffEntry) -> Option<String> {
        // Try git diff first
        if let Ok(output) = Command::new("git")
            .args(["diff", "--no-index"])
            .arg(&diff.source_path)
            .arg(&diff.destination_path)
            .output()
        {
            if !output.stdout.is_empty() {
                return String::from_utf8(output.stdout).ok();
            }
        }
        
        // Fallback to simple diff
        match (
            fs::read_to_string(&diff.source_path),
            fs::read_to_string(&diff.destination_path),
        ) {
            (Ok(source), Ok(dest)) => Some(Self::generate_simple_diff(&source, &dest, &diff.source_path)),
            (Ok(source), Err(_)) => Some(format!(
                "--- {}\n+++ {}\n@@ -1,0 +0,0 @@\n{}",
                diff.source_path.display(),
                diff.destination_path.display(),
                source.lines().map(|l| format!("+{}", l)).collect::<Vec<_>>().join("\n")
            )),
            (Err(_), Ok(dest)) => Some(format!(
                "--- {}\n+++ {}\n@@ -0,0 +1,0 @@\n{}",
                diff.source_path.display(),
                diff.destination_path.display(),
                dest.lines().map(|l| format!("-{}", l)).collect::<Vec<_>>().join("\n")
            )),
            _ => None,
        }
    }
    
    /// Generate a simple line-by-line diff
    fn generate_simple_diff(source: &str, dest: &str, path: &Path) -> String {
        let source_lines: Vec<&str> = source.lines().collect();
        let dest_lines: Vec<&str> = dest.lines().collect();
        
        let mut diff_lines = vec![
            format!("--- {}", path.display()),
            format!("+++ {}", path.display()),
        ];
        
        let max_len = source_lines.len().max(dest_lines.len());
        for i in 0..max_len {
            let source_line = source_lines.get(i);
            let dest_line = dest_lines.get(i);
            
            match (source_line, dest_line) {
                (Some(s), Some(d)) if s == d => {
                    diff_lines.push(format!(" {}", s));
                }
                (Some(s), Some(d)) => {
                    diff_lines.push(format!("-{}", s));
                    diff_lines.push(format!("+{}", d));
                }
                (Some(s), None) => {
                    diff_lines.push(format!("-{}", s));
                }
                (None, Some(d)) => {
                    diff_lines.push(format!("+{}", d));
                }
                (None, None) => {}
            }
        }
        
        diff_lines.join("\n")
    }
}

// ============================================================================
// Line Alignment for Side-by-Side View
// ============================================================================

/// How lines are aligned between source and destination
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineAlignment {
    /// Lines exist in both files at given indices
    Both(usize, usize),
    /// Line only exists in source (removed)
    SourceOnly(usize),
    /// Line only exists in destination (added)
    DestOnly(usize),
}

/// Check if two lines are similar enough to show as modified (Both) vs separate (SourceOnly/DestOnly)
/// Lines are considered similar if they share significant content
fn lines_are_similar(line1: &str, line2: &str) -> bool {
    // Normalize empty/whitespace-only lines
    let norm1 = normalize_line_for_comparison(line1);
    let norm2 = normalize_line_for_comparison(line2);
    
    if norm1.is_empty() && norm2.is_empty() {
        return true;
    }
    if norm1.is_empty() || norm2.is_empty() {
        return false;
    }
    
    // Check if lines share significant word overlap
    let words1: std::collections::HashSet<&str> = norm1.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = norm2.split_whitespace().collect();
    
    let intersection: usize = words1.intersection(&words2).count();
    let union: usize = words1.union(&words2).count();
    
    if union == 0 {
        return false;
    }
    
    // If more than 30% of words overlap, consider them similar
    let similarity = intersection as f64 / union as f64;
    similarity > 0.3
}

/// Normalize a line for comparison (trim whitespace, treat empty/whitespace-only as empty)
fn normalize_line_for_comparison(line: &str) -> &str {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        ""
    } else {
        line
    }
}

/// Align lines between source and destination using LCS (Longest Common Subsequence)
/// This finds the optimal alignment by maximizing matching lines
/// CRITICAL: Blank lines at the same position are ALWAYS matched to preserve line number alignment
pub fn align_lines(source: &[String], dest: &[String]) -> Vec<LineAlignment> {
    let n = source.len();
    let m = dest.len();
    
    // STEP 1: Identify forced matches - blank lines at the same position MUST match
    let mut forced_matches = std::collections::HashSet::new();
    let min_len = n.min(m);
    for idx in 0..min_len {
        let src_norm = normalize_line_for_comparison(&source[idx]);
        let dest_norm = normalize_line_for_comparison(&dest[idx]);
        if src_norm.is_empty() && dest_norm.is_empty() {
            forced_matches.insert((idx, idx));
        }
    }
    
    // STEP 2: Build alignment by processing forced matches first, then LCS for the rest
    let mut aligned = Vec::new();
    let mut src_used = std::collections::HashSet::new();
    let mut dest_used = std::collections::HashSet::new();
    
    // First, add all forced matches
    let mut forced_alignments: Vec<(usize, usize)> = forced_matches.iter().copied().collect();
    forced_alignments.sort(); // Process in order
    
    for (src_idx, dest_idx) in forced_alignments {
        aligned.push(LineAlignment::Both(src_idx, dest_idx));
        src_used.insert(src_idx);
        dest_used.insert(dest_idx);
    }
    
    // STEP 3: Build remaining source and dest lines (excluding forced matches)
    let mut remaining_src: Vec<(usize, String)> = Vec::new();
    let mut remaining_dest: Vec<(usize, String)> = Vec::new();
    
    for i in 0..n {
        if !src_used.contains(&i) {
            remaining_src.push((i, source[i].clone()));
        }
    }
    
    for j in 0..m {
        if !dest_used.contains(&j) {
            remaining_dest.push((j, dest[j].clone()));
        }
    }
    
    // STEP 5: Run LCS on remaining lines
    if !remaining_src.is_empty() && !remaining_dest.is_empty() {
        let remaining_src_lines: Vec<String> = remaining_src.iter().map(|(_, s)| s.clone()).collect();
        let remaining_dest_lines: Vec<String> = remaining_dest.iter().map(|(_, s)| s.clone()).collect();
        
        let remaining_n = remaining_src_lines.len();
        let remaining_m = remaining_dest_lines.len();
        
        // DP table for remaining lines
        let mut dp = vec![vec![0u32; remaining_m + 1]; remaining_n + 1];
        
        // Fill DP table
            for i in 1..=remaining_n {
            for j in 1..=remaining_m {
                let src_line = &remaining_src_lines[i - 1];
                let dest_line = &remaining_dest_lines[j - 1];
                let src_norm = normalize_line_for_comparison(src_line);
                let dest_norm = normalize_line_for_comparison(dest_line);
                
                // Get original indices to check position
                let orig_src_idx = remaining_src[i - 1].0;
                let orig_dest_idx = remaining_dest[j - 1].0;
                
                if src_norm == dest_norm {
                    // For blank lines, only allow match if at same position in original files
                    if src_norm.is_empty() {
                        if orig_src_idx == orig_dest_idx {
                            // Same position - allow match
                            dp[i][j] = dp[i - 1][j - 1] + 1;
                        } else {
                            // Different positions - don't match blank lines
                            dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                        }
                    } else {
                        // Non-blank matching lines - always match
                        dp[i][j] = dp[i - 1][j - 1] + 1;
                    }
                } else {
                    // Check if lines are similar
                    let are_similar = lines_are_similar(src_line, dest_line);
                    if are_similar {
                        dp[i][j] = dp[i - 1][j - 1] + 1;
                    } else {
                        dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                    }
                }
            }
        }
        
        // Backtrack for remaining lines
        let mut i = remaining_n;
        let mut j = remaining_m;
        let mut remaining_aligned = Vec::new();
        
        while i > 0 || j > 0 {
            if i > 0 && j > 0 {
                let src_line = &remaining_src_lines[i - 1];
                let dest_line = &remaining_dest_lines[j - 1];
                let src_norm = normalize_line_for_comparison(src_line);
                let dest_norm = normalize_line_for_comparison(dest_line);
                
                if src_norm == dest_norm {
                    // For blank lines, only match if they're at the same position in original files
                    // This prevents matching blank lines that are at different positions
                    let orig_src_idx = remaining_src[i - 1].0;
                    let orig_dest_idx = remaining_dest[j - 1].0;
                    let should_match_blank = if src_norm.is_empty() {
                        // Only match blank lines if at same position in original files
                        orig_src_idx == orig_dest_idx
                    } else {
                        true // Non-blank matching lines can always match
                    };
                    
                    if should_match_blank && dp[i][j] == dp[i - 1][j - 1] + 1 {
                        remaining_aligned.push((i - 1, j - 1, true)); // Both
                        i -= 1;
                        j -= 1;
                    } else {
                        if dp[i - 1][j] >= dp[i][j - 1] {
                            remaining_aligned.push((i - 1, usize::MAX, false)); // SourceOnly
                            i -= 1;
                        } else {
                            remaining_aligned.push((usize::MAX, j - 1, false)); // DestOnly
                            j -= 1;
                        }
                    }
                } else {
                    let are_similar = lines_are_similar(src_line, dest_line);
                    if are_similar && dp[i][j] == dp[i - 1][j - 1] + 1 {
                        remaining_aligned.push((i - 1, j - 1, true)); // Both (modified)
                        i -= 1;
                        j -= 1;
                    } else if dp[i - 1][j] > dp[i][j - 1] {
                        remaining_aligned.push((i - 1, usize::MAX, false)); // SourceOnly
                        i -= 1;
                    } else if dp[i][j - 1] > dp[i - 1][j] {
                        remaining_aligned.push((usize::MAX, j - 1, false)); // DestOnly
                        j -= 1;
                    } else {
                        remaining_aligned.push((i - 1, usize::MAX, false)); // SourceOnly (tie)
                        i -= 1;
                    }
                }
            } else if i > 0 {
                remaining_aligned.push((i - 1, usize::MAX, false)); // SourceOnly
                i -= 1;
            } else if j > 0 {
                remaining_aligned.push((usize::MAX, j - 1, false)); // DestOnly
                j -= 1;
            }
        }
        
        remaining_aligned.reverse();
        
        // Convert remaining alignments back to original indices
        for (rem_src_idx, rem_dest_idx, is_both) in remaining_aligned {
            if is_both {
                let orig_src_idx = remaining_src[rem_src_idx].0;
                let orig_dest_idx = remaining_dest[rem_dest_idx].0;
                aligned.push(LineAlignment::Both(orig_src_idx, orig_dest_idx));
            } else if rem_dest_idx == usize::MAX {
                let orig_src_idx = remaining_src[rem_src_idx].0;
                aligned.push(LineAlignment::SourceOnly(orig_src_idx));
            } else {
                let orig_dest_idx = remaining_dest[rem_dest_idx].0;
                aligned.push(LineAlignment::DestOnly(orig_dest_idx));
            }
        }
    } else {
        // Handle remaining SourceOnly or DestOnly lines
        for (orig_idx, _) in remaining_src {
            aligned.push(LineAlignment::SourceOnly(orig_idx));
        }
        for (orig_idx, _) in remaining_dest {
            aligned.push(LineAlignment::DestOnly(orig_idx));
        }
    }
    
    // STEP 6: Post-process to handle blank lines following SourceOnly/DestOnly
    // Blank lines immediately following added/removed lines should also be SourceOnly/DestOnly
    // This matches VSCode behavior where blank lines below additions are shown as bright
    let mut processed_aligned = aligned;
    
    // Find SourceOnly lines and check if next line is a blank Both alignment
    for i in 0..processed_aligned.len() {
        match processed_aligned[i] {
            LineAlignment::SourceOnly(src_idx) => {
                // Look for a Both alignment at src_idx + 1 that's blank
                for j in 0..processed_aligned.len() {
                    if let LineAlignment::Both(next_src_idx, _) = processed_aligned[j] {
                        if next_src_idx == src_idx + 1 {
                            let next_src_line = &source[next_src_idx];
                            if normalize_line_for_comparison(next_src_line).is_empty() {
                                // Convert to SourceOnly
                                processed_aligned[j] = LineAlignment::SourceOnly(next_src_idx);
                                break;
                            }
                        }
                    }
                }
            }
            LineAlignment::DestOnly(dest_idx) => {
                // Look for a Both alignment at dest_idx + 1 that's blank
                for j in 0..processed_aligned.len() {
                    if let LineAlignment::Both(_, next_dest_idx) = processed_aligned[j] {
                        if next_dest_idx == dest_idx + 1 {
                            let next_dest_line = &dest[next_dest_idx];
                            if normalize_line_for_comparison(next_dest_line).is_empty() {
                                // Convert to DestOnly
                                processed_aligned[j] = LineAlignment::DestOnly(next_dest_idx);
                                break;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    // STEP 7: Sort alignments by source index to maintain order
    processed_aligned.sort_by(|a, b| {
        let a_src = match a {
            LineAlignment::Both(s, _) | LineAlignment::SourceOnly(s) => *s,
            LineAlignment::DestOnly(_) => usize::MAX, // Put DestOnly at end of their source position
        };
        let b_src = match b {
            LineAlignment::Both(s, _) | LineAlignment::SourceOnly(s) => *s,
            LineAlignment::DestOnly(_) => usize::MAX,
        };
        a_src.cmp(&b_src)
    });
    
    processed_aligned
}

/// Compute word-level diff for source line
/// Returns segments with (text, is_changed) where is_changed=true means this part was removed/changed
pub fn compute_word_diff_source(line: &str, other: &str) -> Vec<(String, bool)> {
    // Normalize empty/whitespace-only lines for comparison
    let line_norm = normalize_line_for_comparison(line);
    let other_norm = normalize_line_for_comparison(other);
    
    if line_norm == other_norm {
        return vec![(line.to_string(), false)];
    }
    
    // Special case: if source is a prefix of destination (text was added), source shows no changes
    if other.starts_with(line) {
        return vec![(line.to_string(), false)];
    }
    
    let line_words: Vec<&str> = line.split_inclusive(char::is_whitespace).collect();
    let other_words: Vec<&str> = other.split_inclusive(char::is_whitespace).collect();
    
    // Find longest common prefix
    let mut prefix_len = 0;
    let min_len = line_words.len().min(other_words.len());
    for i in 0..min_len {
        if line_words[i] == other_words[i] {
            prefix_len = i + 1;
        } else {
            break;
        }
    }
    
    // Find longest common suffix
    let mut suffix_len = 0;
    let max_suffix = (min_len - prefix_len)
        .min(line_words.len() - prefix_len)
        .min(other_words.len() - prefix_len);
    
    for i in 0..max_suffix {
        let line_idx = line_words.len() - 1 - i;
        let other_idx = other_words.len() - 1 - i;
        if line_idx >= prefix_len && other_idx >= prefix_len && line_words[line_idx] == other_words[other_idx] {
            suffix_len = i + 1;
        } else {
            break;
        }
    }
    
    let mut result = Vec::new();
    
    // Add prefix (unchanged)
    if prefix_len > 0 {
        let prefix: String = line_words[..prefix_len].concat();
        result.push((prefix, false));
    }
    
    // Add middle part (changed/removed in source)
    let middle_start = prefix_len;
    let middle_end = line_words.len() - suffix_len;
    if middle_start < middle_end {
        let middle: String = line_words[middle_start..middle_end].concat();
        result.push((middle, true));
    }
    
    // Add suffix (unchanged)
    if suffix_len > 0 {
        let suffix: String = line_words[line_words.len() - suffix_len..].concat();
        result.push((suffix, false));
    }
    
    result
}

/// Compute word-level diff for destination line
/// Returns segments with (text, is_changed) where is_changed=true means this part was added/changed
pub fn compute_word_diff_dest(line: &str, other: &str) -> Vec<(String, bool)> {
    // Normalize empty/whitespace-only lines for comparison
    let line_norm = normalize_line_for_comparison(line);
    let other_norm = normalize_line_for_comparison(other);
    
    if line_norm == other_norm {
        return vec![(line.to_string(), false)];
    }
    
    // Special case: if source is a prefix of destination (text was added), only highlight the added part
    if line.starts_with(other) {
        let added = &line[other.len()..];
        if !added.is_empty() {
            return vec![
                (other.to_string(), false),  // Original part (from source) unchanged
                (added.to_string(), true),    // Added part highlighted
            ];
        }
    }
    
    let line_words: Vec<&str> = line.split_inclusive(char::is_whitespace).collect();
    let other_words: Vec<&str> = other.split_inclusive(char::is_whitespace).collect();
    
    // Find longest common prefix
    let mut prefix_len = 0;
    let min_len = line_words.len().min(other_words.len());
    for i in 0..min_len {
        if line_words[i] == other_words[i] {
            prefix_len = i + 1;
        } else {
            break;
        }
    }
    
    // Find longest common suffix
    let mut suffix_len = 0;
    let max_suffix = (min_len - prefix_len)
        .min(line_words.len() - prefix_len)
        .min(other_words.len() - prefix_len);
    
    for i in 0..max_suffix {
        let line_idx = line_words.len() - 1 - i;
        let other_idx = other_words.len() - 1 - i;
        if line_idx >= prefix_len && other_idx >= prefix_len && line_words[line_idx] == other_words[other_idx] {
            suffix_len = i + 1;
        } else {
            break;
        }
    }
    
    let mut result = Vec::new();
    
    // Add prefix (unchanged)
    if prefix_len > 0 {
        let prefix: String = line_words[..prefix_len].concat();
        result.push((prefix, false));
    }
    
    // Add middle part (added/changed in destination)
    let middle_start = prefix_len;
    let middle_end = line_words.len() - suffix_len;
    if middle_start < middle_end {
        let middle: String = line_words[middle_start..middle_end].concat();
        result.push((middle, true));
    }
    
    // Add suffix (unchanged)
    if suffix_len > 0 {
        let suffix: String = line_words[line_words.len() - suffix_len..].concat();
        result.push((suffix, false));
    }
    
    result
}
