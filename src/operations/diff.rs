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

/// Check if two lines are equal (normalizing empty/whitespace-only lines)
fn lines_equal_normalized(line1: &str, line2: &str) -> bool {
    let norm1 = normalize_line_for_comparison(line1);
    let norm2 = normalize_line_for_comparison(line2);
    norm1 == norm2
}

/// Align lines between source and destination using LCS (Longest Common Subsequence)
/// This finds the optimal alignment by maximizing matching lines
pub fn align_lines(source: &[String], dest: &[String]) -> Vec<LineAlignment> {
    let n = source.len();
    let m = dest.len();
    
    // dp[i][j] = length of LCS of source[0..i] and dest[0..j]
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    
    // Fill DP table - normalize empty/whitespace-only lines for comparison
    // Allow empty lines to match more freely, but prefer matching non-empty lines
    for i in 1..=n {
        for j in 1..=m {
            let src_norm = normalize_line_for_comparison(&source[i - 1]);
            let dest_norm = normalize_line_for_comparison(&dest[j - 1]);
            
            if src_norm == dest_norm {
                // Lines match (normalized)
                // For empty lines, always match if at same position, otherwise prefer matching if close
                if src_norm.is_empty() {
                    // Empty lines at the same position should always match
                    if i == j {
                        // Same position - always match blank lines
                        dp[i][j] = dp[i - 1][j - 1] + 1;
                    } else {
                        // Different positions - prefer matching if close or if it improves LCS
                        let match_score = dp[i - 1][j - 1] + 1;
                        let skip_score = dp[i - 1][j].max(dp[i][j - 1]);
                        let positions_close = (i as i32 - j as i32).abs() <= 2;
                        if positions_close || match_score > skip_score {
                            dp[i][j] = match_score;
                        } else {
                            dp[i][j] = skip_score;
                        }
                    }
                } else {
                    // Non-empty matching lines - always match them
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                }
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }
    
    // Backtrack to build alignment
    // Standard LCS backtracking - empty lines only match at same position (enforced in DP table)
    let mut aligned = Vec::new();
    let mut i = n;
    let mut j = m;
    
    while i > 0 || j > 0 {
        if i > 0 && j > 0 {
            let src_norm = normalize_line_for_comparison(&source[i - 1]);
            let dest_norm = normalize_line_for_comparison(&dest[j - 1]);
            
            if src_norm == dest_norm {
                // Lines match (normalized)
                // For blank lines at the same position, always match them
                if src_norm.is_empty() && i == j {
                    // Blank lines at same position - always match
                    aligned.push(LineAlignment::Both(i - 1, j - 1));
                    i -= 1;
                    j -= 1;
                } else if dp[i][j] == dp[i - 1][j - 1] + 1 {
                    // This match was part of the LCS
                    aligned.push(LineAlignment::Both(i - 1, j - 1));
                    i -= 1;
                    j -= 1;
                } else {
                    // This match wasn't part of the LCS - choose path that maintains LCS
                    if dp[i - 1][j] >= dp[i][j - 1] {
                        aligned.push(LineAlignment::SourceOnly(i - 1));
                        i -= 1;
                    } else {
                        aligned.push(LineAlignment::DestOnly(j - 1));
                        j -= 1;
                    }
                }
            } else {
                // Lines don't match - choose path that maintains LCS
                // Check if lines are similar enough to show as modified (Both) vs separate (SourceOnly/DestOnly)
                let src_line = &source[i - 1];
                let dest_line = &dest[j - 1];
                
                // If lines share significant content, show as Both (modified) for word-level diff
                // Otherwise, show as separate insertions/deletions
                let are_similar = lines_are_similar(src_line, dest_line);
                
                if are_similar {
                    // Lines are similar - show as Both (modified) for word-level highlighting
                    aligned.push(LineAlignment::Both(i - 1, j - 1));
                    i -= 1;
                    j -= 1;
                } else if dp[i - 1][j] > dp[i][j - 1] {
                    // Removing from source maintains better LCS
                    aligned.push(LineAlignment::SourceOnly(i - 1));
                    i -= 1;
                } else if dp[i][j - 1] > dp[i - 1][j] {
                    // Removing from dest maintains better LCS
                    aligned.push(LineAlignment::DestOnly(j - 1));
                    j -= 1;
                } else {
                    // Tie - prefer showing as separate changes
                    aligned.push(LineAlignment::SourceOnly(i - 1));
                    i -= 1;
                }
            }
        } else if i > 0 {
            // Only source has lines left
            aligned.push(LineAlignment::SourceOnly(i - 1));
            i -= 1;
        } else if j > 0 {
            // Only dest has lines left
            aligned.push(LineAlignment::DestOnly(j - 1));
            j -= 1;
        } else {
            break;
        }
    }
    
    // Reverse because we built backwards
    aligned.reverse();
    aligned
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
