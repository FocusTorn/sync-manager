// Debug script to show diff alignment and rendering output

use std::fs;
use std::path::PathBuf;
use sync_manager::operations::diff::align_lines;

fn main() {
    // Load the two files
    let shared_file = PathBuf::from("_shared-resources/shared-cursor/rules/code-quality.mdc");
    let project_file = PathBuf::from(".cursor/rules/code-quality.mdc");
    
    let shared_content = fs::read_to_string(&shared_file)
        .expect("Failed to read shared file");
    let project_content = fs::read_to_string(&project_file)
        .expect("Failed to read project file");
    
    let shared_lines: Vec<String> = shared_content.lines().map(|s| s.to_string()).collect();
    let project_lines: Vec<String> = project_content.lines().map(|s| s.to_string()).collect();
    
    println!("=== FILE INFO ===");
    println!("Shared file: {} lines", shared_lines.len());
    println!("Project file: {} lines", project_lines.len());
    println!();
    
    // Show first few lines with indices
    println!("=== SHARED FILE (first 12 lines) ===");
    for (i, line) in shared_lines.iter().take(12).enumerate() {
        let preview = if line.trim().is_empty() {
            "[BLANK]".to_string()
        } else if line.len() > 60 {
            format!("{}...", &line[..60])
        } else {
            line.clone()
        };
        println!("  [{}] {}", i, preview);
    }
    println!();
    
    println!("=== PROJECT FILE (first 12 lines) ===");
    for (i, line) in project_lines.iter().take(12).enumerate() {
        let preview = if line.trim().is_empty() {
            "[BLANK]".to_string()
        } else if line.len() > 60 {
            format!("{}...", &line[..60])
        } else {
            line.clone()
        };
        println!("  [{}] {}", i, preview);
    }
    println!();
    
    // Check for forced matches before alignment
    println!("=== CHECKING FOR FORCED MATCHES (blank lines at same position) ===");
    let min_len = shared_lines.len().min(project_lines.len());
    for idx in 0..min_len {
        let src_trimmed = shared_lines[idx].trim();
        let dest_trimmed = project_lines[idx].trim();
        if src_trimmed.is_empty() && dest_trimmed.is_empty() {
            println!("  Index {} (line {}): Both are blank - SHOULD BE FORCED MATCH", idx, idx + 1);
        }
    }
    println!();
    
    // Run alignment
    println!("=== ALIGNMENT RESULTS ===");
    let aligned = align_lines(&shared_lines, &project_lines);
    
    println!("Total alignments: {}", aligned.len());
    println!();
    
    // Show alignment details
    for (idx, alignment) in aligned.iter().enumerate() {
        match alignment {
            sync_manager::operations::diff::LineAlignment::Both(src_idx, dest_idx) => {
                let src_line = &shared_lines[*src_idx];
                let dest_line = &project_lines[*dest_idx];
                let src_preview = if src_line.trim().is_empty() {
                    "[BLANK]"
                } else if src_line.len() > 40 {
                    "LONG LINE..."
                } else {
                    src_line
                };
                let dest_preview = if dest_line.trim().is_empty() {
                    "[BLANK]"
                } else if dest_line.len() > 40 {
                    "LONG LINE..."
                } else {
                    dest_line
                };
                println!("  [{}] Both(src[{}]={}, dest[{}]={})", 
                    idx, src_idx, src_preview, dest_idx, dest_preview);
            }
            sync_manager::operations::diff::LineAlignment::SourceOnly(src_idx) => {
                let src_line = &shared_lines[*src_idx];
                let src_preview = if src_line.trim().is_empty() {
                    "[BLANK]"
                } else if src_line.len() > 40 {
                    "LONG LINE..."
                } else {
                    src_line
                };
                println!("  [{}] SourceOnly(src[{}]={})", idx, src_idx, src_preview);
            }
            sync_manager::operations::diff::LineAlignment::DestOnly(dest_idx) => {
                let dest_line = &project_lines[*dest_idx];
                let dest_preview = if dest_line.trim().is_empty() {
                    "[BLANK]"
                } else if dest_line.len() > 40 {
                    "LONG LINE..."
                } else {
                    dest_line
                };
                println!("  [{}] DestOnly(dest[{}]={})", idx, dest_idx, dest_preview);
            }
        }
    }
    println!();
    
    // Check specifically for line 8 (index 7)
    println!("=== CHECKING LINE 8 (index 7) ===");
    println!("Shared line 8 (index 7): {:?}", 
        if shared_lines.get(7).map(|s| s.trim().is_empty()).unwrap_or(false) {
            "[BLANK]"
        } else {
            shared_lines.get(7).map(|s| s.as_str()).unwrap_or("MISSING")
        });
    println!("Project line 8 (index 7): {:?}", 
        if project_lines.get(7).map(|s| s.trim().is_empty()).unwrap_or(false) {
            "[BLANK]"
        } else {
            project_lines.get(7).map(|s| s.as_str()).unwrap_or("MISSING")
        });
    
    // Find alignment for index 7 - check all alignments
    let mut found_both = false;
    let mut src7_alignment = None;
    let mut dest7_alignment = None;
    
    for (idx, alignment) in aligned.iter().enumerate() {
        match alignment {
            sync_manager::operations::diff::LineAlignment::Both(src_idx, dest_idx) => {
                if *src_idx == 7 && *dest_idx == 7 {
                    println!("  ✓ Found Both(7, 7) alignment at position [{}]!", idx);
                    found_both = true;
                }
                if *src_idx == 7 {
                    src7_alignment = Some(format!("Both(src[7], dest[{}]) at [{}]", dest_idx, idx));
                }
                if *dest_idx == 7 {
                    dest7_alignment = Some(format!("Both(src[{}], dest[7]) at [{}]", src_idx, idx));
                }
            }
            sync_manager::operations::diff::LineAlignment::SourceOnly(src_idx) => {
                if *src_idx == 7 {
                    println!("  ✗ Line 7 is SourceOnly at position [{}] (not matched)", idx);
                    src7_alignment = Some(format!("SourceOnly(src[7]) at [{}]", idx));
                }
            }
            sync_manager::operations::diff::LineAlignment::DestOnly(dest_idx) => {
                if *dest_idx == 7 {
                    println!("  ✗ Line 7 is DestOnly at position [{}] (not matched)", idx);
                    dest7_alignment = Some(format!("DestOnly(dest[7]) at [{}]", idx));
                }
            }
        }
    }
    
    if !found_both {
        println!("  ✗ No Both(7, 7) alignment found!");
        if let Some(ref align) = src7_alignment {
            println!("  → src[7] alignment: {}", align);
        }
        if let Some(ref align) = dest7_alignment {
            println!("  → dest[7] alignment: {}", align);
        }
    }
    println!();
    
    // Simulate what would be rendered
    println!("=== SIMULATED RENDERING (first 12 alignments) ===");
    
    for (idx, alignment) in aligned.iter().take(12).enumerate() {
        match alignment {
            sync_manager::operations::diff::LineAlignment::Both(src_idx, dest_idx) => {
                let src_line = &shared_lines[*src_idx];
                let dest_line = &project_lines[*dest_idx];
                let src_is_blank = src_line.trim().is_empty();
                let dest_is_blank = dest_line.trim().is_empty();
                
                println!("  Alignment [{}]:", idx);
                println!("    Left:  Line {} (src[{}]) = {}", 
                    src_idx + 1, src_idx, 
                    if src_is_blank { "[BLANK]" } else { "TEXT" });
                println!("    Right: Line {} (dest[{}]) = {}", 
                    dest_idx + 1, dest_idx,
                    if dest_is_blank { "[BLANK]" } else { "TEXT" });
                
                if *src_idx == 7 && *dest_idx == 7 {
                    println!("    *** THIS IS LINE 8 - SHOULD BE RENDERED ***");
                }
            }
            sync_manager::operations::diff::LineAlignment::SourceOnly(src_idx) => {
                let src_line = &shared_lines[*src_idx];
                println!("  Alignment [{}]:", idx);
                println!("    Left:  Line {} (src[{}]) = {} [ADDED]", 
                    src_idx + 1, src_idx,
                    if src_line.trim().is_empty() { "[BLANK]" } else { "TEXT" });
                println!("    Right: [BLANK SPACE - NO LINE NUMBER]");
            }
            sync_manager::operations::diff::LineAlignment::DestOnly(dest_idx) => {
                let dest_line = &project_lines[*dest_idx];
                println!("  Alignment [{}]:", idx);
                println!("    Left:  [BLANK SPACE - NO LINE NUMBER]");
                println!("    Right: Line {} (dest[{}]) = {} [ADDED]", 
                    dest_idx + 1, dest_idx,
                    if dest_line.trim().is_empty() { "[BLANK]" } else { "TEXT" });
            }
        }
    }
}
