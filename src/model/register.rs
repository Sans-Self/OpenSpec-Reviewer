//! The register: every requirement the repository asserts, plus the scopes
//! and ignore entries that are judged against it. Built once per run, so
//! `lint` and `change` answer "does this still match anything?" alike.

use super::{Canon, DeltaKind, DeltaSpec};
use crate::citations::normalize_name;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The separator between a capability and a requirement name, in the
/// register, in citations and in a configured scope.
pub const SEPARATOR: char = '§';

/// One requirement of the register, as `<capability> § <requirement name>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub capability: String,
    pub requirement: String,
}

impl Entry {
    pub fn new(capability: &str, requirement: &str) -> Entry {
        Entry {
            capability: capability.to_string(),
            requirement: normalize_name(requirement),
        }
    }

    /// `<capability> § <requirement name>` as a configuration writes it.
    /// Without the separator there is no requirement to name.
    pub fn parse(text: &str) -> Option<Entry> {
        let (capability, requirement) = text.split_once(SEPARATOR)?;
        Some(Entry::new(capability.trim(), requirement))
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {SEPARATOR} {}", self.capability, self.requirement)
    }
}

/// Canon plus every open change, ordered by capability then requirement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Register {
    entries: BTreeSet<Entry>,
}

impl Register {
    /// `open_deltas` is every delta of every open change, the one under
    /// review included. A REMOVED entry asserts nothing, so it adds
    /// nothing; canon still holds the requirement until the change lands.
    pub fn build(canon: &Canon, open_deltas: &[DeltaSpec]) -> Register {
        let canon_entries = canon
            .specs
            .iter()
            .flat_map(|(cap, reqs)| reqs.iter().map(move |r| Entry::new(cap, &r.name)));
        let delta_entries = open_deltas.iter().flat_map(|d| {
            d.entries
                .iter()
                .filter(|e| !matches!(e.kind, DeltaKind::Removed))
                .map(move |e| Entry::new(&d.capability, &e.requirement.name))
        });
        Register {
            entries: canon_entries.chain(delta_entries).collect(),
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, capability: &str, requirement: &str) -> bool {
        self.entries.contains(&Entry::new(capability, requirement))
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.entries.iter().any(|e| e.capability == capability)
    }

    /// Whether anything in the repository falls inside the scope.
    pub fn holds(&self, scope: &Scope) -> bool {
        self.entries
            .iter()
            .any(|e| scope.covers(&e.capability, &e.requirement))
    }

    /// Requirement names per capability, both in register order.
    pub fn by_capability(&self) -> BTreeMap<&str, Vec<&str>> {
        self.entries.iter().fold(BTreeMap::new(), |mut out, e| {
            out.entry(e.capability.as_str())
                .or_insert_with(Vec::new)
                .push(e.requirement.as_str());
            out
        })
    }
}

/// Where an ignore entry applies: a whole capability, or one requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Capability(String),
    Requirement(Entry),
}

impl Scope {
    pub fn parse(text: &str) -> Scope {
        match Entry::parse(text) {
            Some(entry) => Scope::Requirement(entry),
            None => Scope::Capability(text.trim().to_string()),
        }
    }

    pub fn covers(&self, capability: &str, requirement: &str) -> bool {
        match self {
            Scope::Capability(c) => c == capability,
            Scope::Requirement(e) => {
                e.capability == capability && e.requirement == normalize_name(requirement)
            }
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Capability(c) => f.write_str(c),
            Scope::Requirement(e) => write!(f, "{e}"),
        }
    }
}

/// A term the project has read and dismissed. No scopes means the whole
/// repository; the term is an exact string, never a pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermIgnore {
    pub term: String,
    pub scopes: Vec<Scope>,
}

impl TermIgnore {
    pub fn hides(&self, term: &str, capability: &str, requirement: &str) -> bool {
        self.term == term
            && (self.scopes.is_empty()
                || self
                    .scopes
                    .iter()
                    .any(|s| s.covers(capability, requirement)))
    }
}

/// Every dismissed term, as the checks read them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ignores {
    pub terms: Vec<TermIgnore>,
}

impl Ignores {
    pub fn hides(&self, term: &str, capability: &str, requirement: &str) -> bool {
        self.terms
            .iter()
            .any(|i| i.hides(term, capability, requirement))
    }
}
