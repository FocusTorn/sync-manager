// Operations module
// Business logic for sync operations, diff computation, and git integration

pub mod diff;
pub mod sync;
pub mod git;

pub use diff::{DiffEngine, DiffEntry, DiffType, FileStatus};
pub use sync::SyncEngine;
pub use git::GitOps;
