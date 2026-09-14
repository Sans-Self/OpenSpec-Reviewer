//! Notes are edited in `$EDITOR`, falling back to `vi`.

use std::io;
use std::process::Command;

fn editor_command() -> Vec<String> {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .ok()
        .map(|e| e.split_whitespace().map(str::to_string).collect())
        .filter(|v: &Vec<String>| !v.is_empty())
        .unwrap_or_else(|| vec!["vi".to_string()])
}

/// Run `$EDITOR` on a temp file seeded with `current`; an empty file
/// removes the note. The caller suspends and resumes the terminal.
pub fn edit_note(current: Option<&str>) -> io::Result<Option<String>> {
    edit_note_with(&editor_command(), current)
}

pub fn edit_note_with(cmd: &[String], current: Option<&str>) -> io::Result<Option<String>> {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("openspec-reviewer-note-{}.md", std::process::id()));
    std::fs::write(&path, current.unwrap_or(""))?;
    let status = Command::new(&cmd[0]).args(&cmd[1..]).arg(&path).status();
    let text = std::fs::read_to_string(&path);
    let _ = std::fs::remove_file(&path);
    let status = status?;
    if !status.success() {
        return Err(io::Error::other(format!("editor exited with {status}")));
    }
    let text = text?;
    let trimmed = text.trim();
    Ok((!trimmed.is_empty()).then(|| trimmed.to_string()))
}
