//! A change as read from a snapshot's files.

use super::{parse_delta_spec, Artefact, Change, DeltaSpec, ParseError, ARTEFACT_NAMES};
use crate::source::{FileChange, Snapshot};
use std::collections::BTreeMap;
use std::path::{Component, Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChangeError {
    #[error("change `{0}` has no artefacts and no delta specs")]
    Empty(String),
    #[error("the snapshot holds no change under openspec/changes/")]
    NoChange,
    #[error(transparent)]
    Parse(#[from] ParseError),
}

const CHANGES_DIR: &str = "openspec/changes";

fn components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

/// `openspec/changes/<name>/<rest...>` → `(name, rest)`, skipping archive.
fn change_path(path: &Path) -> Option<(String, Vec<String>)> {
    let parts = components(path);
    if parts.len() < 3 || parts[0] != "openspec" || parts[1] != "changes" || parts[2] == "archive" {
        return None;
    }
    Some((parts[2].clone(), parts[3..].to_vec()))
}

/// Every change a snapshot holds, one `Change` per directory under
/// `openspec/changes/`, in name order.
pub fn load_change(snapshot: &Snapshot) -> Result<Vec<Change>, ChangeError> {
    let mut by_name: BTreeMap<String, Vec<(&FileChange, Vec<String>)>> = BTreeMap::new();
    for file in &snapshot.files {
        if let Some((name, rest)) = change_path(&file.path) {
            by_name.entry(name).or_default().push((file, rest));
        }
    }
    if by_name.is_empty() {
        return Err(ChangeError::NoChange);
    }

    by_name
        .into_iter()
        .map(|(name, files)| build_change(name, files))
        .collect()
}

fn build_change(
    name: String,
    files: Vec<(&FileChange, Vec<String>)>,
) -> Result<Change, ChangeError> {
    let artefacts: Vec<Artefact> = ARTEFACT_NAMES
        .iter()
        .filter_map(|artefact| {
            files
                .iter()
                .find(|(_, rest)| rest.len() == 1 && rest[0] == *artefact)
                .map(|(file, _)| Artefact {
                    name: artefact.to_string(),
                    before: file.before.clone(),
                    after: file.after.clone(),
                })
        })
        .collect();

    let deltas: Vec<DeltaSpec> = files
        .iter()
        .filter(|(file, rest)| {
            rest.len() == 3 && rest[0] == "specs" && rest[2] == "spec.md" && file.after.is_some()
        })
        .map(|(file, rest)| {
            let text = file.after.as_deref().unwrap_or_default();
            parse_delta_spec(
                &rest[1],
                &format!("{CHANGES_DIR}/{name}/specs/{}/spec.md", rest[1]),
                text,
            )
            .map(DeltaSpec::join_renames)
        })
        .collect::<Result<_, _>>()?;

    if artefacts.is_empty() && deltas.is_empty() {
        return Err(ChangeError::Empty(name));
    }

    Ok(Change {
        name,
        artefacts,
        deltas,
    })
}
