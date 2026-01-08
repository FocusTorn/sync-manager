// Split diff view component
// Side-by-side diff view with word-level highlighting, folding, and scrolling
//
// Layout Algorithm:
// - Split view divides available width in half (SPLIT_RATIO = 2)
// - Source panel positioned at (parent.x+1, parent.y+1) with half width minus borders
// - Destination panel positioned to the right of source with same height
// - Gutter width calculated based on maximum line number digit count
// - Text width = panel width - gutter width - border offset

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::core::RectRegistry;
use crate::managers::{BoundingBox, SplitDiffManager};
use crate::utilities::LayoutCalculator;

/// Error type for layout constants validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutConstantsError {
    /// Split ratio must be greater than zero
    InvalidSplitRatio,
    /// Border width must be at least 2 (top + bottom or left + right)
    InvalidBorderWidth,
    /// Min line digits must be at least 1
    InvalidMinLineDigits,
}

impl std::fmt::Display for LayoutConstantsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutConstantsError::InvalidSplitRatio => {
                write!(f, "Split ratio must be greater than zero")
            }
            LayoutConstantsError::InvalidBorderWidth => {
                write!(f, "Border width must be at least 2")
            }
            LayoutConstantsError::InvalidMinLineDigits => {
                write!(f, "Minimum line digits must be at least 1")
            }
        }
    }
}

impl std::error::Error for LayoutConstantsError {}

/// Layout configuration constants
/// Fields are private to ensure validation - use getters to access values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutConstants {
    /// Offset from parent border (used for positioning panels)
    border_offset: u16,
    /// Total border width (top + bottom or left + right)
    border_width: u16,
    /// Split ratio for dividing width (2 = 50/50 split)
    split_ratio: u16,
    /// Space after line number digits in gutter
    gutter_space_after_digits: usize,
    /// Minimum digits for line numbers when no lines exist
    min_line_digits: usize,
}

impl Default for LayoutConstants {
    fn default() -> Self {
        // Safe to unwrap - default values are valid
        Self::new(
            1,  // border_offset
            2,  // border_width
            2,  // split_ratio (50/50 split)
            1,  // gutter_space_after_digits
            1,  // min_line_digits
        ).expect("Default layout constants should always be valid")
    }
}

impl LayoutConstants {
    /// Create a new layout constants configuration with validation
    /// Returns an error if validation fails
    pub fn new(
        border_offset: u16,
        border_width: u16,
        split_ratio: u16,
        gutter_space_after_digits: usize,
        min_line_digits: usize,
    ) -> Result<Self, LayoutConstantsError> {
        // Validate split_ratio
        if split_ratio == 0 {
            return Err(LayoutConstantsError::InvalidSplitRatio);
        }
        
        // Validate border_width (should be at least 2 for top+bottom or left+right)
        if border_width < 2 {
            return Err(LayoutConstantsError::InvalidBorderWidth);
        }
        
        // Validate min_line_digits
        if min_line_digits == 0 {
            return Err(LayoutConstantsError::InvalidMinLineDigits);
        }
        
        Ok(Self {
            border_offset,
            border_width,
            split_ratio,
            gutter_space_after_digits,
            min_line_digits,
        })
    }
    
    /// Get border offset
    pub fn border_offset(&self) -> u16 {
        self.border_offset
    }
    
    /// Get border width
    pub fn border_width(&self) -> u16 {
        self.border_width
    }
    
    /// Get split ratio
    pub fn split_ratio(&self) -> u16 {
        self.split_ratio
    }
    
    /// Get gutter space after digits
    pub fn gutter_space_after_digits(&self) -> usize {
        self.gutter_space_after_digits
    }
    
    /// Get minimum line digits
    pub fn min_line_digits(&self) -> usize {
        self.min_line_digits
    }
}

/// Default layout constants (50/50 split with standard borders)
/// This constant is safe because the values are validated (split_ratio=2, border_width=2, min_line_digits=1)
/// For runtime construction with validation, use LayoutConstants::default() or LayoutConstants::new()
/// 
/// Note: This uses a const fn to construct with private fields - the values are validated constants
pub const DEFAULT_LAYOUT_CONSTANTS: LayoutConstants = {
    // Helper const fn to construct LayoutConstants - only used for known-valid constants
    const fn make_constants(
        border_offset: u16,
        border_width: u16,
        split_ratio: u16,
        gutter_space_after_digits: usize,
        min_line_digits: usize,
    ) -> LayoutConstants {
        // These values are validated as constants (split_ratio=2>0, border_width=2>=2, min_line_digits=1>=1)
        LayoutConstants {
            border_offset,
            border_width,
            split_ratio,
            gutter_space_after_digits,
            min_line_digits,
        }
    }
    make_constants(1, 2, 2, 1, 1)
};

