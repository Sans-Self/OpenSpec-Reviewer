//! Wire a snapshot, canon and the working tree into a `Review`.

use crate::citations::radius::blast_radius;
use crate::citations::scan::{reason, scan, OpenChange, SpecFile};
use crate::citations::{read_config, Config, ConfigError, Grammar, Resolution};
use crate::drift::drift_findings;
use crate::glossary::{self, Glossary};
use crate::model::{load_change, Change, ChangeError};
use crate::review::pair::requirement_text;
use crate::review::{
    collect_history, pair_change, CanonEdit, ChangeReview, Finding, FindingKind, Pairing, Review,
};
use crate::source::{
    load_archives, load_canon, load_open_changes, repo_key, CanonError, Snapshot, Workspace,
    WorkspaceError,
};
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
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
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

    let config = read_config(root)?;
    let definitions = config
        .as_ref()
        .map(|c| c.definitions.clone())
        .unwrap_or_default();
    let glossary = Glossary::build(
        &canon,
        &changes
            .iter()
            .flat_map(|c| c.deltas.iter().cloned())
            .collect::<Vec<_>>(),
        &definitions.capability,
    );
    let citations = match &config {
        Some(c) => Some(Workspace::load(root, &c.lint)?),
        None => None,
    };

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
            if let (Some(config), Some(workspace)) = (&config, &citations) {
                join_citations(&mut review, change, config, workspace, snapshot);
            }
            let max_common = config.as_ref().map_or(5, |c| c.term_drift.max_common);
            for p in review.pairings_mut() {
                let drift = drift_findings(p, &canon, &change.deltas, max_common);
                p.findings.extend(drift);
            }
            join_glossary(&mut review, change, &canon, &others, &definitions);
            review
        })
        .collect();

    let canon_edits = snapshot.canon_files().map(CanonEdit::from_file).collect();
    Ok(Review::new(snapshot.origin.clone(), reviews, canon_edits).with_glossary(glossary))
}

/// The lint's view of the change under review: its deltas as the snapshot
/// has them, replacing whatever the working tree holds under that name.
fn open_change(change: &Change, snapshot: &Snapshot) -> OpenChange {
    let delta_files = snapshot
        .files
        .iter()
        .filter_map(|f| {
            let rel = f.path.strip_prefix("openspec/changes").ok()?;
            let mut parts = rel.components();
            let name = parts.next()?.as_os_str().to_string_lossy();
            if name != change.name.as_str() || parts.next()?.as_os_str() != "specs" {
                return None;
            }
            let capability = parts.next()?.as_os_str().to_string_lossy().into_owned();
            Some(SpecFile {
                capability,
                path: f.path.clone(),
                text: f.after.clone()?,
            })
        })
        .collect();
    OpenChange {
        name: change.name.clone(),
        deltas: change.deltas.clone(),
        delta_files,
    }
}

/// Dangling citations in a pairing's text, blast radius on its pairing.
fn join_citations(
    review: &mut ChangeReview,
    change: &Change,
    config: &Config,
    workspace: &Workspace,
    snapshot: &Snapshot,
) {
    let this = open_change(change, snapshot);
    let mut changes: Vec<OpenChange> = workspace
        .changes
        .iter()
        .filter(|c| c.name != change.name)
        .cloned()
        .collect();
    changes.push(this.clone());
    let grammar = Grammar::new(config.lint.cite_helper.as_deref());
    let scanned = scan(
        &workspace.canon,
        &workspace.specs,
        &changes,
        &workspace.sources,
        &grammar,
    );
    let mut radius = blast_radius(&scanned.index, &this);
    for p in review.pairings_mut() {
        let location = p.location();
        // The delta's own text, not the paired canon: a REMOVED entry has
        // no after side but its reason line can still cite.
        let entry_text = change
            .deltas
            .iter()
            .filter(|d| d.capability == p.capability)
            .flat_map(|d| d.entries.iter())
            .find(|e| e.requirement.name == p.name)
            .map(|e| requirement_text(&e.requirement))
            .unwrap_or_default();
        let dangling: Vec<Finding> = Grammar::literal(&entry_text)
            .into_iter()
            .filter_map(|c| match scanned.index.resolve(&c) {
                Resolution::Resolved => None,
                r => Some(Finding::new(
                    FindingKind::CitationDangling {
                        citation: c.to_string(),
                        reason: reason(&r, &c),
                    },
                    location.clone(),
                )),
            })
            .collect();
        p.findings.extend(dangling);
        let (mine, rest): (Vec<Finding>, Vec<Finding>) =
            radius.into_iter().partition(|f| f.location == location);
        radius = rest;
        p.findings.extend(mine);
    }
}

