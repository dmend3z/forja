use forja_core::error::Result;
use forja_core::models::config::{self, ForjaConfig};
use crate::output;
use forja_core::paths::{ForjaMode, ForjaPaths};
use forja_core::registry::catalog;
use forja_core::settings;
use forja_core::symlink::manager::save_installed_ids;
use crate::wizard;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Initialize forja with sensible defaults or `--wizard` for interactive setup.
pub fn run(registry_url: Option<String>, use_wizard: bool, force_global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Check for existing .forja/ in cwd (restore flow)
    let existing_config = cwd.join(".forja").join("config.json");
    if existing_config.exists() {
        return restore_flow(&cwd, registry_url);
    }

    // Default: project-local mode, all phases, balanced profile
    // --global: force global mode
    // --wizard: interactive setup
    let (mode, selected_phases, profile) = if use_wizard {
        let result = wizard::run_wizard()?;
        (result.mode, result.selected_phases, result.profile)
    } else if force_global {
        (ForjaMode::Global, all_phases(), "balanced".to_string())
    } else {
        (ForjaMode::Project, all_phases(), "balanced".to_string())
    };

    let paths = match mode {
        ForjaMode::Project => ForjaPaths::from_project(cwd.clone())?,
        ForjaMode::Global => ForjaPaths::global()?,
    };

    if paths.forja_root.exists() {
        println!(
            "{}",
            "forja is already initialized. Use `forja update` to refresh the registry.".yellow()
        );
        return Ok(());
    }

    let url = registry_url.unwrap_or_else(|| "https://github.com/dmend3z/forja.git".to_string());

    // Create .forja/ directory
    fs::create_dir_all(&paths.forja_root)?;

    // Link or clone registry
    let is_local = catalog::is_forja_registry(&cwd);

    if is_local {
        std::os::unix::fs::symlink(&cwd, &paths.registry)?;
    } else {
        forja_core::registry::git::clone(&url, &paths.registry)?;
    }

    // Write config.json (new format with version + mode)
    let forja_config = ForjaConfig::new(mode, url, is_local);
    config::save_config(&paths.config, &forja_config)?;

    // Create plans directory
    fs::create_dir_all(&paths.plans)?;

    // Write empty state
    save_installed_ids(&paths.state, &[])?;

    // Ensure ~/.claude/agents/ exists
    fs::create_dir_all(&paths.claude_agents)?;

    // Auto-enable agent teams env var in ~/.claude/settings.json (always global)
    let global_claude = ForjaPaths::global_claude_dir()?;
    match settings::enable_teams_env_var(&global_claude) {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "  {} Could not enable teams env var: {}",
                "WARN:".yellow().bold(),
                e
            );
        }
    }

    // Detect project stack (used in scaffold and output)
    let stack = detect_stack(&cwd);

    // In project mode, create .gitignore files and scaffold the spec framework
    if mode == ForjaMode::Project {
        let gitignore_path = paths.forja_root.join(".gitignore");
        fs::write(
            &gitignore_path,
            "# Managed by forja - do not edit\nregistry/\nplans/\nruns/\n",
        )?;

        // Exclude symlinked dirs from version control
        let claude_gitignore = paths.claude_dir.join(".gitignore");
        if !claude_gitignore.exists() {
            fs::create_dir_all(&paths.claude_dir)?;
            fs::write(
                &claude_gitignore,
                "# Managed by forja\nagents/\ncommands/\n",
            )?;
        }

        // Scaffold .forja/ spec framework directories and template files
        scaffold_framework(&paths.forja_root, &stack)?;
    }

    // Install skills filtered by selected phases
    let (installed, _skipped) = super::install::install_by_phases(&paths, &selected_phases)?;

    // Core skills always installed (regardless of phase selection)
    forja_core::symlink::auto_install::auto_install_missing(
        &paths,
        &["review/documentation/chronicler"],
    )?;

    // Sync symlinks to ~/.claude/
    let _sync_result = forja_core::symlink::sync::sync_symlinks(&paths)?;

    // Output
    output::print_divider();

    output::print_section_header("Results");
    output::print_success(&format!("forja initialized ({})", mode_label(mode)));
    output::print_success(&format!(
        "{} skills installed ({})",
        installed,
        selected_phases
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    output::print_section_header("Setup");
    println!("  {}  {}", "Mode:".cyan().bold(), mode_label(mode));
    if let Some(ref detected) = stack {
        println!("  {}  {}", "Stack:".cyan().bold(), detected);
    }
    println!("  {}  {}", "Profile:".cyan().bold(), profile);
    if mode == ForjaMode::Project {
        println!("  {}  .forja/", "Location:".cyan().bold());
    }

    output::print_section_header("Next Steps");
    output::print_command_hint("forja specs list", "View your specs");
    output::print_command_hint("forja validate", "Validate your .forja/ framework");
    output::print_command_hint("forja task \"your task\"", "Run a task with AI skills");
    output::print_command_hint("forja doctor", "Verify your setup");

    if mode == ForjaMode::Project {
        println!();
        output::print_tip(".forja/ created — edit specs in .forja/specs/ and commit to git");
    }

    Ok(())
}

fn restore_flow(cwd: &Path, registry_url: Option<String>) -> Result<()> {
    let config = config::load_config(&cwd.join(".forja").join("config.json"));

    output::print_divider();

    output::print_section_header("Found");
    output::print_success("Existing .forja/ detected in this directory");
    if let Some(ref cfg) = config {
        println!(
            "  {}  {}, local: {}",
            "Mode:".cyan().bold(),
            if cfg.mode == ForjaMode::Project {
                "project"
            } else {
                "global"
            },
            cfg.local
        );
    }

    let paths = ForjaPaths::from_project(cwd.to_path_buf())?;

    // Ensure registry exists
    if !paths.registry.exists() {
        let url = registry_url
            .or_else(|| config.as_ref().map(|c| c.registry_url.clone()))
            .unwrap_or_else(|| "https://github.com/dmend3z/forja.git".to_string());

        if catalog::is_forja_registry(cwd) {
            std::os::unix::fs::symlink(cwd, &paths.registry)?;
        } else {
            forja_core::registry::git::clone(&url, &paths.registry)?;
        }
    }

    // Sync symlinks
    forja_core::symlink::sync::sync_symlinks(&paths)?;

    output::print_section_header("Results");
    output::print_success(&format!(
        "Restored — symlinks synced to {}",
        paths.claude_dir.display()
    ));

    output::print_section_header("Next Steps");
    output::print_command_hint("forja doctor", "Verify your setup");
    output::print_command_hint("forja status", "Check current state");

    println!();
    output::print_tip("Run 'forja doctor' to verify your setup");

    Ok(())
}

fn all_phases() -> Vec<forja_core::models::phase::Phase> {
    use forja_core::models::phase::Phase;
    vec![
        Phase::Research,
        Phase::Code,
        Phase::Test,
        Phase::Review,
        Phase::Deploy,
    ]
}

fn mode_label(mode: ForjaMode) -> &'static str {
    match mode {
        ForjaMode::Project => "project mode",
        ForjaMode::Global => "global mode",
    }
}

