//! Canon is always `openspec/specs/` in the working directory.

use crate::model::{parse_canon_spec, Canon};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonError {
    #[error("found no OpenSpec canon here: {0} does not exist")]
    NotFound(PathBuf),
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub fn load_canon(root: &Path) -> Result<Canon, CanonError> {
    let specs = root.join("openspec").join("specs");
    if !specs.is_dir() {
        return Err(CanonError::NotFound(specs));
    }
    let read = |path: &Path| -> Result<_, CanonError> {
        std::fs::read_dir(path).map_err(|source| CanonError::Io {
            path: path.to_path_buf(),
            source,
        })
    };
    let mut canon = Canon::default();
    for entry in read(&specs)? {
        let entry = entry.map_err(|source| CanonError::Io {
            path: specs.clone(),
            source,
        })?;
        let file = entry.path().join("spec.md");
        if !file.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&file).map_err(|source| CanonError::Io {
            path: file.clone(),
            source,
        })?;
        let capability = entry.file_name().to_string_lossy().into_owned();
        canon.specs.insert(capability, parse_canon_spec(&text));
    }
    Ok(canon)
}
