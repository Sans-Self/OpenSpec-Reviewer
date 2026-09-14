//! `git <ref> [--base <ref>]`: two refs, whole files, no patching.

use super::{FileChange, Snapshot, Source, SourceError};
use std::path::PathBuf;
use std::process::Command;

pub struct GitSource {
    pub root: PathBuf,
    pub reference: String,
    pub base: Option<String>,
}

impl GitSource {
    fn git(&self, args: &[&str]) -> Result<String, SourceError> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()
            .map_err(SourceError::io("running git"))?;
        if !out.status.success() {
            return Err(SourceError::Git {
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    fn resolves(&self, reference: &str) -> bool {
        self.git(&["rev-parse", "--verify", "--quiet", reference])
            .is_ok()
    }

    fn base(&self) -> Result<String, SourceError> {
        if let Some(base) = &self.base {
            return Ok(base.clone());
        }
        ["main", "master"]
            .into_iter()
            .find(|b| self.resolves(b))
            .map(str::to_string)
            .ok_or(SourceError::NoBase)
    }

    fn show(&self, reference: &str, path: &str) -> Result<String, SourceError> {
        self.git(&["show", &format!("{reference}:{path}")])
    }
}

impl Source for GitSource {
    fn fetch(&self) -> Result<Snapshot, SourceError> {
        let base = self.base()?;
        let range = format!("{base}...{}", self.reference);
        let listing = self.git(&["diff", "--name-status", &range, "--", "openspec/"])?;
        let files = listing
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                let mut parts = line.split('\t');
                let status = parts.next().unwrap_or("");
                let first = parts.next().unwrap_or("");
                let second = parts.next();
                let (old_path, new_path) = match status.chars().next() {
                    Some('R') | Some('C') => (first, second.unwrap_or(first)),
                    _ => (first, first),
                };
                let before = match status.chars().next() {
                    Some('A') => None,
                    _ => Some(self.show(&base, old_path)?),
                };
                let after = match status.chars().next() {
                    Some('D') => None,
                    _ => Some(self.show(&self.reference, new_path)?),
                };
                Ok(FileChange {
                    path: PathBuf::from(new_path),
                    before,
                    after,
                })
            })
            .collect::<Result<Vec<_>, SourceError>>()?;
        Ok(Snapshot {
            files,
            origin: format!("git {} against {base}", self.reference),
        })
    }
}
