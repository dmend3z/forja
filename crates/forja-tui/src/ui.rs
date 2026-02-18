use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Focus, TuiMode};

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    if app.mode == TuiMode::Plan {
        let chunks = Layout::vertical([
            Constraint::Length(1),  // title
            Constraint::Min(8),    // textarea
            Constraint::Min(4),    // preview
            Constraint::Length(1), // submit button
            Constraint::Length(1), // help bar
        ])
        .split(area);

        render_title(frame, app, chunks[0]);
        render_textarea(frame, app, chunks[1]);
        render_preview(frame, app, chunks[2]);
        render_submit(frame, app, chunks[3]);
        render_help(frame, app, chunks[4]);
    } else {
        let chunks = Layout::vertical([
            Constraint::Length(1),  // title
            Constraint::Min(6),    // textarea
            Constraint::Length(3), // config (team + profile + gap)
            Constraint::Min(4),    // preview
            Constraint::Length(1), // help bar
        ])
        .split(area);

        render_title(frame, app, chunks[0]);
        render_textarea(frame, app, chunks[1]);
        render_config(frame, app, chunks[2]);
        render_preview(frame, app, chunks[3]);
        render_help(frame, app, chunks[4]);
    }
}

fn render_title(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let label = match app.mode {
        TuiMode::Task => " forja task ",
        TuiMode::Plan => " forja plan ",
    };
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            label,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(title, area);
}

fn render_textarea(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let border_style = if app.focus == Focus::Textarea {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Task Description ");
    app.textarea.set_block(block);
    frame.render_widget(&app.textarea, area);
}

fn render_config(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    // Team row
    let team_spans: Vec<Span> = std::iter::once(Span::styled(
        "  Team:    ",
        Style::default().fg(Color::White),
    ))
    .chain(app.team_labels.iter().enumerate().map(|(i, name)| {
        if i == app.team_index {
            let style = if app.focus == Focus::Team {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };
            Span::styled(format!(" {name} "), style)
        } else {
            Span::styled(format!(" {name} "), Style::default().fg(Color::DarkGray))
        }
    }))
    .collect();
    frame.render_widget(Paragraph::new(Line::from(team_spans)), rows[0]);

    // Profile row
    let profile_spans: Vec<Span> = std::iter::once(Span::styled(
        "  Profile: ",
        Style::default().fg(Color::White),
    ))
    .chain(
        app.profile_options
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if i == app.profile_index {
                    let style = if app.focus == Focus::Profile {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    };
                    Span::styled(format!(" {name} "), style)
                } else {
                    Span::styled(
                        format!(" {name} "),
                        Style::default().fg(Color::DarkGray),
                    )
                }
            }),
    )
    .collect();
    frame.render_widget(Paragraph::new(Line::from(profile_spans)), rows[1]);
}

fn render_submit(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let style = if app.focus == Focus::Submit {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let hint = if app.focus == Focus::Submit { "  press Enter" } else { "" };
    let spans = vec![
        Span::raw("  "),
        Span::styled(" Create Plan ", style),
        Span::styled(hint, Style::default().fg(Color::DarkGray)),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_preview(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let preview_text = build_preview(app);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Prompt Preview ");
    let para = Paragraph::new(preview_text)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_help(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut spans = if app.mode == TuiMode::Plan {
        vec![
            Span::styled(" Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": select button  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": quit"),
        ]
    } else {
        vec![
            Span::styled(" Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": next  "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": prev  "),
            Span::styled("Ctrl+Enter", Style::default().fg(Color::Green)),
            Span::raw(": launch  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": quit"),
        ]
    };

    if let Some(ref msg) = app.error_message {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            msg.as_str(),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn build_preview(app: &App) -> String {
    let desc = app.description();
    if desc.trim().is_empty() {
        return match app.mode {
            TuiMode::Plan => "(describe what you want to plan above)".to_string(),
            TuiMode::Task => "(type a task description above)".to_string(),
        };
    }

    if app.mode == TuiMode::Plan {
        return format!("Task:\n{desc}");
    }

    let mut preview = String::new();
    match app.selected_team() {
        Some(team) => {
            preview.push_str(&format!("Mode: team ({team})\n"));
            preview.push_str(&format!("Profile: {}\n", app.selected_profile()));
        }
        None => {
            preview.push_str("Mode: solo\n");
        }
    }
    preview.push_str(&format!("\nTask:\n{desc}"));
    preview
}
