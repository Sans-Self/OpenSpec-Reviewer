//! Review OpenSpec changes as the semantic diff they are.
//!
//! The domain modules (`model`, `review`, `state`) take values in and return
//! values out. Everything that touches the filesystem, git, gh or the
//! terminal lives under `source` and `render`.

pub mod build;
pub mod model;
pub mod render;
pub mod review;
pub mod source;
pub mod state;
