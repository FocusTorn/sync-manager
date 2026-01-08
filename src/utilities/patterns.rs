// Pattern Matching Utilities
// Glob-like pattern matching for file exclusions

use std::path::Path;

/// Check if a path matches a pattern
pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    let pattern = pattern.to_lowercase();
    
    if pattern.starts_with('*') {
        // Wildcard at start: match suffix
        path_str.ends_with(&pattern[1..])
    } else if pattern.ends_with('*') {
        // Wildcard at end: match prefix
        path_str.starts_with(&pattern[..pattern.len() - 1])
    } else if pattern.contains('*') {
        // Wildcard in middle: simple contains check
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            path_str.starts_with(parts[0]) && path_str.ends_with(parts[1])
        } else {
            path_str.contains(&pattern.replace('*', ""))
        }
    } else {
        // No wildcard: direct contains check
        path_str.contains(&pattern)
    }
}

/// Pattern matcher for file exclusions
pub struct PatternMatcher {
    patterns: Vec<String>,
}

impl PatternMatcher {
    /// Create a new pattern matcher with the given patterns
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }
    
    /// Check if a path should be excluded
    pub fn should_exclude(&self, path: &Path) -> bool {
        self.patterns.iter().any(|p| matches_pattern(path, p))
    }
    
    /// Add a pattern
    pub fn add_pattern(&mut self, pattern: String) {
        self.patterns.push(pattern);
    }
    
    /// Get all patterns
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_suffix_pattern() {
        assert!(matches_pattern(Path::new("file.txt"), "*.txt"));
        assert!(!matches_pattern(Path::new("file.md"), "*.txt"));
    }
    
    #[test]
    fn test_prefix_pattern() {
        assert!(matches_pattern(Path::new("test_file.rs"), "test_*"));
        assert!(!matches_pattern(Path::new("file_test.rs"), "test_*"));
    }
    
    #[test]
    fn test_contains_pattern() {
        assert!(matches_pattern(Path::new("path/to/node_modules/file"), "node_modules"));
        assert!(!matches_pattern(Path::new("path/to/src/file"), "node_modules"));
    }
    
    #[test]
    fn test_pattern_matcher() {
        let matcher = PatternMatcher::new(vec![
            "*.swp".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
        ]);
        
        assert!(matcher.should_exclude(Path::new("file.swp")));
        assert!(matcher.should_exclude(Path::new("project/node_modules/pkg")));
        assert!(matcher.should_exclude(Path::new(".git/config")));
        assert!(!matcher.should_exclude(Path::new("src/main.rs")));
    }
}
