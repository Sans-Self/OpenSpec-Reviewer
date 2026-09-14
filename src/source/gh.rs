//! `gh <pr>`: `gh pr diff` handed to the diff source.

use super::{parse_diff, Snapshot, Source, SourceError};
use std::path::PathBuf;
use std::process::Command;

pub struct GhSource {
    pub root: PathBuf,
    pub pr: String,
}

impl Source for GhSource {
    fn fetch(&self) -> Result<Snapshot, SourceError> {
        let out = Command::new("gh")
            .args(["pr", "diff", &self.pr])
            .current_dir(&self.root)
            .output()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => SourceError::GhMissing,
                _ => SourceError::Io {
                    context: "running gh".to_string(),
                    source: e,
                },
            })?;
        if !out.status.success() {
            return Err(SourceError::Gh {
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }
        let text = String::from_utf8_lossy(&out.stdout);
        parse_diff(&self.root, &text, format!("gh pr {}", self.pr))
    }
}
