//! One parser for every agent's reply. Pure: text in, hints out.

use super::{Hint, HintKind};
use serde::Deserialize;

/// A reply element before its kind is checked. An unknown kind is kept
/// rather than dropped, so a prompt that stops working looks broken.
#[derive(Deserialize)]
struct RawHint {
    kind: String,
    message: String,
    #[serde(default)]
    quote: Option<String>,
    #[serde(default)]
    scenario: Option<String>,
}

/// The JSON array in a reply, which agents like to wrap in prose or a
/// fenced block. The outermost bracket pair is the array.
fn array_slice(reply: &str) -> Option<&str> {
    let start = reply.find('[')?;
    let end = reply.rfind(']')?;
    (end > start).then(|| &reply[start..=end])
}

fn first_line(reply: &str) -> String {
    reply
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("empty reply")
        .to_string()
}

fn unparsed(reply: &str) -> Vec<Hint> {
    vec![Hint::new(HintKind::Unparsed, first_line(reply))]
}

/// The reply as hints. A reply that holds no JSON array, or an array that
/// does not deserialize, becomes one `unparsed` hint carrying the first
/// line of the reply.
pub fn parse_hints(reply: &str) -> Vec<Hint> {
    let Some(slice) = array_slice(reply) else {
        return unparsed(reply);
    };
    let Ok(raw) = serde_json::from_str::<Vec<RawHint>>(slice) else {
        return unparsed(reply);
    };
    raw.into_iter()
        .map(|r| match HintKind::parse(&r.kind) {
            Some(kind) => Hint {
                kind,
                message: r.message,
                quote: r.quote,
                scenario: r.scenario,
                dismissed: false,
            },
            None => Hint::new(
                HintKind::Unparsed,
                format!("unknown hint kind `{}`: {}", r.kind, r.message),
            ),
        })
        .collect()
}
