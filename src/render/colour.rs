//! Colour is additive. Every mark has a glyph; colour only layers on top.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub colour: bool,
}

/// `NO_COLOR` set to anything, or a dumb terminal, means no colour.
pub fn colour_enabled_from_env() -> bool {
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return false;
    }
    !std::env::var("TERM").is_ok_and(|t| t == "dumb")
}

impl Palette {
    pub fn from_env() -> Palette {
        Palette {
            colour: colour_enabled_from_env(),
        }
    }

    pub fn none() -> Palette {
        Palette { colour: false }
    }

    fn coloured(self, c: Color, fallback: Modifier) -> Style {
        if self.colour {
            Style::default().fg(c)
        } else {
            Style::default().add_modifier(fallback)
        }
    }

    pub fn added(self) -> Style {
        self.coloured(Color::Green, Modifier::BOLD)
    }

    pub fn removed(self) -> Style {
        self.coloured(Color::Red, Modifier::CROSSED_OUT)
    }

    pub fn changed(self) -> Style {
        self.coloured(Color::Yellow, Modifier::BOLD)
    }

    pub fn muted(self) -> Style {
        self.coloured(Color::DarkGray, Modifier::DIM)
    }

    pub fn heading(self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    pub fn selected(self) -> Style {
        if self.colour {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().add_modifier(Modifier::REVERSED)
        }
    }

    pub fn error(self) -> Style {
        self.coloured(Color::Red, Modifier::BOLD)
    }

    pub fn warning(self) -> Style {
        self.coloured(Color::Yellow, Modifier::BOLD)
    }
}

/// ANSI wrappers for `--color` in plain text.
pub mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
}
