//! The glossary checks as pure functions over the glossary, canon and the
//! pairings of the change under review.

use super::{Glossary, Matcher};
use crate::drift::terms::quoted_spans;
use crate::model::{Canon, DeltaKind, Requirement};
use crate::review::pair::requirement_text;
use crate::review::{Finding, FindingKind, Location, Pairing};
use std::collections::{BTreeMap, BTreeSet};

fn outside<'a>(
    canon: &'a Canon,
    glossary: &Glossary,
) -> impl Iterator<Item = (&'a str, &'a Requirement)> {
    let cap = glossary.capability.clone();
    canon
        .specs
        .iter()
        .filter(move |(c, _)| **c != cap)
        .flat_map(|(c, reqs)| reqs.iter().map(move |r| (c.as_str(), r)))
}

/// Deprecated synonyms in the pairings of the change under review, reported
/// on the pairing and, for a scenario hit, at the scenario.
pub fn deprecated_in_pairings(glossary: &Glossary, pairings: &[&Pairing]) -> Vec<Finding> {
    let mut out = Vec::new();
    for p in pairings
        .iter()
        .filter(|p| p.capability != glossary.capability)
    {
        let Some(after) = &p.after else { continue };
        for term in &glossary.terms {
            for synonym in &term.deprecated {
                let m = Matcher::new(synonym);
                let kind = || FindingKind::UsesDeprecatedSynonym {
                    synonym: synonym.clone(),
                    term: term.name.clone(),
                };
                if m.is_match(&after.body) {
                    out.push(Finding::new(kind(), p.location()));
                }
                for s in &after.scenarios {
                    if m.is_match(&s.body) || m.is_match(&s.name) {
                        out.push(Finding::new(
                            kind(),
                            Location {
                                scenario: Some(s.name.clone()),
                                ..p.location()
                            },
                        ));
                    }
                }
            }
        }
    }
    out
}

/// A deprecated synonym found in canon outside the glossary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonHit {
    pub term: String,
    pub synonym: String,
    pub capability: String,
    pub requirement: String,
    pub scenario: Option<String>,
}

pub fn deprecated_in_canon(glossary: &Glossary, canon: &Canon) -> Vec<CanonHit> {
    let mut out = Vec::new();
    for (cap, req) in outside(canon, glossary) {
        for term in &glossary.terms {
            for synonym in &term.deprecated {
                let m = Matcher::new(synonym);
                let hit = |scenario: Option<String>| CanonHit {
                    term: term.name.clone(),
                    synonym: synonym.clone(),
                    capability: cap.to_string(),
                    requirement: req.name.clone(),
                    scenario,
                };
                if m.is_match(&req.body) {
                    out.push(hit(None));
                }
                for s in &req.scenarios {
                    if m.is_match(&s.body) || m.is_match(&s.name) {
                        out.push(hit(Some(s.name.clone())));
                    }
                }
            }
        }
    }
    out
}

/// Terms that appear in no canon requirement outside the glossary and in no
/// delta of any open change.
pub fn unused_terms<'a>(
    glossary: &'a Glossary,
    canon: &Canon,
    open_deltas: &[crate::model::DeltaSpec],
) -> Vec<&'a super::Term> {
    glossary
        .terms
        .iter()
        .filter(|t| {
            let m = Matcher::new(&t.name);
            let in_canon = outside(canon, glossary).any(|(_, r)| m.is_match(&requirement_text(r)));
            let in_deltas = open_deltas
                .iter()
                .filter(|d| d.capability != glossary.capability)
                .flat_map(|d| d.entries.iter())
                .any(|e| m.is_match(&requirement_text(&e.requirement)));
            !in_canon && !in_deltas
        })
        .collect()
}

/// A backticked or quoted span that recurs across canon with no definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recurring {
    pub term: String,
    /// `capability § requirement`, one per requirement it appears in.
    pub uses: Vec<String>,
}

pub fn recurring_undefined(
    glossary: &Glossary,
    canon: &Canon,
    min_recurrence: usize,
) -> Vec<Recurring> {
    let mut uses: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (cap, req) in outside(canon, glossary) {
        for span in quoted_spans(&requirement_text(req)) {
            uses.entry(span)
                .or_default()
                .push((cap.to_string(), req.name.clone()));
        }
    }
    uses.into_iter()
        .filter(|(term, _)| !glossary.knows(term))
        .filter(|(_, where_)| {
            let caps: BTreeSet<&str> = where_.iter().map(|(c, _)| c.as_str()).collect();
            where_.len() >= min_recurrence && caps.len() >= 2
        })
        .map(|(term, where_)| Recurring {
            term,
            uses: where_.iter().map(|(c, r)| format!("{c} § {r}")).collect(),
        })
        .collect()
}

/// Spans on any after side and on no before side of the change, used at
/// least twice, that the glossary does not know: a warning on the first
/// pairing that introduces each.
pub fn new_undefined(glossary: &Glossary, pairings: &[&Pairing]) -> Vec<Finding> {
    let before: BTreeSet<String> = pairings
        .iter()
        .filter_map(|p| p.before.as_ref())
        .flat_map(|r| quoted_spans(&requirement_text(r)))
        .collect();
    let mut first: BTreeMap<String, usize> = BTreeMap::new();
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for (i, p) in pairings.iter().enumerate() {
        if p.capability == glossary.capability {
            continue;
        }
        let Some(after) = &p.after else { continue };
        let text = requirement_text(after);
        for span in quoted_spans(&text) {
            if before.contains(&span) || glossary.knows(&span) {
                continue;
            }
            *count.entry(span.clone()).or_default() += Matcher::new(&span).count(&text).max(1);
            first.entry(span).or_insert(i);
        }
    }
    let mut out: Vec<(usize, Finding)> = count
        .into_iter()
        .filter(|(_, n)| *n >= 2)
        .map(|(term, _)| {
            let i = first[&term];
            (
                i,
                Finding::new(
                    FindingKind::NewTermUndefined { term },
                    pairings[i].location(),
                ),
            )
        })
        .collect();
    out.sort_by_key(|(i, _)| *i);
    out.into_iter().map(|(_, f)| f).collect()
}

/// For a MODIFIED or RENAMED term, the canon requirements that use it.
pub fn term_in_use(glossary: &Glossary, canon: &Canon, p: &Pairing) -> Option<Finding> {
    if p.capability != glossary.capability
        || !matches!(p.kind, DeltaKind::Modified | DeltaKind::Renamed { .. })
    {
        return None;
    }
    let m = Matcher::new(&p.name);
    let uses: Vec<String> = outside(canon, glossary)
        .filter(|(_, r)| m.is_match(&requirement_text(r)))
        .map(|(c, r)| format!("{c} § {}", r.name))
        .collect();
    Some(Finding::new(FindingKind::TermInUse { uses }, p.location()))
}
