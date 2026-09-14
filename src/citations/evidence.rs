//! Evidence cited from canon specs: paths that must exist, regression tests
//! that must appear in source, hashes that must be commits.

use super::scan::{SourceFile, SpecFile};
use regex::Regex;
use std::collections::BTreeSet;

/// The two checks that need the repository: the filesystem and git.
pub trait Probe {
    fn path_exists(&self, rel: &str) -> bool;
    fn is_commit(&self, hash: &str) -> bool;
}

pub struct Checked {
    pub count: usize,
    pub missing: Vec<(std::path::PathBuf, String)>,
}

fn unique(iter: impl Iterator<Item = String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    iter.filter(|s| seen.insert(s.clone())).collect()
}

/// Every distinct `<prefix>/…​.<ext>` in a text. The character before the
/// match may not be one that could continue a path; `regex` has no
/// lookbehind, so the boundary is checked by hand. That is what lets a
/// prefix starting with `.` match at all.
pub fn cited_paths(text: &str, prefixes: &[String], extensions: &[String]) -> Vec<String> {
    if prefixes.is_empty() || extensions.is_empty() {
        return Vec::new();
    }
    let alt = |xs: &[String]| {
        xs.iter()
            .map(|x| regex::escape(x))
            .collect::<Vec<_>>()
            .join("|")
    };
    let re = Regex::new(&format!(
        r"(?:{})/[\w./-]+\.(?:{})",
        alt(prefixes),
        alt(extensions)
    ))
    .expect("path grammar compiles");
    let continues = |c: char| c.is_alphanumeric() || matches!(c, '_' | '.' | '/' | '-');
    unique(
        re.find_iter(text)
            .filter(|m| !text[..m.start()].chars().next_back().is_some_and(continues))
            .map(|m| m.as_str().to_string()),
    )
}

pub fn check_paths(
    specs: &[SpecFile],
    prefixes: &[String],
    extensions: &[String],
    probe: &dyn Probe,
) -> Checked {
    let mut checked = Checked {
        count: 0,
        missing: Vec::new(),
    };
    for spec in specs {
        for path in cited_paths(&spec.text, prefixes, extensions) {
            checked.count += 1;
            if !probe.path_exists(&path) {
                checked.missing.push((spec.path.clone(), path));
            }
        }
    }
    checked
}

/// Test names the pattern matches, looked up as fixed strings in the source
/// files the scan already read.
pub fn check_tests(specs: &[SpecFile], pattern: &Regex, sources: &[SourceFile]) -> Checked {
    let mut checked = Checked {
        count: 0,
        missing: Vec::new(),
    };
    for spec in specs {
        for name in unique(
            pattern
                .find_iter(&spec.text)
                .map(|m| m.as_str().to_string()),
        ) {
            checked.count += 1;
            if !sources.iter().any(|s| s.text.contains(&name)) {
                checked.missing.push((spec.path.clone(), name));
            }
        }
    }
    checked
}

pub fn check_hashes(specs: &[SpecFile], probe: &dyn Probe) -> Checked {
    let re = Regex::new("`([0-9a-f]{7,40})`").expect("hash grammar compiles");
    let mut checked = Checked {
        count: 0,
        missing: Vec::new(),
    };
    for spec in specs {
        for hash in unique(re.captures_iter(&spec.text).map(|c| c[1].to_string())) {
            checked.count += 1;
            if !probe.is_commit(&hash) {
                checked.missing.push((spec.path.clone(), hash));
            }
        }
    }
    checked
}
