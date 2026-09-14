//! Wire a snapshot, canon and the working tree into a `Review`.

use crate::model::{load_change, ChangeError};
use crate::review::{collect_history, pair_change, CanonEdit, ChangeReview, Review};
use crate::source::{load_archives, load_canon, load_open_changes, repo_key, CanonError, Snapshot};
use crate::state::{state_path, Store};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error(transparent)]
    Canon(#[from] CanonError),
    #[error(transparent)]
    Change(#[from] ChangeError),
    #[error(transparent)]
    State(#[from] crate::state::StateError),
}

pub struct Built {
    pub review: Review,
    pub stores: BTreeMap<String, Store>,
}

/// Pure part: pair every change in the snapshot against canon and the
/// other open changes, then attach history from the archives.
pub fn build_review(root: &Path, snapshot: &Snapshot) -> Result<Review, BuildError> {
    let canon = load_canon(root)?;
    let changes = load_change(snapshot)?;
    let names: Vec<String> = changes.iter().map(|c| c.name.clone()).collect();
    let others = load_open_changes(root, &names);
    let archives = load_archives(root);

    let reviews: Vec<ChangeReview> = changes
        .iter()
        .map(|change| {
            let mut review = pair_change(change, &canon, &others);
            for p in review.pairings_mut() {
                let (history, findings) =
                    collect_history(&archives, &p.capability, &p.name, &p.location());
                p.history = history;
                p.findings.extend(findings);
            }
            review
        })
        .collect();

    let canon_edits = snapshot.canon_files().map(CanonEdit::from_file).collect();
    Ok(Review::new(snapshot.origin.clone(), reviews, canon_edits))
}

/// Attach persisted approvals and notes, one store per change. With no
/// `state_home` the stores are disabled: `--no-state`.
pub fn attach_state(
    root: &Path,
    mut review: Review,
    state_home: Option<&Path>,
) -> Result<Built, BuildError> {
    let key = repo_key(root);
    let mut stores = BTreeMap::new();
    for change in &mut review.changes {
        let store = match state_home {
            Some(home) => Store::open(state_path(home, &key, &change.name))?,
            None => Store::disabled(),
        };
        for a in &mut change.artefacts {
            a.state = store.get(&a.key());
        }
        for p in change.pairings_mut() {
            p.state = store.get(&p.key());
        }
        stores.insert(change.name.clone(), store);
    }
    Ok(Built { review, stores })
}
