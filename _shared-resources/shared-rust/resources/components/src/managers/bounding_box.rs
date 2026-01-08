// Bounding Box Manager
// Provides OOP-style bounding box wrapper with handle support

use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};
use crate::core::{RectHandle, RectRegistry, RectMetrics};
use crate::utilities::DimmingContext;

// ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
// │                            Bounding Box Manager - OOP Style Bounding Box Operations            │
// └────────────────────────────────────────────────────────────────────────────────────────────────┘

/// BoundingBox manager for creating and managing bounding boxes with handles (HWND-like)
/// Provides easy property access and relative positioning support
///
/// # Usage Examples
///
/// ```rust
/// // Create a bounding box with a handle name
/// const HWND_MAIN_CONTENT_BOX: &str = "hwndMainContentBox";
/// let main_box = BoundingBox::create(&mut registry, HWND_MAIN_CONTENT_BOX, rect);
///
/// // Prepare and render
/// if let Some(rect) = main_box.prepare(&mut registry) {
///     // Use rect for rendering
/// }
/// main_box.render(&mut f, &mut registry, &dimming);
///
/// // Access properties
/// if let Some(x) = main_box.x(&registry) {
///     println!("X position: {}", x);
/// }
/// main_box.set_x(&mut registry, 10);
///
/// // Relative positioning
/// let box2 = BoundingBox::create(&mut registry, "box2", rect2);
/// box2.set_relative_x(&mut registry, &main_box, 1);  // box2.x = main_box.x + 1
/// box2.set_relative_y(&mut registry, &main_box, 1);  // box2.y = main_box.y + 1
///
/// // List all boxes to see what elements exist
/// let all_boxes = list_all_boxes(&registry);
/// for (name, metrics) in all_boxes {
///     println!("Box {}: x={}, y={}, w={}, h={}", name, metrics.x, metrics.y, metrics.width, metrics.height);
/// }
/// ```
pub struct BoundingBox {
    handle: RectHandle,
    handle_name: String,
}

impl BoundingBox {
    /// Create and register a new bounding box
    pub fn create(registry: &mut RectRegistry, handle_name: &str, rect: Rect) -> Self {
        let handle = registry.register(Some(handle_name), rect);
        Self {
            handle,
            handle_name: handle_name.to_string(),
        }
    }
    
    /// Get the handle (object identifier)
    pub fn handle(&self) -> RectHandle {
        self.handle
    }
    
    /// Get the handle name (HWND string)
    pub fn name(&self) -> &str {
        &self.handle_name
    }
    
    /// Prepare bounding box for rendering (returns current Rect)
    pub fn prepare(&self, registry: &mut RectRegistry) -> Option<Rect> {
        registry.get_metrics(self.handle).map(|m| m.into())
    }
    
