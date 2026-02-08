use crate::error::{ForjaError, Result};
use crate::models::phase::Phase;
use crate::output;
use crate::paths::ForjaMode;
use dialoguer::{MultiSelect, Select};

pub struct WizardResult {
    pub mode: ForjaMode,
    pub selected_phases: Vec<Phase>,
    pub profile: String,
}

/// Run the 3-step interactive init wizard.
pub fn run_wizard() -> Result<WizardResult> {
    output::print_banner();

    output::print_step(1, 3, "Setup mode");
    let mode = prompt_mode()?;

    output::print_step(2, 3, "Workflow phases");
    let selected_phases = prompt_skill_phases()?;

    output::print_step(3, 3, "Model profile");
    let profile = prompt_profile()?;

    Ok(WizardResult {
        mode,
        selected_phases,
        profile,
    })
}

fn prompt_mode() -> Result<ForjaMode> {
    let items = vec![
        "Project — skills scoped to this repo (.forja/)",
        "Global  — shared across all projects (~/.forja/)",
    ];

    let selection = Select::new()
        .with_prompt("How do you want to use forja?")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    Ok(match selection {
        0 => ForjaMode::Project,
        _ => ForjaMode::Global,
    })
}

fn prompt_skill_phases() -> Result<Vec<Phase>> {
    let phases = vec![
        Phase::Research,
        Phase::Code,
        Phase::Test,
        Phase::Review,
        Phase::Deploy,
    ];
    let labels: Vec<String> = phases
        .iter()
        .map(|p| format!("{:<10}— {}", p.as_str(), p.description()))
        .collect();

    // All selected by default
    let defaults: Vec<bool> = vec![true; phases.len()];

    loop {
        let selections = MultiSelect::new()
            .with_prompt("Select phases (space to toggle, enter to confirm)")
            .items(&labels)
            .defaults(&defaults)
            .interact()
            .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

        if selections.is_empty() {
            eprintln!("At least one phase is required. Please select again.");
            continue;
        }

        return Ok(selections.into_iter().map(|i| phases[i]).collect());
    }
}

fn prompt_profile() -> Result<String> {
    let items = vec![
        "balanced — Good trade-off of speed and quality (recommended)",
        "fast     — Fastest responses, best for simple tasks",
        "max      — Maximum quality, uses most capable models",
    ];

    let selection = Select::new()
        .with_prompt("Default model profile")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    Ok(match selection {
        1 => "fast".to_string(),
        2 => "max".to_string(),
        _ => "balanced".to_string(),
    })
}
