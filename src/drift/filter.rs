//! Phrases that are noise: stop words, spec keywords, and anything canon
//! says everywhere.

use super::terms::{contains_phrase, words, RemovedTerm, Tier};

const STOP: &[&str] = &[
    "a",
    "an",
    "the",
    "and",
    "or",
    "but",
    "of",
    "to",
    "in",
    "on",
    "at",
    "by",
    "for",
    "with",
    "from",
    "as",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "it",
    "its",
    "this",
    "that",
    "these",
    "those",
    "there",
    "here",
    "when",
    "then",
    "given",
    "must",
    "not",
    "no",
    "any",
    "each",
    "every",
    "all",
    "one",
    "two",
    "into",
    "onto",
    "over",
    "under",
    "after",
    "before",
    "than",
    "so",
    "if",
    "else",
    "which",
    "who",
    "whom",
    "whose",
    "what",
    "where",
    "how",
    "does",
    "do",
    "did",
    "has",
    "have",
    "had",
    "can",
    "may",
    "shall",
    "should",
    "will",
    "would",
    "also",
    "only",
    "same",
    "other",
    "such",
    "more",
    "most",
    "own",
    "per",
    "via",
    "up",
    "out",
    "off",
    "about",
    "without",
    "within",
    "them",
    "they",
    "their",
    "we",
    "you",
    "your",
    "our",
    "i",
    "he",
    "she",
    "his",
    "her",
    "them",
    "scenario",
    "requirement",
    "requirements",
];

pub fn is_stop(word: &str) -> bool {
    STOP.contains(&word)
}

/// True when every word is a stop word or a spec keyword.
pub fn all_stop(phrase: &str) -> bool {
    words(phrase).iter().all(|w| is_stop(w))
}

/// Phrases that are all stop words go; phrases in more than `max_common`
/// canon requirements go. Backticked and quoted terms always stay: an
/// identifier used everywhere that one change removes is the case worth
/// every one of its findings.
pub fn filter_terms<'a>(
    terms: Vec<RemovedTerm>,
    canon_texts: impl Fn() -> Box<dyn Iterator<Item = String> + 'a>,
    max_common: usize,
) -> Vec<RemovedTerm> {
    terms
        .into_iter()
        .filter(|t| {
            if t.tier != Tier::Phrase {
                return true;
            }
            if all_stop(&t.text) {
                return false;
            }
            let count = canon_texts()
                .filter(|text| contains_phrase(text, &t.text))
                .count();
            count <= max_common
        })
        .collect()
}
