//! Prompt assembly. Pure: a pairing and its context in, one string out.
//!
//! The template owns the task text; the assembler owns the data sections.
//! A section with no data is left out rather than written as an empty
//! heading, so the agent is never told "Glossary:" followed by nothing.

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

/// A glossary term the after text uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryTerm {
    pub name: String,
    pub meaning: String,
    pub deprecated: Vec<String>,
}

/// Everything a pairing prompt quotes besides the pairing itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptContext {
    /// `rules.specs` from `openspec/config.yaml`, verbatim.
    pub rules: Vec<String>,
    pub siblings: Vec<SiblingText>,
    pub glossary: Vec<GlossaryTerm>,
    /// The hint schema, appended for a batch run only.
    pub schema: Option<String>,
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

fn glossary_block(terms: &[GlossaryTerm]) -> String {
    terms
        .iter()
        .map(|t| {
            let mut entry = format!("### {}\n\n{}", t.name, t.meaning.trim_end());
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

/// The glossary terms a text uses, in canon order. Whole-word and
/// case-insensitive, with a trailing `s` on the candidate treated as the
/// same word.
pub fn glossary_terms(canon: &Canon, capability: &str, text: &str) -> Vec<GlossaryTerm> {
    let Some(terms) = canon.specs.get(capability) else {
        return Vec::new();
    };
    let haystack = normalize_words(text);
    terms
        .iter()
        .filter(|t| contains_term(&haystack, &t.name))
        .map(|t| {
            let (meaning, deprecated) = strip_deprecated(&t.body);
            GlossaryTerm {
                name: t.name.clone(),
                meaning,
                deprecated,
            }
        })
        .collect()
}

/// Lowercase words separated by single spaces, punctuation stripped from
/// the edges, so a term matches on word boundaries and not inside one.
fn normalize_words(text: &str) -> String {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    format!(" {} ", words.join(" "))
}

fn contains_term(haystack: &str, term: &str) -> bool {
    let needle = normalize_words(term);
    let needle = needle.trim();
    if needle.is_empty() {
        return false;
    }
    haystack.contains(&format!(" {needle} ")) || haystack.contains(&format!(" {needle}s "))
}

/// A term's body split into its meaning and the `- **Deprecated:**` list.
fn strip_deprecated(body: &str) -> (String, Vec<String>) {
    const MARKER: &str = "- **Deprecated:**";
    let mut meaning = String::new();
    let mut deprecated = Vec::new();
    for line in body.lines() {
        match line.trim().strip_prefix(MARKER) {
            Some(list) => deprecated.extend(
                list.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
            ),
            None => {
                meaning.push_str(line);
                meaning.push('\n');
            }
        }
    }
    (meaning.trim().to_string(), deprecated)
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
