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
        pull_request: None,
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

/// A stand-in for the `gh` CLI. It answers from files under `gh/` in the
/// directory it is run in, which is the repository the tool was pointed
/// at, and records the arguments and the request body there, so a test
/// reads back exactly what the tool asked for.
const GH_STUB: &str = r#"#!/bin/sh
dir=gh
mkdir -p "$dir"
printf '%s\n' "$*" >> "$dir/args"
case "$1 $2" in
  'pr view') cat "$dir/pr-view.json"; exit 0 ;;
  'pr diff') cat "$dir/pr.diff"; exit 0 ;;
esac
attempt=$(cat "$dir/attempts" 2>/dev/null || echo 0)
attempt=$((attempt + 1))
echo "$attempt" > "$dir/attempts"
cat > "$dir/request-$attempt.json"
if [ -f "$dir/fail" ]; then
  cat "$dir/fail" >&2
  exit 1
fi
if [ -f "$dir/reject-first" ] && [ "$attempt" -eq 1 ]; then
  echo 'gh: Validation Failed (HTTP 422)' >&2
  exit 1
fi
cat "$dir/response.json"
"#;

/// Put the stub in front of `PATH`, once per test process. The rest of
/// `PATH` stays, so `git` and the like still resolve.
pub fn gh_stub() {
    use std::sync::OnceLock;
    static STUB: OnceLock<()> = OnceLock::new();
    STUB.get_or_init(|| {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("openspec-reviewer-gh-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("gh");
        std::fs::write(&path, GH_STUB).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let rest = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{rest}", dir.display()));
    });
}

impl Repo {
    /// One of the files the stub `gh` answers from.
    pub fn gh(&self, name: &str, text: &str) -> &Repo {
        self.write(&format!("gh/{name}"), text)
    }

    /// The body of the nth request the stub received, as JSON.
    pub fn gh_request(&self, nth: usize) -> serde_json::Value {
        let path = self.root().join(format!("gh/request-{nth}.json"));
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        serde_json::from_str(&text).expect("the request is JSON")
    }

    pub fn gh_requests(&self) -> usize {
        std::fs::read_to_string(self.root().join("gh/attempts"))
            .map(|t| t.trim().parse().unwrap_or(0))
            .unwrap_or(0)
    }
}

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

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

impl Repo {
    /// A fixture directory copied into a fresh repo and committed on `main`.
    pub fn from_fixture(name: &str) -> Repo {
        let repo = Repo::new();
        copy_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name),
            repo.root(),
        );
        repo.init_git("main");
        repo
    }

    pub fn remove(&self, rel: &str) -> &Repo {
        let path = self.root().join(rel);
        if path.is_dir() {
            std::fs::remove_dir_all(path).unwrap();
        } else {
            std::fs::remove_file(path).unwrap();
        }
        self
    }

    pub fn append(&self, rel: &str, text: &str) -> &Repo {
        let path = self.root().join(rel);
        let mut current = std::fs::read_to_string(&path).unwrap_or_default();
        current.push_str(text);
        std::fs::write(path, current).unwrap();
        self
    }

    pub fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"]).trim().to_string()
    }
}
