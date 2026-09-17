//! Drawing. Every mark carries a glyph; the palette only adds colour.

use super::app::{
    history_entries, App, DetailMode, Focus, HistoryMode, Modal, NoteEdit, Pane, Row, Transient,
    BINDINGS,
};
use super::tree;
use crate::glossary::{Mark, Marks, Occurrence};
use crate::render::colour::Palette;
use crate::render::{approval, finding_marker, history_summary, note_marker};
use crate::review::{AnchoredNote, DiffLine, LineRole, ParaKind, Severity, SpanMark};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let palette = Palette::from_env();
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(vertical[0]);
    app.detail_height = panes[1].height.saturating_sub(2);
    app.refresh_detail();

    if app.history_state().is_some() {
        draw_history(frame, app, panes[0], panes[1], palette);
    } else {
        draw_list(frame, app, panes[0], palette);
        draw_detail(frame, app, panes[1], palette);
    }
    draw_status(frame, app, vertical[1], palette);
    match &app.focus {
        Focus::Transient(Transient::Help) => draw_help(frame, area, palette),
        Focus::Modal(Modal::Note(edit)) => draw_note(frame, edit, panes[1], palette),
        _ => {}
    }
}

fn row_line(app: &App, row: &Row, prefix: &str, palette: Palette) -> Line<'static> {
    let guide = Span::styled(prefix.to_string(), palette.muted());
    match row {
        Row::Change { change } => Line::styled(
            format!("change {}", app.review.changes[*change].name),
            palette.heading(),
        ),
        Row::Capability { change, capability } => Line::from(vec![
            guide,
            Span::styled(
                app.review.changes[*change].capabilities[*capability]
                    .name
                    .clone(),
                palette.heading().add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        Row::Artefact { .. } => {
            let a = app.artefact_at(row).expect("artefact row");
            let mark = a.state.status(a.text_hash()).mark();
            Line::from(vec![
                guide,
                Span::raw(format!("{mark} ")),
                Span::styled("· ", palette.muted()),
                Span::raw(a.artefact.name.clone()),
                Span::raw(format!(" {}", note_marker(a.state.note.is_some()))),
            ])
        }
        Row::Requirement { .. } => {
            let p = app.pairing_at(row).expect("requirement row");
            let glyph_style = match p.kind {
                crate::model::DeltaKind::Added => palette.added(),
                crate::model::DeltaKind::Removed => palette.removed(),
                _ => palette.changed(),
            };
            let marker_style = match p.worst_severity() {
                Some(Severity::Error) => palette.error(),
                Some(Severity::Warning) => palette.warning(),
                _ => Style::default(),
            };
            Line::from(vec![
                guide,
                Span::raw(format!("{} {} ", fold_marker(app, row), approval(p).mark())),
                Span::styled(format!("{} ", p.kind.glyph()), glyph_style),
                Span::raw(p.name.clone()),
                Span::raw(" "),
                Span::styled(finding_marker(p).to_string(), marker_style),
                Span::raw(note_marker(p.has_note()).to_string()),
            ])
        }
        Row::Scenario { .. } => {
            let (p, m) = app.scenario_at(row).expect("scenario row");
            let kind = m.heading_kind();
            let glyph_style = match kind {
                ParaKind::Added => palette.added(),
                ParaKind::Removed => palette.removed(),
                _ => palette.changed(),
            };
            let worst = p
                .findings
                .iter()
                .filter(|f| f.location.scenario.as_deref() == Some(m.name()))
                .map(|f| f.severity)
                .max();
            let (marker, marker_style) = match worst {
                Some(Severity::Error) => ("!", palette.error()),
                Some(Severity::Warning) => ("?", palette.warning()),
                _ => ("", Style::default()),
            };
            // Neither fold marker nor approval mark: both belong to the
            // requirement, and the guide already says whose scenario it is.
            Line::from(vec![
                guide,
                Span::styled(format!("{} ", kind.glyph()), glyph_style),
                Span::raw(m.name().to_string()),
                Span::raw(" "),
                Span::styled(marker.to_string(), marker_style),
                Span::raw(note_marker(p.scenario_note(m.name()).is_some()).to_string()),
            ])
        }
    }
}

/// A note under the diff, with its anchor and whether the words moved.
fn note_lines(note: &AnchoredNote, palette: Palette) -> Vec<Line<'static>> {
    let mut lines = vec![Line::styled(
        format!("✎ note{}", note.anchor_suffix()),
        palette.heading(),
    )];
    lines.extend(note.text.lines().map(|l| Line::raw(format!("  {l}"))));
    if note.outdated {
        lines.push(Line::styled(
            "  text changed since the note was written",
            palette.warning(),
        ));
    }
    lines
}

/// `▸` or `▾` says what `Space` does on this row; a requirement without
/// scenarios draws a space so the approval marks stay in one column.
fn fold_marker(app: &App, row: &Row) -> &'static str {
    let has_scenarios = app
        .pairing_at(row)
        .is_some_and(|p| !p.diff.scenarios.is_empty());
    match row.anchor() {
        Some(anchor) if has_scenarios => {
            if app.is_open(anchor) {
                "▾"
            } else {
                "▸"
            }
        }
        _ => " ",
    }
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect, palette: Palette) {
    let guides = tree::guides(&app.rows);
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .zip(&guides)
        .map(|(row, prefix)| ListItem::new(row_line(app, row, prefix, palette)))
        .collect();
    let title = if app.pane == Pane::List {
        "[items]"
    } else {
        " items "
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(palette.selected());
    let mut state = ListState::default().with_selected(Some(app.cursor));
    frame.render_stateful_widget(list, area, &mut state);
}

/// The style a glossary occurrence adds. Underline is orthogonal to the
/// diff's colours, so it composes instead of replacing them; a deprecated
/// synonym also takes the warning style, because a word that must change
/// outranks how it changed.
fn mark_style(mark: Mark, base: Style, palette: Palette) -> Style {
    let underlined = base.add_modifier(Modifier::UNDERLINED);
    match mark {
        Mark::Admitted => underlined,
        Mark::Deprecated(_) => underlined.patch(palette.warning()),
    }
}

/// `text` split on the occurrences that fall inside it. `from` is where
/// `text` starts in the string the occurrences were found in.
fn marked_spans(
    text: &str,
    from: usize,
    occurrences: &[Occurrence],
    style: Style,
    palette: Palette,
) -> Vec<Span<'static>> {
    let end = from + text.len();
    let mut out = Vec::new();
    let mut at = from;
    for o in occurrences {
        if o.range.end <= at || o.range.start >= end {
            continue;
        }
        let start = o.range.start.max(at);
        let stop = o.range.end.min(end);
        if start > at {
            out.push(Span::styled(
                text[at - from..start - from].to_string(),
                style,
            ));
        }
        out.push(Span::styled(
            text[start - from..stop - from].to_string(),
            mark_style(o.mark, style, palette),
        ));
        at = stop;
    }
    if at < end {
        out.push(Span::styled(text[at - from..].to_string(), style));
    }
    out
}

