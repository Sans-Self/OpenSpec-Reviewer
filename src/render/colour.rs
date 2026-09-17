//! Colour is additive. Every mark has a glyph; colour only layers on top.
//! One table answers "which style for this role" for the view, the plain
//! text and the lint, so the three agree and `none` is one column away.

use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaletteChoice {
    #[default]
    Default,
    /// Blue for added and orange for removed: the pair that survives the
    /// common colour-vision deficiencies.
    Accessible,
    /// Modifiers only, as a standing `NO_COLOR`.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Background {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    choice: PaletteChoice,
    background: Background,
}

/// Where a role's colour goes. Tints paint the background and leave the
/// foreground to the terminal, because a red foreground on a red tint is
/// the first thing a light theme breaks.
#[derive(Clone, Copy)]
enum Paint {
    Fg,
    Bg,
}

/// `NO_COLOR` set to anything, or a dumb terminal, means no colour.
pub fn colour_enabled_from_env() -> bool {
    colour_enabled(
        std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()),
        std::env::var("TERM").ok().as_deref(),
    )
}

fn colour_enabled(no_color: bool, term: Option<&str>) -> bool {
    !no_color && term != Some("dumb")
}

impl PaletteChoice {
    /// The palette that survives the environment. Top wins: `NO_COLOR`,
    /// then `none` itself, then a stdout that is not a terminal unless
    /// `--color` forces it, then `TERM=dumb`; what is left is the choice.
    pub fn resolve(self, stdout_is_terminal: bool, force: bool) -> PaletteChoice {
        self.resolve_with(
            stdout_is_terminal,
            force,
            std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()),
            std::env::var("TERM").ok().as_deref(),
        )
    }

    pub fn resolve_with(
        self,
        stdout_is_terminal: bool,
        force: bool,
        no_color: bool,
        term: Option<&str>,
    ) -> PaletteChoice {
        if no_color || self == PaletteChoice::None {
            return PaletteChoice::None;
        }
        if !stdout_is_terminal && !force {
            return PaletteChoice::None;
        }
        if term == Some("dumb") {
            return PaletteChoice::None;
        }
        self
    }
}

impl Palette {
    pub fn new(choice: PaletteChoice, background: Background) -> Palette {
        Palette { choice, background }
    }

    pub fn none() -> Palette {
        Palette::new(PaletteChoice::None, Background::Dark)
    }

    /// What the environment alone allows: the default palette on a dark
    /// background, or none. The user file and the background probe refine
    /// it before the view opens.
    pub fn from_env() -> Palette {
        if colour_enabled_from_env() {
            Palette::new(PaletteChoice::Default, Background::Dark)
        } else {
            Palette::none()
        }
    }

    pub fn colour(self) -> bool {
        self.choice != PaletteChoice::None
    }

    pub fn choice(self) -> PaletteChoice {
        self.choice
    }

    pub fn background(self) -> Background {
        self.background
    }

    /// Every role carries its modifier in every palette, so no role is
    /// colour only.
    fn role(self, paint: Paint, dark: Color, light: Color, modifier: Modifier) -> Style {
        let style = Style::default().add_modifier(modifier);
        if !self.colour() {
            return style;
        }
        let colour = match self.background {
            Background::Dark => dark,
            Background::Light => light,
        };
        match paint {
            Paint::Fg => style.fg(colour),
            Paint::Bg => style.bg(colour),
        }
    }

    fn added_colour(self) -> Color {
        match self.choice {
            PaletteChoice::Accessible => Color::Blue,
            _ => Color::Green,
        }
    }

    /// Orange has no ANSI 16 slot: yellow stands in on dark, magenta on
    /// light, where yellow is unreadable.
    fn removed_colour(self) -> Color {
        match (self.choice, self.background) {
            (PaletteChoice::Accessible, Background::Dark) => Color::Yellow,
            (PaletteChoice::Accessible, Background::Light) => Color::Magenta,
            _ => Color::Red,
        }
    }

    /// The glyph and marker colour of an addition.
    pub fn added(self) -> Style {
        let c = self.added_colour();
        self.role(Paint::Fg, c, c, Modifier::BOLD)
    }

    pub fn removed(self) -> Style {
        let c = self.removed_colour();
        self.role(Paint::Fg, c, c, Modifier::CROSSED_OUT)
    }

