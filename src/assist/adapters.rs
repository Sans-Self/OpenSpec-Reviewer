//! One adapter per agent CLI. The difference between them is flags, so
//! everything else — reading the prompt, running the process, turning a
//! non-zero exit into an error — lives in the helpers at the bottom.

use super::{Agent, Assist, AssistError, Assistant};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// `claude`: the prompt as an argument interactively, on stdin for a
/// review, whose JSON envelope carries the reply under `result`.
pub struct Claude {
    pub program: PathBuf,
}

impl Assistant for Claude {
    fn handoff(&self, prompt: &Path) -> Result<(), AssistError> {
        let text = read_prompt(prompt)?;
        let mut command = Command::new(&self.program);
        command.arg(text);
        interactive(command, "claude")
    }

    fn review(&self, prompt: &Path) -> Result<String, AssistError> {
        let text = read_prompt(prompt)?;
        let mut command = Command::new(&self.program);
        command.args(["-p", "--output-format", "json"]);
        let out = captured(command, "claude", Some(&text))?;
        Ok(unwrap_result_field(&out))
    }
}

/// `codex`: `exec --json` for a review, the prompt as an argument for a
/// session.
pub struct Codex {
    pub program: PathBuf,
}

impl Assistant for Codex {
    fn handoff(&self, prompt: &Path) -> Result<(), AssistError> {
        let text = read_prompt(prompt)?;
        let mut command = Command::new(&self.program);
        command.arg(text);
        interactive(command, "codex")
    }

    fn review(&self, prompt: &Path) -> Result<String, AssistError> {
        let text = read_prompt(prompt)?;
        let mut command = Command::new(&self.program);
        command.args(["exec", "--json"]);
        captured(command, "codex", Some(&text))
    }
}

/// `opencode run`, which takes the prompt as an argument either way.
pub struct Opencode {
    pub program: PathBuf,
}

impl Assistant for Opencode {
    fn handoff(&self, prompt: &Path) -> Result<(), AssistError> {
        let text = read_prompt(prompt)?;
        let mut command = Command::new(&self.program);
        command.args(["run", &text]);
        interactive(command, "opencode")
    }

    fn review(&self, prompt: &Path) -> Result<String, AssistError> {
        let text = read_prompt(prompt)?;
        let mut command = Command::new(&self.program);
        command.args(["run", "--format", "json", &text]);
        captured(command, "opencode", None)
    }
}

/// Two shell templates in which `{prompt}` is the prompt file's path.
pub struct Custom {
    pub shell: PathBuf,
    pub handoff_command: Option<String>,
    pub review_command: Option<String>,
}

impl Custom {
    fn command(&self, template: &str, prompt: &Path) -> Command {
        let mut command = Command::new(&self.shell);
        command.arg("-c");
        command.arg(template.replace("{prompt}", &prompt.display().to_string()));
        command
    }
}

impl Assistant for Custom {
    fn handoff(&self, prompt: &Path) -> Result<(), AssistError> {
        let template = self
            .handoff_command
            .as_deref()
            .ok_or(AssistError::NoCommand {
                field: "handoff_command",
            })?;
        interactive(self.command(template, prompt), "sh")
    }

    fn review(&self, prompt: &Path) -> Result<String, AssistError> {
        let template = self
            .review_command
            .as_deref()
            .ok_or(AssistError::NoCommand {
                field: "review_command",
            })?;
        captured(self.command(template, prompt), "sh", None)
    }
}

/// The adapter for a configuration, or the message that says why there is
/// none. `path` is the search path the binary is looked up in.
pub fn assistant(
    assist: Option<&Assist>,
    path: Option<&OsString>,
) -> Result<Box<dyn Assistant>, AssistError> {
    let assist = assist.ok_or(AssistError::NotConfigured)?;
    let binary = assist.agent.binary();
    let program = super::which(binary, path).ok_or_else(|| AssistError::NotFound {
        binary: binary.to_string(),
    })?;
    Ok(match assist.agent {
        Agent::Claude => Box::new(Claude { program }),
        Agent::Codex => Box::new(Codex { program }),
        Agent::Opencode => Box::new(Opencode { program }),
        Agent::Custom => Box::new(Custom {
            shell: program,
            handoff_command: assist.handoff_command.clone(),
            review_command: assist.review_command.clone(),
        }),
    })
}

fn read_prompt(prompt: &Path) -> Result<String, AssistError> {
    std::fs::read_to_string(prompt).map_err(|source| AssistError::Prompt {
        path: prompt.to_path_buf(),
        source,
    })
}

/// Hand the terminal over and wait. The caller has already left the
/// alternate screen.
fn interactive(mut command: Command, binary: &str) -> Result<(), AssistError> {
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|source| spawn_error(binary, source))?;
    if status.success() {
        return Ok(());
    }
    Err(AssistError::Failed {
        binary: binary.to_string(),
        code: status.code().unwrap_or(-1),
        stderr: String::new(),
    })
}

fn captured(
    mut command: Command,
    binary: &str,
    stdin_text: Option<&str>,
) -> Result<String, AssistError> {
    command
        .stdin(match stdin_text {
            Some(_) => Stdio::piped(),
            None => Stdio::null(),
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|source| spawn_error(binary, source))?;
    if let (Some(text), Some(mut stdin)) = (stdin_text, child.stdin.take()) {
        stdin
            .write_all(text.as_bytes())
            .map_err(|source| spawn_error(binary, source))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|source| spawn_error(binary, source))?;
    if !out.status.success() {
        return Err(AssistError::Failed {
            binary: binary.to_string(),
            code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn spawn_error(binary: &str, source: std::io::Error) -> AssistError {
    match source.kind() {
        std::io::ErrorKind::NotFound => AssistError::NotFound {
            binary: binary.to_string(),
        },
        _ => AssistError::Spawn {
            binary: binary.to_string(),
            source,
        },
    }
}

/// `claude -p --output-format json` wraps the reply in an envelope. An
/// envelope whose `result` is a string is unwrapped; anything else is
/// passed through for the hint parser to make what it can of.
fn unwrap_result_field(out: &str) -> String {
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(out) else {
        return out.to_string();
    };
    match map.get("result") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => out.to_string(),
    }
}
