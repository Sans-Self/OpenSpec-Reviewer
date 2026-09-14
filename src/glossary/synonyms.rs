//! The `- **Deprecated:**` line and the whole-word matcher.

use crate::citations::Grammar;
use regex::Regex;

const DEPRECATED: &str = "- **Deprecated:**";

/// The body without the Deprecated line, and the comma-separated words the
/// line listed.
pub fn parse_deprecated(body: &str) -> (String, Vec<String>) {
    let mut meaning = Vec::new();
    let mut deprecated = Vec::new();
    for line in body.lines() {
        match line.trim_start().strip_prefix(DEPRECATED) {
            Some(rest) => deprecated.extend(
                rest.split(',')
                    .map(|s| s.trim().trim_matches('`').trim().to_string())
                    .filter(|s| !s.is_empty()),
            ),
            None => meaning.push(line),
        }
    }
    (meaning.join("\n").trim().to_string(), deprecated)
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
