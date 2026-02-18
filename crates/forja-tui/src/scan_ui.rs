use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::scan_app::ScanApp;
use forja_core::scanner::models::Confidence;

pub fn render(frame: &mut Frame, app: &ScanApp) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(2), // title + blank line
        Constraint::Min(4),   // items
        Constraint::Length(1), // help bar
    ])
    .split(area);

    render_title(frame, app, chunks[0]);
    render_items(frame, app, chunks[1]);
    render_help(frame, chunks[2]);
}

fn render_title(frame: &mut Frame, app: &ScanApp, area: Rect) {
    let tech_count = app.tech_count;
    let total = app.total_items();
    let checked = app.checked_count();

    let title = Line::from(vec![
        Span::styled(
            " forja scan ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("— {tech_count} technologies detected, {total} skills recommended, {checked} selected"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(title), area);
}

fn render_items(frame: &mut Frame, app: &ScanApp, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    for (gi, (phase, items)) in app.groups.iter().enumerate() {
        // Phase header
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                capitalize(phase),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]));

        for (ii, item) in items.iter().enumerate() {
            let is_cursor = app.cursor == (gi, ii);

            let checkbox = if item.locked {
                Span::styled(
                    " [=] ",
                    Style::default().fg(Color::DarkGray),
                )
            } else if item.checked {
                Span::styled(
                    " [x] ",
                    Style::default().fg(Color::Green),
                )
            } else {
                Span::styled(
                    " [ ] ",
                    Style::default().fg(Color::DarkGray),
                )
            };

            let name_style = if item.locked {
                Style::default().fg(Color::DarkGray)
            } else if is_cursor {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let confidence_span = confidence_span(&item.rec.confidence);

            let reason_style = Style::default().fg(Color::DarkGray);

            let reason_text = if item.locked {
                "(installed)".to_string()
            } else {
                item.rec.reason.clone()
            };

            let cursor_indicator = if is_cursor {
                Span::styled(">", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" ")
            };

            let name_padded = format!("{:<20}", item.rec.name);

            lines.push(Line::from(vec![
                cursor_indicator,
                checkbox,
                Span::styled(name_padded, name_style),
                Span::raw(" "),
                confidence_span,
                Span::raw(" "),
                Span::styled(reason_text, reason_style),
            ]));
        }

        // Blank line between groups
        lines.push(Line::raw(""));
    }

    // Apply scroll offset
    let visible_height = area.height as usize;
    let cursor_line = calculate_cursor_line(app);

    // Adjust scroll to keep cursor visible
    let scroll = if cursor_line < app.scroll_offset {
        cursor_line
    } else if cursor_line >= app.scroll_offset + visible_height {
        cursor_line.saturating_sub(visible_height - 1)
    } else {
        app.scroll_offset
    };

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();

    frame.render_widget(Paragraph::new(visible_lines), area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" Space", Style::default().fg(Color::Cyan)),
        Span::raw(": toggle  "),
        Span::styled("a", Style::default().fg(Color::Cyan)),
        Span::raw(": all  "),
        Span::styled("n", Style::default().fg(Color::Cyan)),
        Span::raw(": none  "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next group  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": install  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": quit"),
    ]);
    frame.render_widget(Paragraph::new(help), area);
}

fn confidence_span(confidence: &Confidence) -> Span<'static> {
    let (text, color) = match confidence {
        Confidence::High => ("+++", Color::Green),
        Confidence::Medium => ("++ ", Color::Yellow),
        Confidence::Low => ("+  ", Color::DarkGray),
    };
    Span::styled(text, Style::default().fg(color))
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Calculate which display line the cursor is on (accounting for headers and blank lines).
fn calculate_cursor_line(app: &ScanApp) -> usize {
    let (target_gi, target_ii) = app.cursor;
    let mut line = 0;

    for (gi, (_, items)) in app.groups.iter().enumerate() {
        line += 1; // phase header

        if gi == target_gi {
            line += target_ii;
            return line;
        }

        line += items.len();
        line += 1; // blank line between groups
    }

    line
}
