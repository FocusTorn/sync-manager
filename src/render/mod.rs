// Render module - UI rendering functions

pub mod dashboard;
pub mod content;
pub mod sync_content;

// pub use dashboard::render_dashboard;  // Reserved for future dashboard functionality
pub use content::render_content;
pub use sync_content::render_sync_manager_content;
