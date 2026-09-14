#![allow(dead_code)]

use openspec_reviewer::model::{
    parse_canon_spec, parse_delta_spec, Canon, Change, DeltaSpec, Requirement,
};
use openspec_reviewer::review::{pair_change, ChangeReview};
use openspec_reviewer::source::{FileChange, Snapshot};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn fixture(name: &str, file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

pub fn canon_of(capability: &str, text: &str) -> Canon {
    let mut canon = Canon::default();
    canon
        .specs
        .insert(capability.to_string(), parse_canon_spec(text));
    canon
}

pub fn delta_of(capability: &str, text: &str) -> DeltaSpec {
    parse_delta_spec(capability, &format!("{capability}/spec.md"), text)
        .expect("fixture delta parses")
        .join_renames()
}

pub fn change_of(name: &str, deltas: Vec<DeltaSpec>) -> Change {
    Change {
        name: name.to_string(),
        artefacts: Vec::new(),
        deltas,
    }
}

/// Pair one fixture's delta against its canon under `capability`.
pub fn review_fixture(fixture_name: &str, capability: &str) -> ChangeReview {
    let canon = canon_of(capability, &fixture(fixture_name, "canon.md"));
    let delta = delta_of(capability, &fixture(fixture_name, "delta.md"));
    pair_change(&change_of(fixture_name, vec![delta]), &canon, &[])
}

pub fn req(name: &str, body: &str, scenarios: &[(&str, &str)]) -> Requirement {
    Requirement {
        name: name.to_string(),
        body: body.to_string(),
        scenarios: scenarios
            .iter()
            .map(|(n, b)| openspec_reviewer::model::Scenario {
                name: n.to_string(),
                body: b.to_string(),
            })
            .collect(),
    }
}

pub fn snapshot(files: Vec<(&str, Option<&str>, Option<&str>)>) -> Snapshot {
    Snapshot {
        files: files
            .into_iter()
            .map(|(p, b, a)| FileChange {
                path: PathBuf::from(p),
                before: b.map(str::to_string),
                after: a.map(str::to_string),
            })
            .collect(),
        origin: "test".to_string(),
    }
}

/// A throwaway repository with an `openspec/` tree.
pub struct Repo {
    pub dir: tempfile::TempDir,
}

impl Repo {
    pub fn new() -> Repo {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("openspec/specs")).unwrap();
        std::fs::create_dir_all(dir.path().join("openspec/changes")).unwrap();
        Repo { dir }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// A state home inside the repo, so tests never touch the real one.
    pub fn state_home(&self) -> PathBuf {
        self.root().join(".state")
    }

    pub fn write(&self, rel: &str, text: &str) -> &Repo {
        let path = self.root().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
        self
    }

    pub fn canon(&self, capability: &str, text: &str) -> &Repo {
        self.write(&format!("openspec/specs/{capability}/spec.md"), text)
    }

    pub fn delta(&self, change: &str, capability: &str, text: &str) -> &Repo {
        self.write(
            &format!("openspec/changes/{change}/specs/{capability}/spec.md"),
            text,
        )
    }

    pub fn archive(&self, archive: &str, capability: &str, text: &str) -> &Repo {
        self.write(
            &format!("openspec/changes/archive/{archive}/specs/{capability}/spec.md"),
            text,
        )
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(self.root())
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// `git init` with the given default branch and one commit of the
    /// current tree.
    pub fn init_git(&self, branch: &str) -> &Repo {
        self.git(&["init", "-q", "-b", branch]);
        self.git(&["config", "commit.gpgsign", "false"]);
        self.commit("initial");
        self
    }

    pub fn commit(&self, message: &str) -> &Repo {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
        self
    }
}

pub const ALPHA_CANON: &str = "\
# alpha

## Requirements

### Requirement: Flat index of all routes and pages

The dashboard MUST offer an index view listing every route and every
page of the active website.

#### Scenario: Route-mounted page appears as a row

- **WHEN** a route mounts a page
- **THEN** the index shows one row

#### Scenario: Feature mount appears

- **WHEN** a route has a feature mount
- **THEN** the row shows the feature name

#### Scenario: Orphan page appears without a path

- **WHEN** a page is not mounted by any route
- **THEN** the page appears as a row without a path

### Requirement: Index rows are ordered by path

Rows MUST be ordered alphabetically by path.

#### Scenario: Paths sort alphabetically

- **WHEN** the index renders routes
- **THEN** the rows appear in path order
";

pub fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_openspec-reviewer"))
}

pub fn run_in(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(exe())
        .args(args)
        .current_dir(root)
        .env("XDG_STATE_HOME", root.join(".state"))
        .env_remove("NO_COLOR")
        .output()
        .expect("binary runs")
}

pub fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}
