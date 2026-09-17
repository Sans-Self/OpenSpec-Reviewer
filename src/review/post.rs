//! From the unposted notes to one pull request review. Pure: the review,
//! the stores and nothing else; the request itself leaves from `source`.

use super::{Review, OUTDATED_NOTE};
use crate::state::Store;
use std::collections::BTreeMap;

/// The body when every note found a line. GitHub rejects a review with
/// neither body nor comments, so a bare line is the cheapest body that
/// always passes.
pub const TOOL_LINE: &str = "Notes from openspec-reviewer.";

/// A note on the line its anchor's heading stands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub path: String,
    /// One-based, as GitHub counts lines.
    pub line: usize,
    /// What the body section would be headed, kept for the retry.
    pub heading: String,
    pub body: String,
}

/// A note the body carries, because its anchor has no line to hang on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub body: String,
}

/// One review, ready to send, and the notes it stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewPost {
    pub comments: Vec<Comment>,
    pub sections: Vec<Section>,
    /// The change and the item key of every note in this post, in the order
    /// the review lists them: what a successful post stamps.
    pub keys: Vec<(String, String)>,
}

impl ReviewPost {
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The review's body: the notes that found no line, or one line naming
    /// the tool when every note is a comment.
    pub fn body(&self) -> String {
        if self.sections.is_empty() {
            return TOOL_LINE.to_string();
        }
        self.sections
            .iter()
            .map(|s| format!("### {}\n\n{}", s.heading, s.body))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// GitHub's `POST /repos/{owner}/{repo}/pulls/{n}/reviews` shape.
    pub fn payload(&self, head: &str) -> serde_json::Value {
        let mut payload = serde_json::json!({
            "commit_id": head,
            "event": "COMMENT",
            "body": self.body(),
        });
        if !self.comments.is_empty() {
            let comments: Vec<serde_json::Value> = self
                .comments
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "path": c.path,
                        "line": c.line,
                        "body": c.body,
                    })
                })
                .collect();
            payload["comments"] = serde_json::Value::Array(comments);
        }
        payload
    }

    /// The same notes with every comment moved into the body, for the one
    /// retry a rejected review gets.
    pub fn into_body_only(self) -> ReviewPost {
        let mut sections: Vec<Section> = self
            .comments
            .into_iter()
            .map(|c| Section {
                heading: c.heading,
                body: c.body,
            })
            .collect();
        sections.extend(self.sections);
        ReviewPost {
            comments: Vec::new(),
            sections,
            keys: self.keys,
        }
    }
}

/// The unposted notes of a review, each anchored to a line where its
/// heading is there to be found. `None` when there is nothing to post.
pub fn plan(review: &Review, stores: &BTreeMap<String, Store>) -> Option<ReviewPost> {
    let mut post = ReviewPost {
        comments: Vec::new(),
        sections: Vec::new(),
        keys: Vec::new(),
    };
    for change in &review.changes {
        let Some(store) = stores.get(&change.name) else {
            continue;
        };
        for artefact in &change.artefacts {
            let key = artefact.key();
            let Some(note) = unposted(store, &key) else {
                continue;
            };
            let path = format!(
                "openspec/changes/{}/{}",
                change.name, artefact.artefact.name
            );
            let body = body_of(&note, artefact.note_outdated());
            // An artefact is one file: the first line is the only anchor it
            // has, and every reader of the comment sees the file anyway.
            let line = review.after_text(&path).map(|_| 1);
            post.push(
                change.name.clone(),
                key,
                artefact.artefact.name.clone(),
                path,
                line,
                body,
            );
        }
        for capability in &change.capabilities {
            let path = format!(
                "openspec/changes/{}/specs/{}/spec.md",
                change.name, capability.name
            );
            let text = review.after_text(&path);
            for pairing in &capability.pairings {
                for note in &pairing.notes {
                    let key = match &note.scenario {
                        None => pairing.key(),
                        Some(s) => pairing.scenario_key(s),
                    };
                    let Some(stored) = unposted(store, &key) else {
                        continue;
                    };
                    let heading = format!(
                        "{} § {}{}",
                        pairing.capability,
                        pairing.name,
                        note.anchor_suffix()
                    );
                    let line =
                        text.and_then(|t| heading_line(t, &pairing.name, note.scenario.as_deref()));
                    let body = body_of(&stored, note.outdated);
                    post.push(change.name.clone(), key, heading, path.clone(), line, body);
                }
            }
        }
    }
    (!post.is_empty()).then_some(post)
}

impl ReviewPost {
    /// One note into the post: a comment when its anchor has a line, a body
    /// section when it has not, and its key either way.
    fn push(
        &mut self,
        change: String,
        key: String,
        heading: String,
        path: String,
        line: Option<usize>,
        body: String,
    ) {
        match line {
            Some(line) => self.comments.push(Comment {
                path,
                line,
                heading,
                body,
            }),
            None => self.sections.push(Section { heading, body }),
        }
        self.keys.push((change, key));
    }
}

fn unposted(store: &Store, key: &str) -> Option<crate::state::Note> {
    store.get(key).note.filter(|note| note.posted.is_none())
}

fn body_of(note: &crate::state::Note, outdated: bool) -> String {
    match outdated {
        false => note.text.clone(),
        true => format!("{}\n\n{OUTDATED_NOTE}", note.text),
    }
}

/// The one-based line of a requirement's heading in a delta spec, or of one
/// of its scenarios' headings under it. A scenario is looked for between
/// its requirement's heading and the next requirement, so two requirements
/// that share a scenario name anchor to their own.
fn heading_line(text: &str, requirement: &str, scenario: Option<&str>) -> Option<usize> {
    let requirement_heading = format!("### Requirement: {requirement}");
    let start = text
        .lines()
        .position(|l| l.trim_end() == requirement_heading)?;
    let Some(scenario) = scenario else {
        return Some(start + 1);
    };
    let scenario_heading = format!("#### Scenario: {scenario}");
    text.lines()
        .enumerate()
        .skip(start + 1)
        .take_while(|(_, l)| !l.starts_with("### ") && !l.starts_with("## "))
        .find(|(_, l)| l.trim_end() == scenario_heading)
        .map(|(i, _)| i + 1)
}
