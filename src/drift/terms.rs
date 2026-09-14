//! What a pairing removes, in three tiers.

use crate::model::Requirement;
use crate::review::pair::requirement_text;
use regex::Regex;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Backticked,
    Quoted,
    Phrase,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RemovedTerm {
    pub tier: Tier,
    pub text: String,
}

/// Every backticked span and double-quoted string in `text`, the two tiers
/// the glossary checks and term drift share.
pub fn quoted_spans(text: &str) -> BTreeSet<String> {
    let backtick = Regex::new("`([^`\n]+)`").expect("compiles");
    let quoted = Regex::new("\"([^\"\n]+)\"").expect("compiles");
    let mut out = spans(&backtick, text);
    out.extend(spans(&quoted, text));
    out
}

fn spans(re: &Regex, text: &str) -> BTreeSet<String> {
    re.captures_iter(text)
        .map(|c| c[1].trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Words for phrase matching: lowercase, punctuation stripped from the
/// edges, so "badge." and "badge" are one word.
pub fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

fn bigrams(words: &[String]) -> BTreeSet<(String, String)> {
    words
        .windows(2)
        .map(|w| (w[0].clone(), w[1].clone()))
        .collect()
}

/// Maximal runs of consecutive words whose every adjacent pair is gone from
/// the after side, trimmed of stop words at both ends. A rewritten sentence
/// yields one long phrase that matches nothing; a dropped "status badge"
/// yields exactly that.
pub fn removed_phrases(before: &[String], after: &[String]) -> Vec<String> {
    let gone: BTreeSet<(String, String)> = bigrams(before)
        .difference(&bigrams(after))
        .cloned()
        .collect();
    let mut runs: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for pair in before.windows(2) {
        if gone.contains(&(pair[0].clone(), pair[1].clone())) {
            if current.is_empty() {
                current.push(&pair[0]);
            }
            current.push(&pair[1]);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    let mut seen = BTreeSet::new();
    runs.into_iter()
        .filter_map(|run| {
            let start = run.iter().position(|w| !super::filter::is_stop(w))?;
            let end = run.iter().rposition(|w| !super::filter::is_stop(w))?;
            let phrase = run[start..=end].join(" ");
            (run[start..=end].len() >= 2
                && phrase.chars().count() >= 8
                && seen.insert(phrase.clone()))
            .then_some(phrase)
        })
        .collect()
}

fn side_text(req: Option<&Requirement>) -> String {
    req.map(requirement_text).unwrap_or_default()
}

/// Terms on the before side and not on the after side, compared
/// case-insensitively.
pub fn removed_terms(
    before: Option<&Requirement>,
    after: Option<&Requirement>,
) -> Vec<RemovedTerm> {
    let before = side_text(before);
    let after = side_text(after);
    let after_lower = after.to_lowercase();
    let backtick = Regex::new("`([^`\n]+)`").expect("compiles");
    let quoted = Regex::new("\"([^\"\n]+)\"").expect("compiles");

    let mut out: Vec<RemovedTerm> = Vec::new();
    for (tier, re) in [(Tier::Backticked, &backtick), (Tier::Quoted, &quoted)] {
        out.extend(
            spans(re, &before)
                .into_iter()
                .filter(|s| !after_lower.contains(&s.to_lowercase()))
                .map(|text| RemovedTerm { tier, text }),
        );
    }

    out.extend(
        removed_phrases(&words(&before), &words(&after))
            .into_iter()
            .map(|text| RemovedTerm {
                tier: Tier::Phrase,
                text,
            }),
    );
    out
}

/// Whole-word containment: "the status badge" contains "status badge" but
/// "restatus badge" does not.
pub fn contains_phrase(haystack: &str, needle: &str) -> bool {
    haystack.match_indices(needle).any(|(i, _)| {
        let before_ok = i == 0 || haystack.as_bytes()[i - 1] == b' ';
        let end = i + needle.len();
        let after_ok = end == haystack.len() || haystack.as_bytes()[end] == b' ';
        before_ok && after_ok
    })
}
