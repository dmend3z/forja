pub mod app;
pub mod input;
pub mod scan_app;
pub mod scan_input;
pub mod scan_ui;
pub mod ui;

use std::io;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use forja_core::error::{ForjaError, Result};
use forja_core::models::profile::Profile;
use forja_core::models::state::load_state;
use forja_core::paths::ForjaPaths;

use app::App;

/// Output produced by the TUI when the user presses launch.
pub struct TaskOutput {
    pub description: String,
    pub team: Option<String>,
    pub profile: Profile,
}

/// Output produced by the plan TUI when the user presses create.
pub struct PlanOutput {
    pub description: String,
}

/// Open the interactive TUI for task configuration.
/// Returns `Some(TaskOutput)` on launch, `None` on Esc/quit.
pub fn launch() -> Result<Option<TaskOutput>> {
    // Check for interactive terminal
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() {
        return Err(ForjaError::Dialoguer(
            "TUI requires an interactive terminal. Use: forja task \"description\"".to_string(),
        ));
    }

    // Build team options from forja state
    let (team_labels, team_names) = load_team_options();
    let profile_options = vec!["fast".to_string(), "balanced".to_string(), "max".to_string()];

    // Default profile index to "balanced" (index 1)
    let mut app = App::new(team_labels, team_names, profile_options);
    app.profile_index = 1;

    // Terminal setup
    enable_raw_mode().map_err(|e| ForjaError::Dialoguer(format!("raw mode: {e}")))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| ForjaError::Dialoguer(format!("alternate screen: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| ForjaError::Dialoguer(format!("terminal: {e}")))?;

    // Main loop
    let result = run_loop(&mut terminal, &mut app);

    // Terminal teardown (always runs, even on error)
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    // Propagate any error from the loop
    result?;

    if app.should_launch {
        let profile: Profile = app
            .selected_profile()
            .parse()
            .map_err(|e: String| ForjaError::Dialoguer(e))?;
        Ok(Some(TaskOutput {
            description: app.description(),
            team: app.selected_team().map(|s| s.to_string()),
            profile,
        }))
    } else {
        Ok(None)
    }
}

/// Open the interactive TUI for plan creation (textarea only, no team/profile).
/// Returns `Some(PlanOutput)` on create, `None` on Esc/quit.
pub fn launch_plan() -> Result<Option<PlanOutput>> {
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() {
        return Err(ForjaError::Dialoguer(
            "TUI requires an interactive terminal. Use: forja plan \"description\"".to_string(),
        ));
    }

    let mut app = App::new_plan();

    enable_raw_mode().map_err(|e| ForjaError::Dialoguer(format!("raw mode: {e}")))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| ForjaError::Dialoguer(format!("alternate screen: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| ForjaError::Dialoguer(format!("terminal: {e}")))?;

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result?;

    if app.should_launch {
        Ok(Some(PlanOutput {
            description: app.description(),
        }))
    } else {
        Ok(None)
    }
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| ui::render(frame, app))
            .map_err(|e| ForjaError::Dialoguer(format!("draw: {e}")))?;

        if let Event::Key(key) = event::read().map_err(ForjaError::Io)? {
            input::handle_key(app, key);
        }

        if app.should_quit || app.should_launch {
            break;
        }
    }
    Ok(())
}

/// Open the interactive TUI for scan result selection (checkboxes).
/// Returns `Some(selected_skill_ids)` on confirm, `None` on Esc/quit.
pub fn launch_scan(
    recommendations: Vec<forja_core::scanner::models::SkillRecommendation>,
    show_installed: bool,
    tech_count: usize,
) -> Result<Option<Vec<String>>> {
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() {
        return Err(ForjaError::Dialoguer(
            "TUI requires an interactive terminal. Use: forja scan --yes".to_string(),
        ));
    }

    let mut app = scan_app::ScanApp::new(recommendations, show_installed);
    app.tech_count = tech_count;

    if app.is_empty() {
        return Ok(None);
    }

    enable_raw_mode().map_err(|e| ForjaError::Dialoguer(format!("raw mode: {e}")))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| ForjaError::Dialoguer(format!("alternate screen: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| ForjaError::Dialoguer(format!("terminal: {e}")))?;

    let result = run_scan_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result?;

    if app.should_install {
        Ok(Some(app.selected_skill_ids()))
    } else {
        Ok(None)
    }
}

fn run_scan_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut scan_app::ScanApp,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| scan_ui::render(frame, app))
            .map_err(|e| ForjaError::Dialoguer(format!("draw: {e}")))?;

        if let Event::Key(key) = event::read().map_err(ForjaError::Io)? {
            scan_input::handle_key(app, key);
        }

        if app.should_quit || app.should_install {
            break;
        }
    }
    Ok(())
}

/// Load team options from ForjaState, falling back to presets.
/// Returns (labels_for_display, names_for_logic) — both indexed by team_index.
fn load_team_options() -> (Vec<String>, Vec<String>) {
    let mut labels = vec!["Solo".to_string()];
    let mut names = vec!["solo".to_string()];

    let presets: &[(&str, &str)] = &[
        ("quick-fix", "quick-fix (coder + deployer)"),
        ("solo-sprint", "solo-sprint (coder-tester + reviewer)"),
        ("full-product", "full-product (5 agents)"),
    ];

    if let Ok(paths) = ForjaPaths::new() && paths.forja_root.exists() {
        let state = load_state(&paths.state);
        for &(name, label) in presets {
            if !state.teams.contains_key(name) {
                labels.push(label.to_string());
                names.push(name.to_string());
            }
        }
        let mut configured: Vec<_> = state.teams.iter().collect();
        configured.sort_by_key(|(n, _)| n.to_string());
        for (name, entry) in configured {
            labels.push(format!("{} ({} agents)", name, entry.members.len()));
            names.push(name.clone());
        }
        return (labels, names);
    }

    // Fallback: just presets (forja not initialized)
    for &(name, label) in presets {
        labels.push(label.to_string());
        names.push(name.to_string());
    }
    (labels, names)
}