/// The glossary occurrences of a whole string, for a caller that draws it
/// in one piece.
fn occurrences_of<'a>(marks: Option<&'a Marks>, text: &str) -> Vec<Occurrence<'a>> {
    marks.map(|m| m.occurrences(text)).unwrap_or_default()
}

pub fn styled_line(line: &DiffLine, palette: Palette, marks: Option<&Marks>) -> Line<'static> {
    if line.spans.is_empty() {
        return Line::raw("");
    }
    // Occurrences are found over the line's own text, so a term that
    // straddles two diff spans is marked in both halves.
    let plain: String = line.spans.iter().map(|s| s.text.as_str()).collect();
    let occurrences = occurrences_of(marks, &plain);
    let glyph = Span::styled(
        format!("{} ", line.kind.glyph()),
        match line.kind {
            ParaKind::Added => palette.added(),
            ParaKind::Removed => palette.removed(),
            ParaKind::Changed => palette.changed(),
            ParaKind::Separator => palette.muted(),
            ParaKind::Equal => Style::default(),
        },
    );
    let heading = matches!(line.role, LineRole::Name | LineRole::Scenario);
    let base = if heading {
        palette.heading()
    } else {
        Style::default()
    };
    let mut spans = vec![glyph];
    let mut at = 0;
    for s in &line.spans {
        let (prefix, suffix, style) = match (line.kind, s.mark) {
            (ParaKind::Changed, SpanMark::Removed) => ("[-", "-]", palette.removed()),
            (ParaKind::Changed, SpanMark::Added) => ("{+", "+}", palette.added()),
            (ParaKind::Added, _) => ("", "", palette.added()),
            (ParaKind::Removed, _) => ("", "", palette.removed()),
            (ParaKind::Separator, _) => ("", "", palette.muted()),
            _ => ("", "", base),
        };
        let style = style.patch(base);
        if !prefix.is_empty() {
            spans.push(Span::styled(prefix.to_string(), style));
        }
        spans.extend(marked_spans(&s.text, at, &occurrences, style, palette));
        if !suffix.is_empty() {
            spans.push(Span::styled(suffix.to_string(), style));
        }
        at += s.text.len();
    }
    Line::from(spans)
}

