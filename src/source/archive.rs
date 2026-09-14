//! Other changes in the working tree: open ones for collision checks,
//! archived ones for history.

use crate::model::{parse_delta_spec, DeltaSpec, ParseError};
use std::path::{Path, PathBuf};

/// One archived change. Parsing may fail; history treats that as a note.
#[derive(Debug, Clone)]
pub struct Archive {
    pub name: String,
    pub deltas: Result<Vec<DeltaSpec>, ParseError>,
}

fn dirs_sorted(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn read_deltas(change_dir: &Path, display_prefix: &str) -> Result<Vec<DeltaSpec>, ParseError> {
    dirs_sorted(&change_dir.join("specs"))
        .into_iter()
        .filter_map(|cap_dir| {
            let file = cap_dir.join("spec.md");
            let capability = cap_dir.file_name()?.to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&file).ok()?;
            Some(
                parse_delta_spec(
                    &capability,
                    &format!("{display_prefix}/specs/{capability}/spec.md"),
                    &text,
                )
                .map(DeltaSpec::join_renames),
            )
        })
        .collect()
}

/// Archived changes sorted by directory name, which starts with the date.
pub fn load_archives(root: &Path) -> Vec<Archive> {
    let archive = root.join("openspec").join("changes").join("archive");
    dirs_sorted(&archive)
        .into_iter()
        .filter_map(|dir| {
            let name = dir.file_name()?.to_string_lossy().into_owned();
            let deltas = read_deltas(&dir, &format!("openspec/changes/archive/{name}"));
            Some(Archive { name, deltas })
        })
        .collect()
}

/// Open changes other than `exclude`, with their deltas. A change that does
/// not parse is skipped: it will fail its own review.
pub fn load_open_changes(root: &Path, exclude: &[String]) -> Vec<(String, Vec<DeltaSpec>)> {
    let changes = root.join("openspec").join("changes");
    dirs_sorted(&changes)
        .into_iter()
        .filter_map(|dir| {
            let name = dir.file_name()?.to_string_lossy().into_owned();
            if name == "archive" || exclude.contains(&name) {
                return None;
            }
            let deltas = read_deltas(&dir, &format!("openspec/changes/{name}")).ok()?;
            Some((name, deltas))
        })
        .collect()
}