/// Configuration for the split diff view
/// Contains only immutable configuration (titles)
/// Runtime state (scroll_offset, fold_unchanged) is stored in SplitDiffViewState
#[derive(Debug, Clone)]
pub struct SplitDiffViewConfig {
    pub source_title: String,
    pub dest_title: String,
    pub layout_constants: LayoutConstants,
    /// File extension for syntax highlighting (e.g., "rs", "py", "js")
    /// If None, no syntax highlighting will be applied
    pub file_extension: Option<String>,
}

impl Default for SplitDiffViewConfig {
    fn default() -> Self {
        Self {
            source_title: "Source".to_string(),
            dest_title: "Destination".to_string(),
            layout_constants: DEFAULT_LAYOUT_CONSTANTS,
            file_extension: None,
        }
    }
}

impl SplitDiffViewConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: Set source title
    /// Uses Cow<str> internally to reduce allocations when string slices are provided
    pub fn with_source_title(mut self, title: impl Into<String>) -> Self {
        self.source_title = title.into();
        self
    }

    /// Builder: Set destination title
    /// Uses Cow<str> internally to reduce allocations when string slices are provided
    pub fn with_dest_title(mut self, title: impl Into<String>) -> Self {
        self.dest_title = title.into();
        self
    }

    /// Builder: Set layout constants
    pub fn with_layout_constants(mut self, constants: LayoutConstants) -> Self {
        self.layout_constants = constants;
        self
    }

    /// Builder: Set file extension for syntax highlighting
    /// Extension should be without leading dot (e.g., "rs" not ".rs")
    pub fn with_file_extension(mut self, extension: impl Into<String>) -> Self {
        self.file_extension = Some(extension.into());
        self
    }

    /// Builder: Set file extension from file path (extracts extension automatically)
    pub fn with_file_path(mut self, file_path: &str) -> Self {
        use crate::utilities::get_file_extension;
        self.file_extension = get_file_extension(file_path);
        self
    }
}

/// State for the split diff view
#[derive(Debug)]
pub struct SplitDiffViewState {
    pub scroll_offset: usize,
    pub fold_unchanged: bool,
    /// Cached gutter width calculation (invalidated when line count changes)
    cached_gutter_width: Option<(usize, usize, usize, usize)>, // (source_lines, dest_lines, gutter_width, max_line_digits)
}

impl Default for SplitDiffViewState {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            fold_unchanged: true,
            cached_gutter_width: None,
        }
    }
}

impl SplitDiffViewState {
    /// Calculate the number of digits needed to represent a number (integer log10)
    /// Faster than floating point log10 calculation
    fn calculate_digit_count(num: usize) -> usize {
        if num == 0 {
            return 1;
        }
        let mut n = num;
        let mut digits = 0;
        while n > 0 {
            n /= 10;
            digits += 1;
        }
        digits
    }

    /// Get cached gutter width if line counts match, otherwise calculate and cache
    pub fn get_gutter_width(
        &mut self,
        source_line_count: usize,
        dest_line_count: usize,
        constants: &LayoutConstants,
    ) -> (usize, usize) {
        // Check cache validity
        if let Some((cached_source, cached_dest, cached_width, cached_digits)) = self.cached_gutter_width {
            if cached_source == source_line_count && cached_dest == dest_line_count {
                return (cached_width, cached_digits);
            }
        }

        // Calculate and cache
        let max_line_num = source_line_count.max(dest_line_count);
        let max_line_digits = if max_line_num == 0 {
            constants.min_line_digits()
        } else {
            Self::calculate_digit_count(max_line_num)
        };
        let gutter_width = max_line_digits + constants.gutter_space_after_digits();
        
        self.cached_gutter_width = Some((source_line_count, dest_line_count, gutter_width, max_line_digits));
        (gutter_width, max_line_digits)
    }
}

/// Error type for split diff view operations
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum SplitDiffViewError {
    /// Bounding box preparation failed
    #[error("Failed to prepare bounding box")]
    BoxPreparationFailed,
    /// Invalid area calculation
    #[error("Invalid area calculation")]
    InvalidArea,
}

/// Window handle names for nested bounding boxes
const HWND_SOURCE_CONTENT: &str = "hwndSourceContent";
const HWND_DEST_CONTENT: &str = "hwndDestContent";

/// Split diff view component
pub struct SplitDiffView<'a> {
    config: &'a SplitDiffViewConfig,
    state: &'a mut SplitDiffViewState,
    source_lines: &'a [String],
    dest_lines: &'a [String],
    layout_calculator: LayoutCalculator,
}

impl<'a> SplitDiffView<'a> {
    pub fn new(
        config: &'a SplitDiffViewConfig,
        state: &'a mut SplitDiffViewState,
        source_lines: &'a [String],
        dest_lines: &'a [String],
    ) -> Self {
        let layout_calculator = LayoutCalculator::new(config.layout_constants);
        Self {
            config,
            state,
            source_lines,
            dest_lines,
            layout_calculator,
        }
    }