    pub fn changed(self) -> Style {
        self.role(Paint::Fg, Color::Yellow, Color::Magenta, Modifier::BOLD)
    }

    /// The tint of an added line or span.
    pub fn added_span(self) -> Style {
        let c = self.added_colour();
        self.role(Paint::Bg, c, c, Modifier::BOLD)
    }

    pub fn removed_span(self) -> Style {
        let c = self.removed_colour();
        self.role(Paint::Bg, c, c, Modifier::CROSSED_OUT)
    }

    pub fn error(self) -> Style {
        self.role(Paint::Fg, Color::Red, Color::Red, Modifier::BOLD)
    }

    pub fn warning(self) -> Style {
        self.role(Paint::Fg, Color::Yellow, Color::Magenta, Modifier::BOLD)
    }

    pub fn note(self) -> Style {
        self.role(Paint::Fg, Color::Cyan, Color::Blue, Modifier::ITALIC)
    }

    pub fn approved(self) -> Style {
        self.role(Paint::Fg, Color::Green, Color::Green, Modifier::BOLD)
    }

    pub fn stale(self) -> Style {
        self.role(Paint::Fg, Color::Yellow, Color::Magenta, Modifier::BOLD)
    }

    pub fn pending(self) -> Style {
        self.role(Paint::Fg, Color::DarkGray, Color::Gray, Modifier::DIM)
    }

    pub fn accent(self) -> Style {
        self.role(Paint::Fg, Color::Cyan, Color::Blue, Modifier::BOLD)
    }

    pub fn muted(self) -> Style {
        self.role(Paint::Fg, Color::DarkGray, Color::Gray, Modifier::DIM)
    }

    pub fn heading(self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    /// An admitted glossary name. Underline is orthogonal to every tint
    /// and every diff modifier, so it composes instead of replacing.
    pub fn term(self) -> Style {
        Style::default().add_modifier(Modifier::UNDERLINED)
    }

    pub fn synonym(self) -> Style {
        self.warning().add_modifier(Modifier::UNDERLINED)
    }

    pub fn focus(self) -> Style {
        self.role(Paint::Fg, Color::Cyan, Color::Blue, Modifier::BOLD)
    }

    pub fn unfocus(self) -> Style {
        self.role(Paint::Fg, Color::DarkGray, Color::Gray, Modifier::DIM)
    }

    /// The cursor row. It sets no foreground, so the marks on the row keep
    /// their own colours under it.
    pub fn selected(self) -> Style {
        if self.colour() {
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::REVERSED)
        }
    }
}

/// ANSI escape codes for plain text on a terminal, from the same styles
/// the view draws with.
pub mod ansi {
    use ratatui::style::{Color, Modifier, Style};

    pub const RESET: &str = "\x1b[0m";

    fn code(c: Color, background: bool) -> Option<u8> {
        let base = match c {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::Gray => 37,
            Color::DarkGray => 90,
            Color::LightRed => 91,
            Color::LightGreen => 92,
            Color::LightYellow => 93,
            Color::LightBlue => 94,
            Color::LightMagenta => 95,
            Color::LightCyan => 96,
            Color::White => 97,
            _ => return None,
        };
        Some(if background { base + 10 } else { base })
    }

    /// The SGR sequence that turns `style` on, empty for the default style.
    pub fn sgr(style: Style) -> String {
        let mut codes: Vec<String> = Vec::new();
        let m = style.add_modifier;
        for (flag, n) in [
            (Modifier::BOLD, 1),
            (Modifier::DIM, 2),
            (Modifier::ITALIC, 3),
            (Modifier::UNDERLINED, 4),
            (Modifier::REVERSED, 7),
            (Modifier::CROSSED_OUT, 9),
        ] {
            if m.contains(flag) {
                codes.push(n.to_string());
            }
        }
        if let Some(n) = style.fg.and_then(|c| code(c, false)) {
            codes.push(n.to_string());
        }
        if let Some(n) = style.bg.and_then(|c| code(c, true)) {
            codes.push(n.to_string());
        }
        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// `text` wrapped in `style`'s codes when the palette has colour, and
    /// bare otherwise: plain text without colour has no escape codes at all.
    pub fn paint(palette: super::Palette, style: Style, text: &str) -> String {
        let codes = sgr(style);
        if palette.colour() && !text.is_empty() && !codes.is_empty() {
            format!("{codes}{text}{RESET}")
        } else {
            text.to_string()
        }
    }
}
