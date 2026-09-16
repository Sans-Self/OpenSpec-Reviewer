//! Prompt assembly. Pure: a pairing and its context in, one string out.
//!
//! The template owns the task text; the assembler owns the data sections.
//! A section with no data is left out rather than written as an empty
//! heading, so the agent is never told "Glossary:" followed by nothing.

use crate::glossary::{Glossary, Term};
use crate::model::{Canon, Requirement};
use crate::review::pair::requirement_text;
use crate::review::Pairing;
use std::fmt::Write;

/// A canon requirement quoted because the pairing's drift findings point
/// at it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiblingText {
    pub capability: String,
    pub requirement: String,
    pub text: String,
}

/// Everything a pairing prompt quotes besides the pairing itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptContext {
    /// `rules.specs` from `openspec/config.yaml`, verbatim.
    pub rules: Vec<String>,
    pub siblings: Vec<SiblingText>,
    pub glossary: Vec<Term>,
    /// The hint schema, appended for a batch run only.
    pub schema: Option<String>,
}

/// The glossary terms a text uses, in glossary order.
pub fn glossary_terms(glossary: &Glossary, text: &str) -> Vec<Term> {
    glossary
        .terms
        .iter()
        .filter(|t| t.used_in(text))
        .cloned()
        .collect()
}

fn section(out: &mut String, heading: &str, body: &str) {
    if body.trim().is_empty() {
        return;
    }
    let _ = writeln!(out, "## {heading}\n\n{}\n", body.trim_end());
}

fn requirement_block(req: &Requirement) -> String {
    let mut out = format!("### Requirement: {}\n\n{}\n", req.name, req.body.trim_end());
    for s in &req.scenarios {
        let _ = write!(
            out,
            "\n#### Scenario: {}\n\n{}\n",
            s.name,
            s.body.trim_end()
        );
    }
    out
}

fn rules_block(rules: &[String]) -> String {
    rules
        .iter()
        .map(|r| format!("- {}", r.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn findings_block(p: &Pairing) -> String {
    p.findings
        .iter()
        .map(|f| format!("- {}  {}", f.severity, f.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn siblings_block(siblings: &[SiblingText]) -> String {
    siblings
        .iter()
        .map(|s| {
            format!(
                "### {} § {}\n\n{}",
                s.capability,
                s.requirement,
                s.text.trim_end()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn glossary_block(terms: &[Term]) -> String {
    terms
        .iter()
        .map(|t| {
            let mut entry = format!("### {}\n\n{}", t.name, t.meaning.trim_end());
            if !t.admitted.is_empty() {
                let _ = write!(entry, "\n\nAdmitted: {}", t.admitted.join(", "));
            }
            if !t.deprecated.is_empty() {
                let _ = write!(entry, "\n\nDeprecated: {}", t.deprecated.join(", "));
            }
            entry
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn history_block(p: &Pairing) -> String {
    if p.history.is_empty() {
        return String::new();
    }
    p.history
        .iter()
        .map(|h| format!("- {}", h.label()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The body of a pairing's sections, without the task template. The
/// change prompt reuses it, one block per pairing.
pub fn pairing_sections(p: &Pairing, context: &PromptContext) -> String {
    let mut out = String::new();
    section(
        &mut out,
        "Pairing",
        &format!(
            "kind: {}\ncapability: {}\nname: {}",
            p.kind.label(),
            p.capability,
            p.name
        ),
    );
    if let Some(before) = &p.before {
        section(&mut out, "Before", &requirement_block(before));
    }
    if let Some(after) = &p.after {
        section(&mut out, "After", &requirement_block(after));
    }
    section(&mut out, "Known findings", &findings_block(p));
    section(
        &mut out,
        "Sibling texts",
        &siblings_block(&context.siblings),
    );
    section(&mut out, "Glossary", &glossary_block(&context.glossary));
    section(
        &mut out,
        "Reviewer note",
        p.state.note.as_deref().unwrap_or_default(),
    );
    section(&mut out, "History", &history_block(p));
    out
}

/// The whole prompt file for one pairing.
pub fn pairing_prompt(template: &str, p: &Pairing, context: &PromptContext) -> String {
    let mut out = format!("{}\n\n", template.trim_end());
    section(&mut out, "Project rules", &rules_block(&context.rules));
    out.push_str(&pairing_sections(p, context));
    if let Some(schema) = &context.schema {
        let _ = writeln!(out, "{}", schema.trim_end());
    }
    out
}

/// The whole prompt file for a change: the proposal, then one section per
/// pairing under its own heading.
pub fn change_prompt(
    template: &str,
    change: &str,
    proposal: Option<&str>,
    rules: &[String],
    pairings: &[(&Pairing, PromptContext)],
    schema: Option<&str>,
) -> String {
    let mut out = format!("{}\n\n", template.trim_end());
    section(&mut out, "Project rules", &rules_block(rules));
    section(&mut out, "Change", change);
    section(&mut out, "Proposal", proposal.unwrap_or_default());
    for (p, context) in pairings {
        let _ = writeln!(out, "# {} § {}\n", p.capability, p.name);
        out.push_str(&pairing_sections(p, context));
    }
    if let Some(schema) = schema {
        let _ = writeln!(out, "{}", schema.trim_end());
    }
    out
}

/// The sibling requirements a pairing's drift findings name, with the
/// canon text of each.
pub fn sibling_texts(p: &Pairing, canon: &Canon) -> Vec<SiblingText> {
    use crate::review::FindingKind;
    let mut out: Vec<SiblingText> = Vec::new();
    for finding in &p.findings {
        let sibling = match &finding.kind {
            FindingKind::SiblingUsesRemoved { sibling, .. }
            | FindingKind::SiblingUsesOldName { sibling, .. } => sibling,
            _ => continue,
        };
        if out
            .iter()
            .any(|s| s.capability == sibling.capability && s.requirement == sibling.requirement)
        {
            continue;
        }
        let Some(req) = canon.get(&sibling.capability, &sibling.requirement) else {
            continue;
        };
        out.push(SiblingText {
            capability: sibling.capability.clone(),
            requirement: sibling.requirement.clone(),
            text: requirement_text(req),
        });
    }
    out
}