/// The glossary checks that belong to the review of one change. Canon
/// hits and unused terms land on the term's pairing when the change touches
/// the term; the lint reports the rest.
fn join_glossary(
    review: &mut ChangeReview,
    change: &Change,
    canon: &crate::model::Canon,
    others: &[(String, Vec<crate::model::DeltaSpec>)],
    definitions: &crate::citations::config::Definitions,
) {
    let glossary = Glossary::build(canon, &change.deltas, &definitions.capability);
    if glossary.is_empty() {
        return;
    }
    let pairings: Vec<Pairing> = review.pairings().cloned().collect();
    let refs: Vec<&Pairing> = pairings.iter().collect();
    let mut extra: Vec<Finding> = glossary::deprecated_in_pairings(&glossary, &refs);
    extra.extend(glossary::new_undefined(&glossary, &refs));
    for p in &pairings {
        extra.extend(glossary::term_in_use(&glossary, canon, p));
    }
    let open_deltas: Vec<crate::model::DeltaSpec> = others
        .iter()
        .flat_map(|(_, d)| d.iter().cloned())
        .chain(change.deltas.iter().cloned())
        .collect();
    let unused: Vec<String> = glossary::unused_terms(&glossary, canon, &open_deltas)
        .into_iter()
        .map(|t| t.name.clone())
        .collect();
    let canon_hits = glossary::deprecated_in_canon(&glossary, canon);
    let unbound: Vec<(String, Option<String>)> = glossary::unbound_terms(&glossary)
        .into_iter()
        .map(|t| {
            (
                t.name.clone(),
                t.binding.names_another().map(str::to_string),
            )
        })
        .collect();
    for p in pairings
        .iter()
        .filter(|p| p.capability == glossary.capability)
    {
        if unused.iter().any(|u| u.eq_ignore_ascii_case(&p.name)) {
            extra.push(Finding::new(FindingKind::DefinedButUnused, p.location()));
        }
        for (_, names) in unbound
            .iter()
            .filter(|(t, _)| t.eq_ignore_ascii_case(&p.name))
        {
            extra.push(Finding::new(
                FindingKind::TermWithoutBindingLine {
                    names: names.clone(),
                },
                p.location(),
            ));
        }
        for hit in canon_hits
            .iter()
            .filter(|h| h.term.eq_ignore_ascii_case(&p.name))
        {
            let mut f = Finding::new(
                FindingKind::UsesDeprecatedSynonym {
                    synonym: hit.synonym.clone(),
                    term: hit.term.clone(),
                },
                p.location(),
            );
            f.details = vec![format!(
                "{} § {}{}",
                hit.capability,
                hit.requirement,
                hit.scenario
                    .as_ref()
                    .map(|s| format!(" # {s}"))
                    .unwrap_or_default()
            )];
            extra.push(f);
        }
    }
    for p in review.pairings_mut() {
        let location = p.location();
        let (mine, rest): (Vec<Finding>, Vec<Finding>) = extra.into_iter().partition(|f| {
            f.location.change == location.change
                && f.location.capability == location.capability
                && f.location.requirement == location.requirement
        });
        extra = rest;
        p.findings.extend(mine);
    }
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
