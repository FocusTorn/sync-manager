// Managers module
// Provides OOP-style manager wrappers for TUI components

pub mod tab_bar;
pub mod bounding_box;
// pub mod split_diff;  // TODO: Create split_diff module

pub use tab_bar::TabBarManager;
pub use bounding_box::{BoundingBox, get_box_by_name, list_all_boxes};
// pub use split_diff::SplitDiffManager;  // TODO: Uncomment when split_diff module is created
// Re-export split diff types
// pub use split_diff::{LineAlignment, SplitDiffRenderData};  // TODO: Uncomment when split_diff module is created

// Re-export YAML configuration types from tab_bar module
pub use tab_bar::{
    TabBarConfigYaml,
    TabConfigYaml,
    AlignmentConfigYaml,
    TabBarColorsYaml,
    convert_tab_bar_config,
    create_tab_configs,
    create_tab_bar_from_config,
};