/// The lines of a pane that can reach the screen at its scroll offset.
/// A line above the offset becomes an empty placeholder, which keeps
/// ratatui's idea of the text the same height without styling anything; a
/// line at or beyond the last visible row is never built at all. Styling a
/// line costs a pass of every glossary matcher over it, so a pane over a
/// long spec is most of a frame.
struct Viewport {
    first: usize,
    end: usize,
    at: usize,
    lines: Vec<Line<'static>>,
}

impl Viewport {
    /// `wraps` is whether the pane wraps its lines. A wrapped line can
    /// occupy more than one row and so pushes what follows it down; the
    /// window then has to open at the top, and only the tail is cut.
    fn new(scroll: u16, height: u16, wraps: bool) -> Viewport {
        let scroll = usize::from(scroll);
        Viewport {
            first: if wraps { 0 } else { scroll },
            end: scroll.saturating_add(usize::from(height)),
            at: 0,
            lines: Vec::new(),
        }
    }

    /// Whether the window has been filled, after which nothing more is
    /// built.
    fn full(&self) -> bool {
        self.at >= self.end
    }

    fn push(&mut self, build: impl FnOnce() -> Line<'static>) {
        if self.full() {
            return;
        }
        let line = if self.at < self.first {
            Line::raw("")
        } else {
            build()
        };
        self.lines.push(line);
        self.at += 1;
    }

    fn extend(&mut self, count: usize, build: impl Fn(usize) -> Line<'static>) {
        for i in 0..count {
            if self.full() {
                return;
            }
            self.push(|| build(i));
        }
    }

    fn text(self) -> Text<'static> {
        Text::from(self.lines)
    }
}

fn severity_style(s: Severity, palette: Palette) -> Style {
    match s {
        Severity::Error => palette.error(),
        Severity::Warning => palette.warning(),
        Severity::Note => palette.muted(),
    }
}

/// A finding as two or more lines: the message, then its details.
fn finding_lines(f: &crate::review::Finding, palette: Palette) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("{}: ", f.severity),
            severity_style(f.severity, palette),
        ),
        Span::raw(f.message.clone()),
    ])];
    lines.extend(
        f.details
            .iter()
            .map(|d| Line::styled(format!("    {d}"), palette.muted())),
    );
    lines
}

fn push_all(vp: &mut Viewport, lines: Vec<Line<'static>>) {
    for line in lines {
        vp.push(|| line);
    }
}