    /// Render the bounding box as a block widget
    pub fn render(&self, f: &mut Frame, registry: &mut RectRegistry, dimming: &DimmingContext) -> bool {
        if let Some(rect) = self.prepare(registry) {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(dimming.border_color(true)));
            f.render_widget(block, rect);
            true
        } else {
            false
        }
    }
    
    /// Get X position
    pub fn x(&self, registry: &RectRegistry) -> Option<u16> {
        registry.get_metrics(self.handle).map(|m| m.x)
    }
    
    /// Set X position
    pub fn set_x(&self, registry: &mut RectRegistry, x: u16) -> bool {
        registry.set_x(self.handle, x)
    }
    
    /// Get Y position
    pub fn y(&self, registry: &RectRegistry) -> Option<u16> {
        registry.get_metrics(self.handle).map(|m| m.y)
    }
    
    /// Set Y position
    pub fn set_y(&self, registry: &mut RectRegistry, y: u16) -> bool {
        registry.set_y(self.handle, y)
    }
    
    /// Get width
    pub fn width(&self, registry: &RectRegistry) -> Option<u16> {
        registry.get_metrics(self.handle).map(|m| m.width)
    }
    
    /// Set width
    pub fn set_width(&self, registry: &mut RectRegistry, width: u16) -> bool {
        if let Some(mut metrics) = registry.get_metrics(self.handle) {
            metrics.width = width;
            registry.update(self.handle, metrics.into());
            true
        } else {
            false
        }
    }
    
    /// Get height
    pub fn height(&self, registry: &RectRegistry) -> Option<u16> {
        registry.get_metrics(self.handle).map(|m| m.height)
    }
    
    /// Set height
    pub fn set_height(&self, registry: &mut RectRegistry, height: u16) -> bool {
        if let Some(mut metrics) = registry.get_metrics(self.handle) {
            metrics.height = height;
            registry.update(self.handle, metrics.into());
            true
        } else {
            false
        }
    }
    
    /// Get current metrics
    pub fn metrics(&self, registry: &RectRegistry) -> Option<RectMetrics> {
        registry.get_metrics(self.handle)
    }
    
    /// Update entire rectangle
    pub fn update(&self, registry: &mut RectRegistry, rect: Rect) -> bool {
        registry.update(self.handle, rect)
    }

    /// Create a BoundingBox from an existing handle name in the registry
    /// Returns None if the handle name doesn't exist
    pub fn from_handle_name(registry: &RectRegistry, handle_name: &str) -> Option<Self> {
        registry.get_handle(handle_name).map(|handle| Self {
            handle,
            handle_name: handle_name.to_string(),
        })
    }
    
    /// Set position relative to another bounding box
    /// Example: box2.set_relative_x(registry, &HWND_MAIN_CONTENT_BOX, 1)
    pub fn set_relative_x(&self, registry: &mut RectRegistry, other: &BoundingBox, offset: i16) -> bool {
        if let Some(other_x) = other.x(registry) {
            let new_x = if offset >= 0 {
                other_x.saturating_add(offset as u16)
            } else {
                other_x.saturating_sub((-offset) as u16)
            };
            self.set_x(registry, new_x)
        } else {
            false
        }
    }
    
    /// Set Y position relative to another bounding box
    pub fn set_relative_y(&self, registry: &mut RectRegistry, other: &BoundingBox, offset: i16) -> bool {
        if let Some(other_y) = other.y(registry) {
            let new_y = if offset >= 0 {
                other_y.saturating_add(offset as u16)
            } else {
                other_y.saturating_sub((-offset) as u16)
            };
            self.set_y(registry, new_y)
        } else {
            false
        }
    }
    
    /// Set position relative to another bounding box (both x and y)
    pub fn set_relative_position(&self, registry: &mut RectRegistry, other: &BoundingBox, offset_x: i16, offset_y: i16) -> bool {
        if let Some(other_metrics) = other.metrics(registry) {
            let new_x = if offset_x >= 0 {
                other_metrics.x.saturating_add(offset_x as u16)
            } else {
                other_metrics.x.saturating_sub((-offset_x) as u16)
            };
            let new_y = if offset_y >= 0 {
                other_metrics.y.saturating_add(offset_y as u16)
            } else {
                other_metrics.y.saturating_sub((-offset_y) as u16)
            };
            if let Some(mut metrics) = registry.get_metrics(self.handle) {
                metrics.x = new_x;
                metrics.y = new_y;
                registry.update(self.handle, metrics.into());
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Helper function to get a BoundingBox from a handle name
/// Useful when you only have the handle name (e.g., inside closures)
pub fn get_box_by_name(registry: &RectRegistry, handle_name: &str) -> Option<BoundingBox> {
    registry.get_handle(handle_name).map(|handle| {
        BoundingBox {
            handle,
            handle_name: handle_name.to_string(),
        }
    })
}

/// Helper function to list all registered bounding boxes
/// Makes it easy to see what elements exist
pub fn list_all_boxes(registry: &RectRegistry) -> Vec<(String, RectMetrics)> {
    registry.all_names()
        .iter()
        .filter_map(|name| {
            registry.get_metrics_by_name(name)
                .map(|metrics| ((*name).clone(), metrics))
        })
        .collect()
}

