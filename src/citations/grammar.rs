//! The citation grammars. The line-bound literal and the call form are
//! ported from the TypeScript lint because their edge cases were earned
//! there; the backticked span that wraps came from Elixir moduledocs.

use super::Citation;
use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

/// Literal: `spec:<capability> § <name>`. The name runs to a double quote,
/// backtick or newline, so an apostrophe survives. Single-quoted citations
/// are therefore unsupported, which the convention already states.
static LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"spec:([a-z0-9-]+)\s*§\s*([^"`\n]+)"#).expect("literal grammar compiles")
});

/// The same tag opened with a backtick: the name runs to the closing
/// backtick, line breaks included.
static SPAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"`spec:([a-z0-9-]+)\s*§\s*([^`]+)`").expect("span grammar compiles")
});

/// What a continuation line inside a span may start with that is not part
/// of the name: `#`, `//`, `///`, `//!`, `*`, `--`.
static MARKER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:#+|/{2,}!?|\*|--)\s*").expect("marker grammar compiles"));

/// A citation as found in text, with what the error message needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub citation: Citation,
    /// A line-bound match that stopped at a newline: the name may have
    /// been cut off by wrapping.
    pub ends_at_line_break: bool,
}

/// Only the call form depends on configuration; the rest of the grammar is
/// the same for every reader, so it is compiled once per process.
pub struct Grammar {
    call: Option<Regex>,
}

impl Grammar {
    pub fn new(helper: Option<&str>) -> Grammar {
        let call = helper.map(|h| {
            Regex::new(&format!(
                r#"{}\(\s*['"]([a-z0-9-]+)['"]\s*,\s*(?:"([^"]+)"|'([^']+)')\s*,?\s*\)"#,
                regex::escape(h)
            ))
            .expect("call grammar compiles")
        });
        Grammar { call }
    }

    pub fn literal(text: &str) -> Vec<Citation> {
        Grammar::new(None).citations(text)
    }

    pub fn citations(&self, text: &str) -> Vec<Citation> {
        self.found(text).into_iter().map(|f| f.citation).collect()
    }

    /// Continuation lines lose their indentation and one comment marker;
    /// the lines then join with a space.
    fn unwrap_span(name: &str) -> String {
        name.split('\n')
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    line.to_string()
                } else {
                    MARKER.replace(line.trim_start(), "").into_owned()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Every citation in `text`: backticked spans first, then line-bound
    /// literals that do not start inside a span, then call forms.
    pub fn found(&self, text: &str) -> Vec<Found> {
        let mut out = Vec::new();
        let mut spans: Vec<Range<usize>> = Vec::new();
        for c in SPAN.captures_iter(text) {
            let whole = c.get(0).expect("match");
            spans.push(whole.range());
            out.push(Found {
                citation: Citation::new(&c[1], &Grammar::unwrap_span(&c[2])),
                ends_at_line_break: false,
            });
        }
        for c in LITERAL.captures_iter(text) {
            let whole = c.get(0).expect("match");
            if spans.iter().any(|r| r.contains(&whole.start())) {
                continue;
            }
            out.push(Found {
                citation: Citation::new(&c[1], &c[2]),
                ends_at_line_break: text[whole.end()..].starts_with('\n'),
            });
        }
        if let Some(re) = &self.call {
            for c in re.captures_iter(text) {
                let name = c.get(2).or_else(|| c.get(3)).map_or("", |m| m.as_str());
                out.push(Found {
                    citation: Citation::new(&c[1], name),
                    ends_at_line_break: false,
                });
            }
        }
        out
    }

    /// Text with every literal citation removed, for searches that must not
    /// count formal citations as prose.
    pub fn strip(text: &str) -> String {
        let without_spans = SPAN.replace_all(text, "");
        LITERAL.replace_all(&without_spans, "").into_owned()
    }

    /// The same, with each citation replaced by spaces of its own byte
    /// length instead of removed. An offset into the result is an offset
    /// into `text`, which is what lets a caller mark what it found.
    pub fn blank(text: &str) -> String {
        let pad = |c: &regex::Captures| " ".repeat(c[0].len());
        let without_spans = SPAN.replace_all(text, pad);
        LITERAL.replace_all(&without_spans, pad).into_owned()
    }
}