fn detail_text(app: &App, palette: Palette, width: u16) -> Text<'static> {
    let marks = app.marks();
    let Some(row) = app.current_row() else {
        return Text::raw("nothing to review");
    };
    let mut vp = Viewport::new(
        app.scroll,
        app.detail_height,
        app.mode == DetailMode::Inline,
    );
    if let Some(p) = app.pairing_at(row) {
        match app.mode {
            DetailMode::Inline => {
                let lines = app.detail_lines();
                vp.extend(lines.len(), |i| styled_line(&lines[i], palette, marks));
            }
            DetailMode::SideBySide => {
                side_by_side(app.detail_lines(), width, palette, marks, &mut vp)
            }
            DetailMode::Raw => {
                let before = p
                    .before
                    .as_ref()
                    .map(raw_text)
                    .unwrap_or_else(|| "(no before side)".to_string());
                let after = p
                    .after
                    .as_ref()
                    .map(raw_text)
                    .unwrap_or_else(|| "(no after side)".to_string());
                two_columns(&before, &after, width, palette, marks, &mut vp);
            }
        }
        vp.push(|| Line::raw(""));
        if approval(p) == crate::state::ApprovalStatus::Stale {
            vp.push(|| Line::styled("[~] text changed since approval", palette.warning()));
        }
        for f in &p.findings {
            push_all(&mut vp, finding_lines(f, palette));
        }
        for note in &p.notes {
            push_all(&mut vp, note_lines(note, palette));
        }
        vp.push(|| Line::styled(history_summary(p), palette.muted()));
        if app.definitions_shown() {
            vp.push(|| Line::raw(""));
            vp.push(|| Line::styled("definitions", palette.heading()));
            let text = p
                .after
                .as_ref()
                .or(p.before.as_ref())
                .map(crate::review::pair::requirement_text)
                .unwrap_or_default();
            let terms = app.review.glossary.terms_in(&text);
            if terms.is_empty() {
                vp.push(|| Line::styled("  no glossary term appears here", palette.muted()));
            }
            for t in terms {
                vp.push(|| {
                    Line::styled(
                        format!("  {}", t.name),
                        Style::default().add_modifier(Modifier::BOLD),
                    )
                });
                for l in crate::review::normalize::paragraphs(&t.meaning) {
                    vp.push(|| Line::raw(format!("    {l}")));
                }
                if !t.admitted.is_empty() {
                    vp.push(|| Line::styled("    Admitted:", palette.muted()));
                    for a in &t.admitted {
                        vp.push(|| Line::styled(format!("      {a}"), palette.muted()));
                    }
                }
                if !t.deprecated.is_empty() {
                    vp.push(|| Line::styled("    Deprecated:", palette.muted()));
                    for d in &t.deprecated {
                        vp.push(|| Line::styled(format!("      {d}"), palette.muted()));
                    }
                }
            }
        }
    } else if let Some((p, m)) = app.scenario_at(row) {
        let lines = app.detail_lines();
        vp.extend(lines.len(), |i| styled_line(&lines[i], palette, marks));
        vp.push(|| Line::raw(""));
        for f in p
            .findings
            .iter()
            .filter(|f| f.location.scenario.as_deref() == Some(m.name()))
        {
            vp.push(|| {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", f.severity),
                        severity_style(f.severity, palette),
                    ),
                    Span::raw(f.message.clone()),
                ])
            });
        }
        if let Some(note) = p.scenario_note(m.name()) {
            push_all(&mut vp, note_lines(note, palette));
        }
    } else if let Some(a) = app.artefact_at(row) {
        let lines = app.detail_lines();
        vp.extend(lines.len(), |i| styled_line(&lines[i], palette, marks));
        if let Some(note) = &a.state.note {
            vp.push(|| Line::raw(""));
            vp.push(|| Line::styled("✎ note", palette.heading()));
            for l in note.text.lines() {
                vp.push(|| Line::raw(format!("  {l}")));
            }
            if a.note_outdated() {
                vp.push(|| {
                    Line::styled(
                        "  text changed since the note was written",
                        palette.warning(),
                    )
                });
            }
        }
    }
    vp.text()
}

fn raw_text(req: &crate::model::Requirement) -> String {
    let mut out = format!("### Requirement: {}\n\n{}\n", req.name, req.body);
    for s in &req.scenarios {
        out.push_str(&format!("\n#### Scenario: {}\n\n{}\n", s.name, s.body));
    }
    out
}

