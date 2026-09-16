//! Reading and writing the skill files `skills` plans.

use crate::skills::{
    plan, FileState, Layout, Planned, SkillsError, HOME_MARKER, OVERRIDES, SKILLS,
};
use std::collections::BTreeMap;
use std::path::Path;

pub fn layout(root: &Path) -> Layout {
    let (marker_path, marker_text) = HOME_MARKER;
    Layout {
        claude: root.join(".claude").is_dir(),
        agents: root.join(".agents").is_dir(),
        home: std::fs::read_to_string(root.join(marker_path))
            .is_ok_and(|t| t.contains(marker_text)),
    }
}

fn read_if_present(root: &Path, rel: &str) -> Result<Option<String>, SkillsError> {
    match std::fs::read_to_string(root.join(rel)) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(SkillsError::Read {
            path: rel.to_string(),
            source,
        }),
    }
}

fn overrides(root: &Path) -> Result<BTreeMap<String, String>, SkillsError> {
    let mut out = BTreeMap::new();
    for skill in SKILLS {
        if let Some(text) = read_if_present(root, &format!("{OVERRIDES}/{}.md", skill.name))? {
            out.insert(skill.name.to_string(), text);
        }
    }
    Ok(out)
}

/// The plan for this repository, with what is on disk at every path.
pub fn planned(root: &Path) -> Result<Vec<Planned>, SkillsError> {
    let layout = layout(root);
    let mut existing = BTreeMap::new();
    for (_, _, path) in crate::skills::paths(layout) {
        if let Some(text) = read_if_present(root, &path)? {
            existing.insert(path, text);
        }
    }
    plan(layout, &existing, &overrides(root)?)
}

/// Write every planned file that is not hand-edited; return the plan so
/// the caller can say what happened to each.
pub fn install(root: &Path) -> Result<Vec<Planned>, SkillsError> {
    let planned = planned(root)?;
    for p in planned.iter().filter(|p| p.writes()) {
        let path = root.join(&p.path);
        let write = |source| SkillsError::Write {
            path: p.path.clone(),
            source,
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(write)?;
        }
        std::fs::write(&path, &p.content).map_err(write)?;
    }
    Ok(planned)
}

/// Like `planned`, but a repository without `.claude/` lists everything
/// as not installed instead of refusing.
pub fn listed(root: &Path) -> Result<Vec<Planned>, SkillsError> {
    match planned(root) {
        Err(SkillsError::NoClaude) => {
            let layout = Layout {
                claude: true,
                ..layout(root)
            };
            let mut all = plan(layout, &BTreeMap::new(), &overrides(root)?)?;
            for p in &mut all {
                p.state = FileState::NotInstalled;
            }
            Ok(all)
        }
        other => other,
    }
}
