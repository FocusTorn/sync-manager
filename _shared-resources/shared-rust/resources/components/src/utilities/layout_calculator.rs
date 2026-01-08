// Layout calculator for split diff view area computations
// Provides area calculation methods for split-panel layouts

use ratatui::layout::Rect;
use crate::elements::LayoutConstants;

/// Layout calculator for split diff view area computations
pub struct LayoutCalculator {
    constants: LayoutConstants,
}

impl LayoutCalculator {
    /// Create a new layout calculator with the given constants
    pub const fn new(constants: LayoutConstants) -> Self {
        Self { constants }
    }

    /// Calculate the source content area within the parent area
    /// Source panel: positioned at (x+offset, y+offset) with half width minus borders
    /// Note: Not const because it performs runtime arithmetic (saturating operations)
    pub fn calculate_source_area(&self, area: Rect) -> Rect {
        Rect {
            x: area.x.saturating_add(self.constants.border_offset()),
            y: area.y.saturating_add(self.constants.border_offset()),
            width: (area.width.saturating_sub(self.constants.border_width())) / self.constants.split_ratio(),
            height: area.height.saturating_sub(self.constants.border_width()),
        }
    }

    /// Calculate the destination content area adjacent to source area
    /// Dest panel: positioned to the right of source with same height
    /// Note: Not const because it performs runtime arithmetic (saturating operations)
    pub fn calculate_dest_area(&self, source_area: Rect, parent_width: u16) -> Rect {
        Rect {
            x: source_area.x.saturating_add(source_area.width).saturating_add(self.constants.border_offset()),
            y: source_area.y,
            width: (parent_width.saturating_sub(self.constants.border_width())) / self.constants.split_ratio(),
            height: source_area.height,
        }
    }

    /// Calculate available height accounting for block borders
    pub fn calculate_available_height(&self, area: Rect) -> usize {
        area.height.saturating_sub(self.constants.border_width()) as usize
    }

    /// Calculate text width from content area width, gutter width, and border offset
    pub fn calculate_text_width(&self, content_width: u16, gutter_width: usize) -> usize {
        let gutter_width_u16 = gutter_width.min(u16::MAX as usize) as u16;
        content_width
            .saturating_sub(gutter_width_u16)
            .saturating_sub(self.constants.border_offset()) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::DEFAULT_LAYOUT_CONSTANTS;

    #[test]
    fn test_layout_calculator_source_area() {
        let calc = LayoutCalculator::new(DEFAULT_LAYOUT_CONSTANTS);
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let source = calc.calculate_source_area(area);
        assert_eq!(source.x, 1); // border_offset
        assert_eq!(source.y, 1); // border_offset
        assert_eq!(source.width, 49); // (100 - 2) / 2
        assert_eq!(source.height, 48); // 50 - 2
    }

    #[test]
    fn test_layout_calculator_dest_area() {
        let calc = LayoutCalculator::new(DEFAULT_LAYOUT_CONSTANTS);
        let source_area = Rect {
            x: 1,
            y: 1,
            width: 49,
            height: 48,
        };
        let dest = calc.calculate_dest_area(source_area, 100);
        assert_eq!(dest.x, 51); // 1 + 49 + 1
        assert_eq!(dest.y, 1); // same as source
        assert_eq!(dest.width, 49); // (100 - 2) / 2
        assert_eq!(dest.height, 48); // same as source
    }
}