fn fit(text: &str, width: usize) -> String {
    let mut s: String = text.chars().take(width).collect();
    while s.chars().count() < width {
        s.push(' ');
    }
    s
}

fn side_by_side(
    lines: &[DiffLine],
    width: u16,
    palette: Palette,
    marks: Option<&Marks>,
    vp: &mut Viewport,
) {
    let half = (usize::from(width).saturating_sub(3) / 2).max(8);
    vp.extend(lines.len(), |i| {
        let l = &lines[i];
        let left = l.side(SpanMark::Added);
        let right = l.side(SpanMark::Removed);
        let style_for = |present: bool| match (l.kind, present) {
            (ParaKind::Added, true) => palette.added(),
            (ParaKind::Removed, true) => palette.removed(),
            (ParaKind::Changed, true) => palette.changed(),
            _ => Style::default(),
        };
        let column = |text: Option<&str>, present: bool| {
            let text = fit(text.unwrap_or(""), half);
            let occurrences = occurrences_of(marks, &text);
            marked_spans(&text, 0, &occurrences, style_for(present), palette)
        };
        let mut spans = column(left.as_deref(), left.is_some());
        spans.push(Span::styled(
            format!(" {} ", l.kind.glyph()),
            palette.muted(),
        ));
        spans.extend(column(right.as_deref(), right.is_some()));
        Line::from(spans)
    });
}

fn two_columns(
    left: &str,
    right: &str,
    width: u16,
    palette: Palette,
    marks: Option<&Marks>,
    vp: &mut Viewport,
) {
    let half = (usize::from(width).saturating_sub(3) / 2).max(8);
    let l: Vec<&str> = left.lines().collect();
    let r: Vec<&str> = right.lines().collect();
    vp.extend(l.len().max(r.len()), |i| {
        let column = |text: &str| {
            let text = fit(text, half);
            let occurrences = occurrences_of(marks, &text);
            marked_spans(&text, 0, &occurrences, Style::default(), palette)
        };
        let mut spans = column(l.get(i).copied().unwrap_or(""));
        spans.push(Span::styled(" │ ", palette.muted()));
        spans.extend(column(r.get(i).copied().unwrap_or("")));
        Line::from(spans)
    });
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect, palette: Palette) {
    let title = format!(
        "{}{}{}",
        if app.pane == Pane::Detail { "[" } else { " " },
        app.mode.label(),
        if app.pane == Pane::Detail { "]" } else { " " }
    );
    let text = detail_text(app, palette, area.width.saturating_sub(2));
    let wrap = match app.mode {
        DetailMode::Inline => Some(Wrap { trim: false }),
        _ => None,
    };
    let mut paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .scroll((app.scroll, 0));
    if let Some(w) = wrap {
        paragraph = paragraph.wrap(w);
    }
    frame.render_widget(paragraph, area);
}

fn draw_history(frame: &mut Frame, app: &App, left: Rect, right: Rect, palette: Palette) {
    let Some(h) = app.history_state() else {
        return;
    };
    let Some(p) = app.current_pairing() else {
        return;
    };
    let versions = history_entries(p);
    let items: Vec<ListItem> = if p.history.is_empty() {
        let mut v = vec![ListItem::new(Line::styled("no history", palette.muted()))];
        v.extend(
            versions
                .iter()
                .map(|(label, _)| ListItem::new(label.clone())),
        );
        v
    } else {
        versions
            .iter()
            .map(|(label, _)| ListItem::new(label.clone()))
            .collect()
    };
    let offset = usize::from(p.history.is_empty());
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("history: {}", p.name)),
        )
        .highlight_style(palette.selected());
    let mut state = ListState::default().with_selected(Some(h.selected + offset));
    frame.render_stateful_widget(list, left, &mut state);

    let title = match h.mode {
        HistoryMode::Version => "version",
        HistoryMode::DiffToPrevious => "diff to previous",
    };
    let marks = app.marks();
    let lines = app.history_lines();
    let mut vp = Viewport::new(h.scroll, right.height.saturating_sub(2), true);
    vp.extend(lines.len(), |i| styled_line(&lines[i], palette, marks));
    let paragraph = Paragraph::new(vp.text())
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false })
        .scroll((h.scroll, 0));
    frame.render_widget(paragraph, right);
}

