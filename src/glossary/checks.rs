//! The glossary checks as pure functions over the glossary, canon and the
//! pairings of the change under review.

use super::{Glossary, Matcher};
use crate::drift::terms::quoted_spans;
use crate::model::{Canon, DeltaKind, DeltaSpec, Ignores, Requirement};
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
    let marks = glossary.marks();
    let mut out = Vec::new();
    for p in pairings
        .iter()
        .filter(|p| p.capability != glossary.capability)
    {
        let Some(after) = &p.after else { continue };
        let in_body = marks.live_deprecated(&after.body);
        let in_scenarios: Vec<Vec<(&str, &str)>> = after
            .scenarios
            .iter()
            .map(|s| {
                let mut both = marks.live_deprecated(&s.body);
                both.extend(marks.live_deprecated(&s.name));
                both
            })
            .collect();
        for term in &glossary.terms {
            for synonym in &term.deprecated {
                let pair = (term.name.as_str(), synonym.as_str());
                let kind = || FindingKind::UsesDeprecatedSynonym {
                    synonym: synonym.clone(),
                    term: term.name.clone(),
                };
                if in_body.contains(&pair) {
                    out.push(Finding::new(kind(), p.location()));
                }
                for (s, live) in after.scenarios.iter().zip(&in_scenarios) {
                    if live.contains(&pair) {
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
    let marks = glossary.marks();
    let mut out = Vec::new();
    for (cap, req) in outside(canon, glossary) {
        let in_body = marks.live_deprecated(&req.body);
        let in_scenarios: Vec<Vec<(&str, &str)>> = req
            .scenarios
            .iter()
            .map(|s| {
                let mut both = marks.live_deprecated(&s.body);
                both.extend(marks.live_deprecated(&s.name));
                both
            })
            .collect();
        for term in &glossary.terms {
            for synonym in &term.deprecated {
                let pair = (term.name.as_str(), synonym.as_str());
                let hit = |scenario: Option<String>| CanonHit {
                    term: term.name.clone(),
                    synonym: synonym.clone(),
                    capability: cap.to_string(),
                    requirement: req.name.clone(),
                    scenario,
                };
                if in_body.contains(&pair) {
                    out.push(hit(None));
                }
                for (s, live) in req.scenarios.iter().zip(&in_scenarios) {
                    if live.contains(&pair) {
                        out.push(hit(Some(s.name.clone())));
                    }
                }
            }
        }
    }
    out
}

/// Terms whose name and admitted synonyms appear in no canon requirement
/// outside the glossary and in no delta of any open change.
pub fn unused_terms<'a>(
    glossary: &'a Glossary,
    canon: &Canon,
    open_deltas: &[crate::model::DeltaSpec],
) -> Vec<&'a super::Term> {
    glossary
        .terms
        .iter()
        .filter(|t| {
            let in_canon = outside(canon, glossary).any(|(_, r)| t.used_in(&requirement_text(r)));
            let in_deltas = open_deltas
                .iter()
                .filter(|d| d.capability != glossary.capability)
                .flat_map(|d| d.entries.iter())
                .any(|e| t.used_in(&requirement_text(&e.requirement)));
            !in_canon && !in_deltas
        })
        .collect()
}

/// Terms whose body does not open with a binding line naming them, either
/// because there is none or because the line names something else.
pub fn unbound_terms(glossary: &Glossary) -> Vec<&super::Term> {
    glossary
        .terms
        .iter()
        .filter(|t| !t.binding.is_bound())
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
    ignores: &Ignores,
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
        .map(|(term, where_)| {
            let kept = where_
                .into_iter()
                .filter(|(c, r)| !ignores.hides(&term, c, r))
                .collect::<Vec<_>>();
            (term, kept)
        })
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

/// Every requirement a quoted span appears in, across canon outside the
/// glossary and the deltas of every open change. An ignore entry is live
/// where this says its term still occurs, which makes the verdict a
/// property of the repository rather than of the subcommand that ran.
pub fn span_sites(
    glossary: &Glossary,
    canon: &Canon,
    open_deltas: &[DeltaSpec],
) -> BTreeMap<String, Vec<(String, String)>> {
    let mut sites: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let canon_texts = outside(canon, glossary)
        .map(|(cap, req)| (cap.to_string(), req.name.clone(), requirement_text(req)));
    let delta_texts = open_deltas
        .iter()
        .filter(|d| d.capability != glossary.capability)
        .flat_map(|d| {
            d.entries.iter().map(move |e| {
                (
                    d.capability.clone(),
                    e.requirement.name.clone(),
                    requirement_text(&e.requirement),
                )
            })
        });
    for (cap, name, text) in canon_texts.chain(delta_texts) {
        for span in quoted_spans(&text) {
            let at = sites.entry(span).or_default();
            let site = (cap.clone(), name.clone());
            if !at.contains(&site) {
                at.push(site);
            }
        }
    }
    sites
}

/// Spans on any after side and on no before side of the change, used at
/// least twice, that the glossary does not know: a warning on the first
/// pairing that introduces each.
pub fn new_undefined(
    glossary: &Glossary,
    pairings: &[&Pairing],
    ignores: &Ignores,
) -> Vec<Finding> {
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
            if ignores.hides(&span, &p.capability, &p.name) {
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
