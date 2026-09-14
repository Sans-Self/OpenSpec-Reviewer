//! The two citation grammars, ported from the TypeScript lint because their
//! edge cases were earned there.

use super::Citation;
use regex::Regex;

/// Literal: `spec:<capability> § <name>`. The name runs to a double quote,
/// backtick or newline, so an apostrophe survives. Single-quoted citations
/// are therefore unsupported, which the convention already states.
const LITERAL: &str = r#"spec:([a-z0-9-]+)\s*§\s*([^"`\n]+)"#;

pub struct Grammar {
    literal: Regex,
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
        Grammar {
            literal: Regex::new(LITERAL).expect("literal grammar compiles"),
            call,
        }
    }

    pub fn literal(text: &str) -> Vec<Citation> {
        Grammar::new(None).citations(text)
    }

    pub fn citations(&self, text: &str) -> Vec<Citation> {
        let literal = self
            .literal
            .captures_iter(text)
            .map(|c| Citation::new(&c[1], &c[2]));
        let call = self.call.iter().flat_map(|re| {
            re.captures_iter(text).map(|c| {
                let name = c.get(2).or_else(|| c.get(3)).map_or("", |m| m.as_str());
                Citation::new(&c[1], name)
            })
        });
        literal.chain(call).collect()
    }

    /// Text with every literal citation removed, for searches that must not
    /// count formal citations as prose.
    pub fn strip(text: &str) -> String {
        Regex::new(LITERAL)
            .expect("literal grammar compiles")
            .replace_all(text, "")
            .into_owned()
    }
}
