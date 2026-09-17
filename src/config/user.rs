//! `$XDG_CONFIG_HOME/openspec-reviewer/config.toml`. Absent is the
//! defaults; malformed is an error naming the path, because a silent
//! fallback would hide a typo behind a default that looks like the tool
//! ignoring the user.

use crate::render::colour::PaletteChoice;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct UserConfig {
    pub palette: PaletteChoice,
}

#[derive(Debug, Error)]
pub enum UserConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {message}")]
    Parse { path: String, message: String },
}

/// `$XDG_CONFIG_HOME/openspec-reviewer/config.toml`, or the same under
/// `~/.config` when the variable is unset or relative.
pub fn config_path() -> Option<PathBuf> {
    let home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| directories::BaseDirs::new().map(|d| d.home_dir().join(".config")))?;
    Some(home.join("openspec-reviewer").join("config.toml"))
}

pub fn read_user_config() -> Result<UserConfig, UserConfigError> {
    match config_path() {
        Some(path) => read_user_config_at(&path),
        None => Ok(UserConfig::default()),
    }
}

pub fn read_user_config_at(path: &Path) -> Result<UserConfig, UserConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(UserConfig::default()),
        Err(source) => {
            return Err(UserConfigError::Read {
                path: path.display().to_string(),
                source,
            })
        }
    };
    parse(&text).map_err(|message| UserConfigError::Parse {
        path: path.display().to_string(),
        message,
    })
}

/// The toml error carries the offending line, so a wrong key or a wrong
/// value is named in the message without help from here.
pub fn parse(text: &str) -> Result<UserConfig, String> {
    toml::from_str(text).map_err(|e| e.to_string().trim_end().to_string())
}
