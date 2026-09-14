//! Search canon outside the pairing's capability for removed terms and
//! old names, then decide per hit whether the change already handles it.

use super::filter::filter_terms;
use super::terms::{contains_phrase, removed_terms, words, RemovedTerm, Tier};
use crate::citations::Grammar;
use crate::model::{Canon, DeltaKind, DeltaSpec, Requirement};
use crate::review::pair::requirement_text;
use crate::review::{Finding, FindingKind, Pairing, Sibling};

fn phrase_text(req: &Requirement) -> String {
    words(&requirement_text(req)).join(" ")
}

fn mentions(term: &RemovedTerm, req: &Requirement) -> bool {
    match term.tier {
        Tier::Phrase => contains_phrase(&phrase_text(req), &term.text),
        _ => requirement_text(req)
            .to_lowercase()
            .contains(&term.text.to_lowercase()),
    }
}

fn sibling(capability: &str, req: &Requirement) -> Sibling {
    Sibling {
        capability: capability.to_string(),
        requirement: req.name.clone(),
        path: format!("openspec/specs/{capability}/spec.md"),
    }
}

/// The after text of `name` in this change's delta for `capability`, if the
/// change touches that requirement.
fn delta_after<'a>(
    deltas: &'a [DeltaSpec],
    capability: &str,
    name: &str,
) -> Option<Option<&'a Requirement>> {
    let delta = deltas.iter().find(|d| d.capability == capability)?;
    let entry = delta.entries.iter().find(|e| {
        e.requirement.name == name || matches!(&e.kind, DeltaKind::Renamed { from } if from == name)
    })?;
    Some(match entry.kind {
        DeltaKind::Removed => None,
        _ => Some(&entry.requirement),
    })
}

/// Drift findings for one pairing. `deltas` are the change's own, so a
/// sibling the change already rewrites without the term is not a finding.
pub fn drift_findings(
    p: &Pairing,
    canon: &Canon,
    deltas: &[DeltaSpec],
    max_common: usize,
) -> Vec<Finding> {
    if p.kind == DeltaKind::Added {
        return Vec::new();
    }
    let location = p.location();
    let others = || {
        canon
            .specs
            .iter()
            .filter(|(cap, _)| **cap != p.capability)
            .flat_map(|(cap, reqs)| reqs.iter().map(move |r| (cap.as_str(), r)))
    };
    let canon_texts = || -> Box<dyn Iterator<Item = String>> {
        Box::new(
            canon
                .specs
                .values()
                .flatten()
                .map(phrase_text)
                .collect::<Vec<_>>()
                .into_iter(),
        )
    };

    let terms = filter_terms(
        removed_terms(p.before.as_ref(), p.after.as_ref()),
        canon_texts,
        max_common,
    );
    let mut findings = Vec::new();
    for term in &terms {
        for (cap, req) in others() {
            if !mentions(term, req) {
                continue;
            }
            let kept_in_delta = match delta_after(deltas, cap, &req.name) {
                Some(None) => continue,
                Some(Some(after)) if !mentions(term, after) => continue,
                Some(Some(_)) => true,
                None => false,
            };
            findings.push(Finding::new(
                FindingKind::SiblingUsesRemoved {
                    term: term.text.clone(),
                    sibling: sibling(cap, req),
                    kept_in_delta,
                },
                location.clone(),
            ));
        }
    }

    if let DeltaKind::Renamed { from } = &p.kind {
        let needle = from.to_lowercase();
        for (cap, req) in others() {
            let prose = Grammar::strip(&requirement_text(req)).to_lowercase();
            if !prose.contains(&needle) {
                continue;
            }
            if let Some(Some(after)) = delta_after(deltas, cap, &req.name) {
                if !Grammar::strip(&requirement_text(after))
                    .to_lowercase()
                    .contains(&needle)
                {
                    continue;
                }
            }
            findings.push(Finding::new(
                FindingKind::SiblingUsesOldName {
                    from: from.clone(),
                    sibling: sibling(cap, req),
                },
                location.clone(),
            ));
        }
    }
    findings
}
