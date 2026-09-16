//! Review OpenSpec changes as the semantic diff they are.
//!
//! The domain modules (`model`, `review`, `state`, `citations`, `drift`,
//! `glossary`, `skills`, `assist`) take values in and return values out.
//! Everything that touches the filesystem, git, gh or the terminal lives
//! under `source` and `render`.

pub mod assist;
pub mod build;
pub mod citations;
pub mod drift;
pub mod glossary;
pub mod model;
pub mod render;
pub mod review;
pub mod skills;
pub mod source;
pub mod state;