pub fn status_text(app: &App) -> String {
    let (approved, total) = app.approval_counts();
    let (e, w, n) = app.finding_counts();
    let mode = match (app.history_state(), app.note_edit()) {
        (Some(h), _) => match h.mode {
            HistoryMode::Version => "history",
            HistoryMode::DiffToPrevious => "history diff",
        },
        (_, Some(_)) => "note",
        _ => app.mode.label(),
    };
    let mut s = format!(
        " {}  {approved}/{total} approved  {e} errors {w} warnings {n} notes  mode: {mode}  ? help",
        app.current_change_name()
    );
    if let Some(m) = &app.message {
        s.push_str("  ");
        s.push_str(m);
    }
    s
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect, palette: Palette) {
    frame.render_widget(
        Paragraph::new(status_text(app)).style(palette.selected()),
        area,
    );
}

fn draw_help(frame: &mut Frame, area: Rect, palette: Palette) {
    let height = (BINDINGS.len() as u16 + 4).min(area.height);
    let width = 64.min(area.width);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    let lines: Vec<Line> = BINDINGS
        .iter()
        .map(|(k, what)| {
            Line::from(vec![
                Span::styled(format!("{k:<30}"), palette.heading()),
                Span::raw(*what),
            ])
        })
        .chain(std::iter::once(Line::raw("")))
        .chain(std::iter::once(Line::styled(
            "any key closes this",
            palette.muted(),
        )))
        .collect();
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("keys")),
        popup,
    );
}

/// The note popup, drawn over the detail pane. It quotes its anchor, so
/// what the note is about stays readable whatever is behind it.
fn draw_note(frame: &mut Frame, edit: &NoteEdit, area: Rect, palette: Palette) {
    let width = area.width.max(20);
    let inner = usize::from(width.saturating_sub(2));
    let quote: Vec<String> = edit
        .quote
        .iter()
        .flat_map(|p| wrapped(p, inner.saturating_sub(2)))
        .collect();
    let buffer = wrapped(&edit.buffer, inner);
    let height = (quote.len() + buffer.len() + 6).min(usize::from(area.height)) as u16;
    let popup = Rect {
        x: area.x,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    let mut lines: Vec<Line> = vec![Line::styled(edit.title.clone(), palette.heading())];
    lines.extend(
        quote
            .iter()
            .map(|l| Line::styled(format!("  {l}"), palette.muted())),
    );
    lines.push(Line::raw(""));
    lines.extend(buffer.iter().map(|l| Line::raw(l.clone())));
    if buffer.is_empty() {
        lines.push(Line::raw(""));
    }
    // The mode is named in text as well as drawn, so it does not depend on
    // the popup's border being noticed.
    lines.push(Line::styled(
        "writing a note: ⏎ save, Esc cancel, ^E $EDITOR",
        palette.muted(),
    ));
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("note")),
        popup,
    );
    let caret_row = quote.len() + 2 + buffer.len().saturating_sub(1);
    let caret_col = buffer.last().map(|l| l.chars().count()).unwrap_or(0);
    let x = popup.x + 1 + caret_col.min(inner.saturating_sub(1)) as u16;
    let y = popup.y + 1 + caret_row as u16;
    if y < popup.y + popup.height - 1 {
        frame.set_cursor_position((x, y));
    }
}

/// Greedy wrap at `width`, which is all a buffer with no line structure
/// needs.
fn wrapped(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let width = width.max(8);
    let mut out: Vec<String> = Vec::new();
    for word in text.split(' ') {
        match out.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => out.push(word.to_string()),
        }
    }
    out
}
