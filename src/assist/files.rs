//! The edge of the assist module: templates on disk, the project's spec
//! rules, and the prompt files a run writes.

use super::AssistError;
use std::path::{Path, PathBuf};

pub const PROMPT_DIR: &str = "openspec/reviewer/prompts";
pub const CONFIG_YAML: &str = "openspec/config.yaml";

pub const PAIRING: &str = include_str!("prompts/pairing.md");
pub const CHANGE: &str = include_str!("prompts/change.md");
pub const HINTS: &str = include_str!("prompts/hints.md");

/// The three templates by the name they are overridden under.
pub const BUILT_IN: [(&str, &str); 3] = [
    ("pairing.md", PAIRING),
    ("change.md", CHANGE),
    ("hints.md", HINTS),
];

pub fn built_in(name: &str) -> Option<&'static str> {
    BUILT_IN
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, text)| *text)
}

/// The project's override of `name`, or the built-in. An unreadable
/// override is the built-in too: a prompt is never worth failing a review
/// over.
pub fn load_template(root: &Path, name: &str) -> String {
    let default = built_in(name).unwrap_or_default();
    std::fs::read_to_string(root.join(PROMPT_DIR).join(name))
        .ok()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// What `assist prompts` did, per file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    pub written: Vec<String>,
    pub skipped: Vec<String>,
}

/// Write the built-in templates under `openspec/reviewer/prompts/`,
/// leaving any file that is already there alone. `create_new` makes the
/// existence check and the write one operation.
pub fn export_prompts(root: &Path) -> Result<Export, std::io::Error> {
    let dir = root.join(PROMPT_DIR);
    std::fs::create_dir_all(&dir)?;
    let mut export = Export {
        written: Vec::new(),
        skipped: Vec::new(),
    };
    for (name, text) in BUILT_IN {
        let path = dir.join(name);
        let relative = format!("{PROMPT_DIR}/{name}");
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                std::io::Write::write_all(&mut file, text.as_bytes())?;
                export.written.push(relative);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                export.skipped.push(relative)
            }
            Err(e) => return Err(e),
        }
    }
    Ok(export)
}

/// `rules.specs` from `openspec/config.yaml`, each item as one string.
/// Missing file, missing key and unreadable file all give no rules; the
/// prompt then leaves the section out.
pub fn spec_rules(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join(CONFIG_YAML))
        .map(|text| parse_spec_rules(&text))
        .unwrap_or_default()
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// The list under `rules: specs:` as written. Reading two nested keys and
/// a list of scalars by hand keeps a YAML dependency out of the tool; the
/// file is OpenSpec's own and has no anchors, flow style or tags.
pub fn parse_spec_rules(yaml: &str) -> Vec<String> {
    let lines: Vec<&str> = yaml.lines().collect();
    let Some(rules) = lines
        .iter()
        .position(|l| l.trim_end() == "rules:" && indent_of(l) == 0)
    else {
        return Vec::new();
    };
    let body = &lines[rules + 1..];
    let end = body
        .iter()
        .position(|l| !l.trim().is_empty() && indent_of(l) == 0)
        .unwrap_or(body.len());
    let body = &body[..end];
    let Some(specs) = body
        .iter()
        .position(|l| l.trim_end().trim_start() == "specs:")
    else {
        return Vec::new();
    };
    let specs_indent = indent_of(body[specs]);
    let items = &body[specs + 1..];
    let end = items
        .iter()
        .position(|l| !l.trim().is_empty() && indent_of(l) <= specs_indent)
        .unwrap_or(items.len());

    let mut out: Vec<String> = Vec::new();
    for line in &items[..end] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match trimmed.strip_prefix("- ") {
            Some(first) => out.push(first.trim().to_string()),
            None => {
                if let Some(last) = out.last_mut() {
                    last.push(' ');
                    last.push_str(trimmed);
                }
            }
        }
    }
    out
}

/// `<state dir>/<change>/<slug>.md`: where a prompt file is written. The
/// state directory already exists per repository and change, so a prompt
/// lands next to the approvals it belongs with.
pub fn prompt_path(state_dir: &Path, change: &str, target: &str) -> PathBuf {
    state_dir
        .join(format!("{change}-prompts"))
        .join(format!("{}.md", slug(target)))
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

/// Write a prompt where the adapter can read it.
pub fn write_prompt(path: &Path, text: &str) -> Result<(), AssistError> {
    let write = || -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, text)
    };
    write().map_err(|source| AssistError::Prompt {
        path: path.to_path_buf(),
        source,
    })
}
