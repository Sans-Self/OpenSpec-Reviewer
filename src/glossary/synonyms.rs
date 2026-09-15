//! The acceptability marker lines and the whole-word matcher.

use crate::citations::Grammar;
use regex::Regex;

/// The ISO 704 acceptability ratings a term may spell out. The preferred
/// term is the requirement name, so it needs no line of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Marker {
    Admitted,
    Deprecated,
}

/// Only these bold-prefixed lines are lifted out of the meaning. An
/// author's own `- **Foo:**` bullet is prose and stays put.
const MARKERS: [(Marker, &str); 2] = [
    (Marker::Admitted, "- **Admitted:**"),
    (Marker::Deprecated, "- **Deprecated:**"),
];

/// A term's body split into prose and the words each marker line listed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Markers {
    pub meaning: String,
    pub admitted: Vec<String>,
    pub deprecated: Vec<String>,
}

fn listed(rest: &str) -> impl Iterator<Item = String> + '_ {
    rest.split(',')
        .map(|s| s.trim().trim_matches('`').trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The body without its marker lines, and the words each one listed.
/// Removing a marker line takes the blank line that followed it with it,
/// so two markers in a row do not leave a gap in the prose.
pub fn parse_markers(body: &str) -> Markers {
    let mut out = Markers::default();
    let mut meaning: Vec<&str> = Vec::new();
    let mut removed = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        match MARKERS
            .iter()
            .find_map(|(m, p)| trimmed.strip_prefix(p).map(|rest| (*m, rest)))
        {
            Some((Marker::Admitted, rest)) => {
                out.admitted.extend(listed(rest));
                removed = true;
            }
            Some((Marker::Deprecated, rest)) => {
                out.deprecated.extend(listed(rest));
                removed = true;
            }
            None => {
                let blank = line.trim().is_empty();
                let after_blank = meaning.last().is_none_or(|l| l.trim().is_empty());
                if removed && blank && after_blank {
                    continue;
                }
                meaning.push(line);
                removed = false;
            }
        }
    }
    out.meaning = meaning.join("\n").trim().to_string();
    out
}

/// Whole-word, case-insensitive, plural-tolerant. Runs of whitespace in the
/// text match one space in the phrase. Citations are blanked first so a
/// requirement name inside `spec:…§…` never counts as usage.
pub struct Matcher {
    re: Regex,
}

impl Matcher {
    pub fn new(phrase: &str) -> Matcher {
        let words: Vec<String> = phrase.split_whitespace().map(regex::escape).collect();
        let body = words.join(r"\s+");
        let starts_word = phrase.chars().next().is_some_and(|c| c.is_alphanumeric());
        let ends_word = phrase
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric());
        let pattern = format!(
            "(?i){}{}{}{}",
            if starts_word { r"\b" } else { "" },
            body,
            if ends_word { "(?:es|s)?" } else { "" },
            if ends_word { r"\b" } else { "" },
        );
        Matcher {
            re: Regex::new(&pattern).expect("matcher compiles"),
        }
    }

    fn cleaned(text: &str) -> String {
        Grammar::strip(text)
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.re.is_match(&Matcher::cleaned(text))
    }

    /// Byte offset of the first hit in the citation-stripped text; only the
    /// order of offsets is meaningful.
    pub fn first(&self, text: &str) -> Option<usize> {
        self.re.find(&Matcher::cleaned(text)).map(|m| m.start())
    }

    pub fn count(&self, text: &str) -> usize {
        self.re.find_iter(&Matcher::cleaned(text)).count()
    }
}
