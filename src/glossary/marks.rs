//! Where the glossary's words stand in a text. One rule with two readers:
//! the deprecation checks ask whether a synonym is live, the view asks
//! which cells to mark. Keeping both on this function is the point — two
//! implementations of "is this synonym shielded" would drift.

use super::{Glossary, Matcher, Term};
use std::ops::Range;

/// What a marked span is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark<'a> {
    /// The term's own name or one of its admitted synonyms.
    Admitted,
    /// A synonym the glossary deprecates, standing outside every admitted
    /// name that contains it.
    Deprecated(&'a str),
}

/// One of the glossary's words, found at a range that indexes the text it
/// was found in.
#[derive(Debug, Clone)]
pub struct Occurrence<'a> {
    pub range: Range<usize>,
    pub term: &'a Term,
    pub mark: Mark<'a>,
}

/// A glossary with its matchers compiled. Compiling is the expensive part,
/// so a caller reading many texts builds this once and reads with it. The
/// terms are owned rather than borrowed, so a long-lived reader — the view,
/// which holds one for the session — can keep the matchers next to the
/// review they were built from.
pub struct Marks {
    terms: Vec<Term>,
    names: Vec<(usize, Matcher)>,
    /// Names of two or more words, the only ones that can contain a
    /// synonym. Admitting a one-word name here would let one term's
    /// admission silence another term's deprecation everywhere.
    shields: Vec<Matcher>,
    deprecated: Vec<(usize, String, Matcher)>,
}

impl Glossary {
    pub fn marks(&self) -> Marks {
        Marks {
            terms: self.terms.clone(),
            names: self
                .terms
                .iter()
                .enumerate()
                .flat_map(|(i, t)| t.names().map(move |n| (i, Matcher::new(n))))
                .collect(),
            shields: self
                .terms
                .iter()
                .flat_map(|t| t.names())
                .filter(|n| n.split_whitespace().count() > 1)
                .map(Matcher::new)
                .collect(),
            deprecated: self
                .terms
                .iter()
                .enumerate()
                .flat_map(|(i, t)| t.deprecated.iter().map(move |d| (i, d.clone())))
                .map(|(i, d)| {
                    let matcher = Matcher::new(&d);
                    (i, d, matcher)
                })
                .collect(),
        }
    }
}

fn inside(hit: &Range<usize>, covered: &[Range<usize>]) -> bool {
    covered
        .iter()
        .any(|c| c.start <= hit.start && hit.end <= c.end)
}

fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

impl Marks {
    /// Every occurrence to mark in `text`, ordered by position and never
    /// overlapping. A deprecated synonym outranks an admitted name over the
    /// same word, so a synonym is never marked as less than a plain term.
    ///
    /// The text is blanked of its citations once here rather than once per
    /// matcher: a glossary of twenty terms otherwise blanks every line it
    /// reads forty times over.
    pub fn occurrences<'a>(&'a self, text: &str) -> Vec<Occurrence<'a>> {
        let cleaned = Matcher::cleaned(text);
        let covered: Vec<Range<usize>> = self
            .shields
            .iter()
            .flat_map(|s| s.ranges_cleaned(&cleaned))
            .collect();
        let mut kept: Vec<Occurrence<'a>> = self
            .deprecated
            .iter()
            .flat_map(|(term, synonym, m)| {
                m.ranges_cleaned(&cleaned)
                    .into_iter()
                    .filter(|hit| !inside(hit, &covered))
                    .map(move |range| Occurrence {
                        range,
                        term: &self.terms[*term],
                        mark: Mark::Deprecated(synonym),
                    })
            })
            .collect();

        // The longest admitted name wins, so `PLC rotation key` underlines
        // as one word rather than as two overlapping ones.
        let mut admitted: Vec<Occurrence<'a>> = self
            .names
            .iter()
            .flat_map(|(term, m)| {
                m.ranges_cleaned(&cleaned)
                    .into_iter()
                    .map(move |range| Occurrence {
                        range,
                        term: &self.terms[*term],
                        mark: Mark::Admitted,
                    })
            })
            .collect();
        admitted.sort_by_key(|o| (o.range.start, std::cmp::Reverse(o.range.len())));
        for o in admitted {
            if !kept.iter().any(|k| overlaps(&k.range, &o.range)) {
                kept.push(o);
            }
        }
        kept.sort_by_key(|o| o.range.start);
        kept
    }

    /// The `(term, synonym)` pairs the glossary deprecates that stand in
    /// `text` outside every name containing them.
    pub fn live_deprecated<'a>(&'a self, text: &str) -> Vec<(&'a str, &'a str)> {
        let mut out: Vec<(&str, &str)> = self
            .occurrences(text)
            .into_iter()
            .filter_map(|o| match o.mark {
                Mark::Deprecated(s) => Some((o.term.name.as_str(), s)),
                Mark::Admitted => None,
            })
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}
