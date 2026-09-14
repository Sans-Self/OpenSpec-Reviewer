//! Citations: `spec:<capability> § <requirement>` in specs and source,
//! evidence cited from specs, and what a change does to its citers. Pure:
//! the walk that feeds it lives in `source::lint`.

pub mod config;
pub mod coverage;
pub mod evidence;
pub mod grammar;
pub mod lint;
pub mod radius;
pub mod scan;
pub mod structure;

pub use config::{read_config, require_config, Config, ConfigError};
pub use grammar::Grammar;
pub use lint::{lint, LintFinding, LintReport};
pub use scan::{scan, Scanned, SourceFile, SpecFile};

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;

/// Runs of whitespace compare as one space, in headings and citations alike.
pub fn normalize_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Citation {
    pub capability: String,
    pub requirement: String,
}

impl Citation {
    pub fn new(capability: &str, requirement: &str) -> Citation {
        Citation {
            capability: capability.to_string(),
            requirement: normalize_name(requirement),
        }
    }
}

impl fmt::Display for Citation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} § {}", self.capability, self.requirement)
    }
}

/// Who cites. A spec or delta records its capability; source records none,
/// which is what blast radius and coverage key on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Citer {
    pub file: PathBuf,
    pub citing_capability: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CitationIndex {
    pub citers: BTreeMap<Citation, Vec<Citer>>,
    /// ADDED by open changes: resolvable before the change syncs to canon.
    pub in_flight: BTreeSet<Citation>,
    pub canon: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    Resolved,
    UnknownCapability,
    UnknownRequirement,
}

impl CitationIndex {
    pub fn resolve(&self, c: &Citation) -> Resolution {
        if self.in_flight.contains(c) {
            return Resolution::Resolved;
        }
        match self.canon.get(&c.capability) {
            Some(names) if names.contains(&c.requirement) => Resolution::Resolved,
            Some(_) => Resolution::UnknownRequirement,
            None if self.in_flight.iter().any(|f| f.capability == c.capability) => {
                Resolution::UnknownRequirement
            }
            None => Resolution::UnknownCapability,
        }
    }

    pub fn citers_of(&self, c: &Citation) -> &[Citer] {
        self.citers.get(c).map_or(&[], Vec::as_slice)
    }

    /// Citers outside the requirement's own capability, one per file: a
    /// test that cites the same requirement three times is one citer.
    pub fn outside_citers(&self, c: &Citation) -> Vec<&Citer> {
        let mut seen = BTreeSet::new();
        self.citers_of(c)
            .iter()
            .filter(|citer| citer.citing_capability.as_deref() != Some(c.capability.as_str()))
            .filter(|citer| seen.insert(citer.file.clone()))
            .collect()
    }
}