    /// Get or create a bounding box with the given name and area
    /// Uses method-based API on BoundingBox instead of helper function
    fn get_or_create_box(
        registry: &mut RectRegistry,
        name: &str,
        area: Rect,
    ) -> Result<BoundingBox, SplitDiffViewError> {
        // Use method-based API: BoundingBox::from_handle_name() instead of get_box_by_name()
        if let Some(box_handle) = BoundingBox::from_handle_name(registry, name) {
            // Update the existing box with new area
            if box_handle.update(registry, area) {
                Ok(box_handle)
            } else {
                Err(SplitDiffViewError::BoxPreparationFailed)
            }
        } else {
            Ok(BoundingBox::create(registry, name, area))
        }
    }

    /// Render the split diff view into a bounding box
    pub fn render(
        &mut self,
        f: &mut Frame,
        bounding_box: &BoundingBox,
        registry: &mut RectRegistry,
    ) -> Result<(), SplitDiffViewError> {
        let area = bounding_box
            .prepare(registry)
            .ok_or(SplitDiffViewError::BoxPreparationFailed)?;

        // Calculate source panel area: positioned at (x+offset, y+offset) with half width minus borders
        let source_content_area = self.layout_calculator.calculate_source_area(area);
        let source_box = Self::get_or_create_box(registry, HWND_SOURCE_CONTENT, source_content_area)?;

        // Calculate destination panel area: positioned to the right of source panel
        let dest_content_area = self
            .layout_calculator
            .calculate_dest_area(source_content_area, area.width);
        let dest_box = Self::get_or_create_box(registry, HWND_DEST_CONTENT, dest_content_area)?;

        // Render using the nested bounding boxes
        self.render_in_area(f, area, &source_box, &dest_box, registry)?;
        Ok(())
    }

    /// Render the split diff view in a given area
    /// This method delegates all business logic to SplitDiffManager
    pub fn render_in_area(
        &mut self,
        f: &mut Frame,
        area: Rect,
        source_box: &BoundingBox,
        dest_box: &BoundingBox,
        registry: &mut RectRegistry,
    ) -> Result<(), SplitDiffViewError> {
        // Handle empty content case
        if self.source_lines.is_empty() && self.dest_lines.is_empty() {
            let loading = Paragraph::new("No content to display")
                .block(Block::default().borders(Borders::ALL).title("Side-by-Side Diff"));
            f.render_widget(loading, area);
            return Ok(());
        }

        // Get the actual content areas from bounding boxes with proper error handling
        let source_content_area = source_box
            .prepare(registry)
            .ok_or(SplitDiffViewError::BoxPreparationFailed)?;
        let dest_content_area = dest_box
            .prepare(registry)
            .ok_or(SplitDiffViewError::BoxPreparationFailed)?;

        // Calculate available height accounting for block borders (top and bottom)
        let available_height = self
            .layout_calculator
            .calculate_available_height(source_content_area);

        // Calculate gutter width using cached calculation (performance optimization)
        let source_line_count = self.source_lines.len();
        let dest_line_count = self.dest_lines.len();
        let (gutter_width, max_line_digits) = self
            .state
            .get_gutter_width(source_line_count, dest_line_count, &self.config.layout_constants);

        // Calculate text width: available width minus gutter and separator space
        let text_width = self
            .layout_calculator
            .calculate_text_width(source_content_area.width, gutter_width);

        // Delegate all business logic to SplitDiffManager
        // Using RenderParams struct for cleaner API and easier maintenance
        use crate::managers::split_diff::rendering::RenderParams;
        let params = RenderParams::new(
            self.config,
            self.state,
            self.source_lines,
            self.dest_lines,
            text_width,
            gutter_width,
            max_line_digits,
            available_height,
        );
        let render_data = SplitDiffManager::compute_render_data_static(params);

        // Render the pre-computed lines (all business logic is now in the manager)
        let source_widget = Paragraph::new(render_data.source_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.config.source_title.as_str()),
        );
        f.render_widget(source_widget, source_content_area);

        let dest_widget = Paragraph::new(render_data.dest_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.config.dest_title.as_str()),
        );
        f.render_widget(dest_widget, dest_content_area);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gutter_width_caching() {
        let mut state = SplitDiffViewState::default();
        let constants = DEFAULT_LAYOUT_CONSTANTS;

        // First call should calculate
        let (width1, digits1) = state.get_gutter_width(100, 200, &constants);
        assert_eq!(digits1, 3); // log10(200) + 1 = 3
        assert_eq!(width1, 4); // 3 + 1

        // Second call with same counts should use cache
        let (width2, digits2) = state.get_gutter_width(100, 200, &constants);
        assert_eq!(width1, width2);
        assert_eq!(digits1, digits2);

        // Different counts should recalculate
        let (width3, digits3) = state.get_gutter_width(1000, 2000, &constants);
        assert_eq!(digits3, 4); // log10(2000) + 1 = 4
        assert_eq!(width3, 5); // 4 + 1
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = SplitDiffViewConfig::new()
            .with_source_title("Source File")
            .with_dest_title("Dest File");
        assert_eq!(config.source_title, "Source File");
        assert_eq!(config.dest_title, "Dest File");
    }
}
