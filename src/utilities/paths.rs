// Path Utilities
// Helper functions for path manipulation

use std::path::{Path, PathBuf};

/// Normalize a path by resolving . and .. components
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            c => components.push(c),
        }
    }
    
    components.iter().collect()
}

/// Resolve a path relative to a base directory
pub fn resolve_path(base: &Path, relative: &str) -> PathBuf {
    if Path::new(relative).is_absolute() {
        PathBuf::from(relative)
    } else {
        normalize_path(&base.join(relative))
    }
}

/// Get the relative path from one path to another
pub fn relative_path(from: &Path, to: &Path) -> Option<PathBuf> {
    // Normalize both paths
    let from = normalize_path(from);
    let to = normalize_path(to);
    
    // Find common prefix
    let mut from_iter = from.components().peekable();
    let mut to_iter = to.components().peekable();
    
    while from_iter.peek() == to_iter.peek() && from_iter.peek().is_some() {
        from_iter.next();
        to_iter.next();
    }
    
    // Build relative path
    let mut result = PathBuf::new();
    
    // Add .. for remaining from components
    for _ in from_iter {
        result.push("..");
    }
    
    // Add remaining to components
    for component in to_iter {
        result.push(component);
    }
    
    Some(result)
}

/// Check if a path is inside a directory
pub fn is_inside(path: &Path, directory: &Path) -> bool {
    let path = normalize_path(path);
    let directory = normalize_path(directory);
    
    path.starts_with(&directory)
}

/// Get the file extension as a string
pub fn extension_str(path: &Path) -> Option<&str> {
    path.extension().and_then(|e| e.to_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_path() {
        let path = Path::new("a/b/../c/./d");
        assert_eq!(normalize_path(path), PathBuf::from("a/c/d"));
    }
    
    #[test]
    fn test_resolve_path() {
        let base = Path::new("/home/user");
        assert_eq!(resolve_path(base, "documents"), PathBuf::from("/home/user/documents"));
        assert_eq!(resolve_path(base, "/etc/config"), PathBuf::from("/etc/config"));
    }
}
