// Core infrastructure module
// Provides foundational systems that other modules depend on

pub mod rect_handle;

pub use rect_handle::{
    RectHandle, RectRegistry, RectMetrics,
    TabBarState, TabConfigData, TabBarConfigData,
    AlignmentConfigData, TabState, TabBarStateColors,
};

