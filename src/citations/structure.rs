//! Change directories must be changes, and named inside the scopes.

/// A directory under `openspec/changes/` other than `archive/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeDir {
    pub name: String,
    /// `proposal.md` or `.openspec.yaml` is present.
    pub has_marker: bool,
}

/// `(change name, message)` pairs; the name is empty for the stale
/// grandfather entries, which point at the configuration instead.
pub fn check_changes(
    dirs: &[ChangeDir],
    scopes: Option<&[String]>,
    grandfathered: &[String],
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for dir in dirs {
        if !dir.has_marker {
            out.push((
                dir.name.clone(),
                format!(
                    "openspec/changes/{} is not a change directory: no proposal.md or .openspec.yaml; the OpenSpec CLI cannot address nested changes, group by name instead",
                    dir.name
                ),
            ));
            continue;
        }
        let Some(scopes) = scopes else { continue };
        if grandfathered.contains(&dir.name) {
            continue;
        }
        let in_scope = scopes.iter().any(|s| {
            dir.name
                .strip_prefix(s.as_str())
                .and_then(|rest| rest.strip_prefix('-'))
                .is_some_and(|rest| !rest.is_empty())
        });
        if !in_scope {
            out.push((
                dir.name.clone(),
                format!(
                    "openspec/changes/{}: change name does not start with a scope ({}) and a hyphen",
                    dir.name,
                    scopes.join(", ")
                ),
            ));
        }
    }
    for name in grandfathered {
        if !dirs.iter().any(|d| &d.name == name) {
            out.push((
                String::new(),
                format!(
                    "grandfathered change `{name}` is gone; remove it from `grandfathered` in openspec/reviewer.toml"
                ),
            ));
        }
    }
    out
}
