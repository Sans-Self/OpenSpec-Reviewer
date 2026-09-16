//! Change directories must be changes, and named inside the scopes; and
//! any configured list of names says when one of its entries has gone dead.

use super::config::{Config, CONFIG_PATH};
use crate::glossary::Glossary;
use crate::model::{Canon, DeltaSpec, Entry, Register};
use std::collections::BTreeMap;

/// A directory under `openspec/changes/` other than `archive/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeDir {
    pub name: String,
    /// `proposal.md` or `.openspec.yaml` is present.
    pub has_marker: bool,
}

/// A configured entry that matches nothing in the repository any more.
/// `label` names the kind of entry, `list` the TOML key it lives under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dangling {
    pub label: &'static str,
    pub list: &'static str,
    pub entry: String,
    /// The scope at fault, for an entry that holds several.
    pub scope: Option<String>,
    pub why: &'static str,
}

impl Dangling {
    pub fn message(&self) -> String {
        let subject = match &self.scope {
            Some(s) => format!("{} `{}`: scope `{s}`", self.label, self.entry),
            None => format!("{} `{}`", self.label, self.entry),
        };
        format!(
            "{subject} {}; remove it from `{}` in {CONFIG_PATH}",
            self.why, self.list
        )
    }
}

/// Entries of a configured list that match nothing, judged one at a time.
fn stale<'a>(
    label: &'static str,
    list: &'static str,
    why: &'static str,
    entries: impl IntoIterator<Item = (&'a str, bool)>,
) -> Vec<Dangling> {
    entries
        .into_iter()
        .filter(|(_, live)| !live)
        .map(|(entry, _)| Dangling {
            label,
            list,
            entry: entry.to_string(),
            scope: None,
            why,
        })
        .collect()
}

/// Ignore entries that have outlived their reason. Every input is a
/// property of the repository rather than of the run, so `lint` and a
/// review reach the same verdict for the same entry.
pub fn dangling_ignores(
    config: &Config,
    register: &Register,
    glossary: &Glossary,
    sites: &BTreeMap<String, Vec<(String, String)>>,
) -> Vec<Dangling> {
    const TERMS: &str = "[[definitions.ignore]]";
    const UNCITED: &str = "[[lint.ignore_uncited]]";
    let mut out = Vec::new();
    for entry in &config.definitions.ignore {
        let dangling = |scope: Option<String>, why: &'static str| Dangling {
            label: "ignored term",
            list: TERMS,
            entry: entry.term.clone(),
            scope,
            why,
        };
        if glossary.knows(&entry.term) {
            out.push(dangling(
                None,
                "is a word the glossary already accounts for",
            ));
            continue;
        }
        let nowhere = Vec::new();
        let sites = sites.get(&entry.term).unwrap_or(&nowhere);
        let scopes = entry.scopes();
        if scopes.is_empty() {
            if sites.is_empty() {
                out.push(dangling(None, "silences nothing"));
            }
            continue;
        }
        for scope in &scopes {
            if !register.holds(scope) {
                out.push(dangling(
                    Some(scope.to_string()),
                    "names no capability or requirement of this repository",
                ));
            } else if !sites.iter().any(|(c, r)| scope.covers(c, r)) {
                out.push(dangling(Some(scope.to_string()), "silences nothing"));
            }
        }
    }
    let named: Vec<(&str, bool)> = config
        .lint
        .ignore_uncited
        .iter()
        .map(|e| {
            let live = Entry::parse(&e.requirement)
                .is_some_and(|r| register.contains(&r.capability, &r.requirement));
            (e.requirement.as_str(), live)
        })
        .collect();
    out.extend(stale(
        "ignored requirement",
        UNCITED,
        "is in neither canon nor any open change",
        named,
    ));
    out
}

/// The glossary and span sites a dangling verdict reads, derived the same
/// way wherever the check runs; the register is built once per run and
/// handed in.
pub fn ignore_warnings(
    config: &Config,
    register: &Register,
    canon: &Canon,
    open_deltas: &[DeltaSpec],
) -> Vec<Dangling> {
    let glossary = Glossary::build(canon, open_deltas, &config.definitions.capability);
    let sites = crate::glossary::span_sites(&glossary, canon, open_deltas);
    dangling_ignores(config, register, &glossary, &sites)
}

/// `(change name, message)` pairs; the name is empty for the stale
/// grandfather entries, which point at the configuration instead.
pub fn check_changes(
    dirs: &[ChangeDir],
    scopes: Option<&[String]>,
    grandfathered: &[String],
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for dir in dirs {
        if !dir.has_marker {
            out.push((
                dir.name.clone(),
                format!(
                    "openspec/changes/{} is not a change directory: no proposal.md or .openspec.yaml; the OpenSpec CLI cannot address nested changes, group by name instead",
                    dir.name
                ),
            ));
            continue;
        }
        let Some(scopes) = scopes else { continue };
        if grandfathered.contains(&dir.name) {
            continue;
        }
        let in_scope = scopes.iter().any(|s| {
            dir.name
                .strip_prefix(s.as_str())
                .and_then(|rest| rest.strip_prefix('-'))
                .is_some_and(|rest| !rest.is_empty())
        });
        if !in_scope {
            out.push((
                dir.name.clone(),
                format!(
                    "openspec/changes/{}: change name does not start with a scope ({}) and a hyphen",
                    dir.name,
                    scopes.join(", ")
                ),
            ));
        }
    }
    let entries: Vec<(&str, bool)> = grandfathered
        .iter()
        .map(|name| (name.as_str(), dirs.iter().any(|d| &d.name == name)))
        .collect();
    out.extend(
        stale("grandfathered change", "grandfathered", "is gone", entries)
            .into_iter()
            .map(|d| (String::new(), d.message())),
    );
    out
}