/// Scaffold the .forja/ spec-driven development framework directories and template files.
fn scaffold_framework(forja_root: &Path, stack: &Option<String>) -> Result<()> {
    // Create framework directories
    fs::create_dir_all(forja_root.join("docs"))?;
    fs::create_dir_all(forja_root.join("tracks"))?;
    fs::create_dir_all(forja_root.join("specs"))?;
    fs::create_dir_all(forja_root.join("decisions"))?;
    fs::create_dir_all(forja_root.join("runs"))?;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Generate vision.md
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "My Project".to_string());

    let tech_stack_yaml = match stack {
        Some(detected) => format!(
            "tech_stack:\n  language: \"{detected}\"\n"
        ),
        None => "# tech_stack:\n#   language: \"\"\n#   framework: \"\"\n#   database: \"\"\n#   hosting: \"\"\n".to_string(),
    };

    let vision = format!(
        r#"---
project: "{project_name}"
description: ""
version: "0.1.0"
{tech_stack_yaml}---
# Product Vision

Describe your product vision here.
"#
    );
    fs::write(forja_root.join("docs").join("vision.md"), vision)?;

    // Generate track template
    let track = format!(
        r#"---
id: mvp
title: "MVP"
description: "Minimum viable product"
status: draft
created: "{today}"
---
# MVP

| ID  | Task    | Status | Spec    |
|-----|---------|--------|---------|
| M1  | Example | todo   | example |
"#
    );
    fs::write(
        forja_root.join("tracks").join("track-mvp.md"),
        track,
    )?;

    // Generate example spec
    let spec = format!(
        r#"---
id: example
title: "Example Spec"
description: "A placeholder spec to get started"
status: draft
priority: medium
track: mvp
acceptance_criteria:
  - "Replace this with real acceptance criteria"
created: "{today}"
---
# Example Spec

Replace this file with your first real spec.

## Context

Describe the problem or feature.

## Approach

Describe the implementation approach.
"#
    );
    fs::write(forja_root.join("specs").join("example.md"), spec)?;

    // Generate INDEX.md and CONTEXT.md
    let _ = forja_core::models::index_gen::write_index(forja_root);
    let _ = forja_core::models::context_gen::write_context(forja_root);

    Ok(())
}

fn detect_stack(cwd: &Path) -> Option<String> {
    let mut components = Vec::new();

    if has_file(cwd, "next.config.js")
        || has_file(cwd, "next.config.ts")
        || has_file(cwd, "next.config.mjs")
    {
        components.push("Next.js");
    } else if has_file(cwd, "nuxt.config.ts") || has_file(cwd, "nuxt.config.js") {
        components.push("Nuxt");
    } else if has_file(cwd, "svelte.config.js") {
        components.push("SvelteKit");
    } else if has_file(cwd, "angular.json") {
        components.push("Angular");
    } else if has_file(cwd, "nest-cli.json") {
        components.push("NestJS");
    }

    if has_file(cwd, "Cargo.toml") {
        components.push("Rust");
    } else if has_file(cwd, "go.mod") {
        components.push("Go");
    } else if has_file(cwd, "tsconfig.json") {
        components.push("TypeScript");
    } else if has_file(cwd, "package.json") {
        components.push("JavaScript");
    } else if has_file(cwd, "pyproject.toml")
        || has_file(cwd, "setup.py")
        || has_file(cwd, "requirements.txt")
    {
        if has_file(cwd, "manage.py") {
            components.push("Python + Django");
        } else {
            components.push("Python");
        }
    }

    if components.is_empty() {
        None
    } else {
        Some(components.join(" + "))
    }
}

fn has_file(dir: &Path, name: &str) -> bool {
    dir.join(name).exists()
}
