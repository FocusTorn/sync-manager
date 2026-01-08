// Core infrastructure module
// Provides foundational systems that other modules depend on

pub mod app;
pub mod app_config;
pub mod project_config;
pub mod events;

pub use app::{App, ViewMode};
pub use app_config::AppConfig;
pub use project_config::ProjectConfig;
pub use events::{AppEvent, EventHandler};
