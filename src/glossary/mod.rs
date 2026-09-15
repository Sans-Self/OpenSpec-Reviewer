//! The project's glossary: one capability whose requirements are terms.
//! Pure: built from canon plus the change's own delta, checked as values.

pub mod checks;
pub mod synonyms;

pub use checks::{
    deprecated_in_canon, deprecated_in_pairings, new_undefined, recurring_undefined, term_in_use,
    unused_terms, CanonHit, Recurring,
};
pub use synonyms::{parse_markers, Markers, Matcher};

use crate::model::{Canon, DeltaKind, DeltaSpec, Requirement, Scenario};
use serde::Serialize;

pub const DEFAULT_CAPABILITY: &str = "definitions";
pub const DEFAULT_MIN_RECURRENCE: usize = 3;

/// A glossary entry. The name is the term, the body its meaning, the
/// scenarios usage examples; `- **Admitted:**` lists the other words that
/// mean it and `- **Deprecated:**` the words not to use for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Term {
    #[serde(rename = "term")]
    pub name: String,
    pub meaning: String,
    pub admitted: Vec<String>,
    pub deprecated: Vec<String>,
    #[serde(skip)]
    pub examples: Vec<Scenario>,
}

impl Term {
    pub fn from_requirement(req: &Requirement) -> Term {
        let m = parse_markers(&req.body);
        Term {
            name: req.name.clone(),
            meaning: m.meaning,
            admitted: m.admitted,
            deprecated: m.deprecated,
            examples: req.scenarios.clone(),
        }
    }

    /// The words that are acceptable for this term: its name first, then
    /// its admitted synonyms.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.name.as_str()).chain(self.admitted.iter().map(String::as_str))
    }

    /// Whether any acceptable word for this term appears in `text`.
    pub fn used_in(&self, text: &str) -> bool {
        self.names().any(|n| Matcher::new(n).is_match(text))
    }

    /// The earliest offset at which any acceptable word for this term
    /// appears in `text`.
    pub fn first_in(&self, text: &str) -> Option<usize> {
        self.names()
            .filter_map(|n| Matcher::new(n).first(text))
            .min()
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

    /// Terms, admitted synonyms and deprecated synonyms alike: every word
    /// the glossary accounts for.
    pub fn knows(&self, word: &str) -> bool {
        self.terms.iter().any(|t| {
            t.names()
                .chain(t.deprecated.iter().map(String::as_str))
                .any(|n| n.eq_ignore_ascii_case(word))
        })
    }

    /// Terms present in `text`, ordered by first appearance. A term appears
    /// where its name or any admitted synonym does, whichever comes first.
    pub fn terms_in<'a>(&'a self, text: &str) -> Vec<&'a Term> {
        let mut found: Vec<(usize, &Term)> = self
            .terms
            .iter()
            .filter_map(|t| t.first_in(text).map(|at| (at, t)))
            .collect();
        found.sort_by_key(|(at, _)| *at);
        found.into_iter().map(|(_, t)| t).collect()
    }
}
