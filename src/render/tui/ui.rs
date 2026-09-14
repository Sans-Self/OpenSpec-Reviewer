//! Drawing. Every mark carries a glyph; the palette only adds colour.

use super::app::{
    artefact_hash, history_entries, App, DetailMode, HistoryMode, Pane, Row, View, BINDINGS,
};
use crate::render::colour::Palette;
use crate::render::{approval, finding_marker, history_summary, note_marker};
use crate::review::{DiffLine, LineRole, ParaKind, Severity, SpanMark};
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

    match app.view.clone() {
        View::History(_) => draw_history(frame, app, panes[0], panes[1], palette),
        _ => {
            draw_list(frame, app, panes[0], palette);
            draw_detail(frame, app, panes[1], palette);
        }
    }
    draw_status(frame, app, vertical[1], palette);
    if app.view == View::Help {
        draw_help(frame, area, palette);
    }
}

fn row_line(app: &App, row: &Row, palette: Palette) -> Line<'static> {
    match row {
        Row::Change { change } => Line::styled(
            format!("change {}", app.review.changes[*change].name),
            palette.heading(),
        ),
        Row::Capability { change, capability } => Line::styled(
            format!(
                "  {}",
                app.review.changes[*change].capabilities[*capability].name
            ),
            palette.heading().add_modifier(Modifier::UNDERLINED),
        ),
        Row::Artefact { .. } => {
            let a = app.artefact_at(row).expect("artefact row");
            let mark = a.state.status(artefact_hash(a)).mark();
            Line::from(vec![
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
                Span::raw(format!("    {} ", approval(p).mark())),
                Span::styled(format!("{} ", p.kind.glyph()), glyph_style),
                Span::raw(p.name.clone()),
                Span::raw(" "),
                Span::styled(finding_marker(p).to_string(), marker_style),
                Span::raw(note_marker(p.state.note.is_some()).to_string()),
            ])
        }
    }
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect, palette: Palette) {
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| ListItem::new(row_line(app, row, palette)))
        .collect();
    let title = if app.focus == Pane::List {
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

pub fn styled_line(line: &DiffLine, palette: Palette) -> Line<'static> {
    if line.spans.is_empty() {
        return Line::raw("");
    }
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
    for s in &line.spans {
        let (prefix, suffix, style) = match (line.kind, s.mark) {
            (ParaKind::Changed, SpanMark::Removed) => ("[-", "-]", palette.removed()),
            (ParaKind::Changed, SpanMark::Added) => ("{+", "+}", palette.added()),
            (ParaKind::Added, _) => ("", "", palette.added()),
            (ParaKind::Removed, _) => ("", "", palette.removed()),
            (ParaKind::Separator, _) => ("", "", palette.muted()),
            _ => ("", "", base),
        };
        spans.push(Span::styled(
            format!("{prefix}{}{suffix}", s.text),
            style.patch(base),
        ));
    }
    Line::from(spans)
}

fn severity_style(s: Severity, palette: Palette) -> Style {
    match s {
        Severity::Error => palette.error(),
        Severity::Warning => palette.warning(),
        Severity::Note => palette.muted(),
    }
}

fn detail_text(app: &App, palette: Palette, width: u16) -> Text<'static> {
    let Some(row) = app.current_row() else {
        return Text::raw("nothing to review");
    };
    let mut lines: Vec<Line<'static>> = Vec::new();
    if let Some(p) = app.pairing_at(row) {
        match app.mode {
            DetailMode::Inline => {
                lines.extend(app.detail_lines().iter().map(|l| styled_line(l, palette)))
            }
            DetailMode::SideBySide => {
                lines.extend(side_by_side(&app.detail_lines(), width, palette))
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
                lines.extend(two_columns(&before, &after, width, palette));
            }
        }
        lines.push(Line::raw(""));
        if approval(p) == crate::state::ApprovalStatus::Stale {
            lines.push(Line::styled(
                "[~] text changed since approval",
                palette.warning(),
            ));
        }
        for f in &p.findings {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", f.severity),
                    severity_style(f.severity, palette),
                ),
                Span::raw(f.message.clone()),
            ]));
            lines.extend(
                f.details
                    .iter()
                    .map(|d| Line::styled(format!("    {d}"), palette.muted())),
            );
        }
        if let Some(note) = &p.state.note {
            lines.push(Line::styled("✎ note", palette.heading()));
            lines.extend(note.lines().map(|l| Line::raw(format!("  {l}"))));
        }
        lines.push(Line::styled(history_summary(p), palette.muted()));
    } else if let Some(a) = app.artefact_at(row) {
        lines.extend(a.lines().iter().map(|l| styled_line(l, palette)));
        if let Some(note) = &a.state.note {
            lines.push(Line::raw(""));
            lines.push(Line::styled("✎ note", palette.heading()));
            lines.extend(note.lines().map(|l| Line::raw(format!("  {l}"))));
        }
    }
    Text::from(lines)
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

fn side_by_side(lines: &[DiffLine], width: u16, palette: Palette) -> Vec<Line<'static>> {
    let half = (usize::from(width).saturating_sub(3) / 2).max(8);
    lines
        .iter()
        .map(|l| {
            let left = l.side(SpanMark::Added);
            let right = l.side(SpanMark::Removed);
            let style_for = |present: bool| match (l.kind, present) {
                (ParaKind::Added, true) => palette.added(),
                (ParaKind::Removed, true) => palette.removed(),
                (ParaKind::Changed, true) => palette.changed(),
                _ => Style::default(),
            };
            Line::from(vec![
                Span::styled(
                    fit(left.as_deref().unwrap_or(""), half),
                    style_for(left.is_some()),
                ),
                Span::styled(format!(" {} ", l.kind.glyph()), palette.muted()),
                Span::styled(
                    fit(right.as_deref().unwrap_or(""), half),
                    style_for(right.is_some()),
                ),
            ])
        })
        .collect()
}

fn two_columns(left: &str, right: &str, width: u16, palette: Palette) -> Vec<Line<'static>> {
    let half = (usize::from(width).saturating_sub(3) / 2).max(8);
    let l: Vec<&str> = left.lines().collect();
    let r: Vec<&str> = right.lines().collect();
    (0..l.len().max(r.len()))
        .map(|i| {
            Line::from(vec![
                Span::raw(fit(l.get(i).copied().unwrap_or(""), half)),
                Span::styled(" │ ", palette.muted()),
                Span::raw(fit(r.get(i).copied().unwrap_or(""), half)),
            ])
        })
        .collect()
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect, palette: Palette) {
    let title = format!(
        "{}{}{}",
        if app.focus == Pane::Detail { "[" } else { " " },
        app.mode.label(),
        if app.focus == Pane::Detail { "]" } else { " " }
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
    let View::History(h) = &app.view else {
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
    let lines: Vec<Line<'static>> = app
        .history_lines()
        .iter()
        .map(|l| styled_line(l, palette))
        .collect();
    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false })
        .scroll((h.scroll, 0));
    frame.render_widget(paragraph, right);
}

pub fn status_text(app: &App) -> String {
    let (approved, total) = app.approval_counts();
    let (e, w, n) = app.finding_counts();
    let mode = match &app.view {
        View::History(h) => match h.mode {
            HistoryMode::Version => "history",
            HistoryMode::DiffToPrevious => "history diff",
        },
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
