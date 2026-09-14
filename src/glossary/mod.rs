//! The project's glossary: one capability whose requirements are terms.
//! Pure: built from canon plus the change's own delta, checked as values.

pub mod checks;
pub mod synonyms;

pub use checks::{
    deprecated_in_canon, deprecated_in_pairings, new_undefined, recurring_undefined, term_in_use,
    unused_terms, CanonHit, Recurring,
};
pub use synonyms::{parse_deprecated, Matcher};

use crate::model::{Canon, DeltaKind, DeltaSpec, Requirement, Scenario};
use serde::Serialize;

pub const DEFAULT_CAPABILITY: &str = "definitions";
pub const DEFAULT_MIN_RECURRENCE: usize = 3;

/// A glossary entry. The name is the term, the body its meaning, the
/// scenarios usage examples; `- **Deprecated:**` lists the words not to use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Term {
    #[serde(rename = "term")]
    pub name: String,
    pub meaning: String,
    pub deprecated: Vec<String>,
    #[serde(skip)]
    pub examples: Vec<Scenario>,
}

impl Term {
    pub fn from_requirement(req: &Requirement) -> Term {
        let (meaning, deprecated) = parse_deprecated(&req.body);
        Term {
            name: req.name.clone(),
            meaning,
            deprecated,
            examples: req.scenarios.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Glossary {
    #[serde(skip)]
    pub capability: String,
    pub terms: Vec<Term>,
}

impl Glossary {
    /// Canon's glossary capability with the change's own delta applied, so
    /// a term added in the same change counts as defined.
    pub fn build(canon: &Canon, deltas: &[DeltaSpec], capability: &str) -> Glossary {
        let mut terms: Vec<Term> = canon
            .specs
            .get(capability)
            .map(|reqs| reqs.iter().map(Term::from_requirement).collect())
            .unwrap_or_default();
        for delta in deltas.iter().filter(|d| d.capability == capability) {
            for entry in &delta.entries {
                let name = &entry.requirement.name;
                match &entry.kind {
                    DeltaKind::Removed => terms.retain(|t| &t.name != name),
                    DeltaKind::Renamed { from } => {
                        terms.retain(|t| &t.name != from && &t.name != name);
                        if !entry.requirement.is_bare() {
                            terms.push(Term::from_requirement(&entry.requirement));
                        } else if let Some(old) = canon.get(capability, from) {
                            terms.push(Term::from_requirement(&old.renamed(name)));
                        }
                    }
                    DeltaKind::Added | DeltaKind::Modified => {
                        terms.retain(|t| &t.name != name);
                        terms.push(Term::from_requirement(&entry.requirement));
                    }
                }
            }
        }
        Glossary {
            capability: capability.to_string(),
            terms,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&Term> {
        self.terms
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Terms and deprecated synonyms alike: every word the glossary knows.
    pub fn knows(&self, word: &str) -> bool {
        self.terms.iter().any(|t| {
            t.name.eq_ignore_ascii_case(word)
                || t.deprecated.iter().any(|d| d.eq_ignore_ascii_case(word))
        })
    }

    /// Terms present in `text`, ordered by first appearance.
    pub fn terms_in<'a>(&'a self, text: &str) -> Vec<&'a Term> {
        let mut found: Vec<(usize, &Term)> = self
            .terms
            .iter()
            .filter_map(|t| Matcher::new(&t.name).first(text).map(|at| (at, t)))
            .collect();
        found.sort_by_key(|(at, _)| *at);
        found.into_iter().map(|(_, t)| t).collect()
    }
}
