use crate::error::Result;
use crate::models::config::{self, ForjaConfig};
use crate::output;
use crate::paths::{ForjaMode, ForjaPaths};
use crate::symlink::manager::save_installed_ids;
use crate::symlink::sync;
use crate::wizard;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Initialize forja with interactive wizard or `--global` shortcut.
pub fn run(registry_url: Option<String>, force_global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Check for existing .forja/ in cwd (restore flow)
    let existing_config = cwd.join(".forja").join("config.json");
    if existing_config.exists() {
        return restore_flow(&cwd, registry_url);
    }

    // Decide mode: --global flag skips wizard
    let (mode, selected_phases, _profile) = if force_global {
        (ForjaMode::Global, all_phases(), "balanced".to_string())
    } else {
        let result = wizard::run_wizard()?;
        (result.mode, result.selected_phases, result.profile)
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

    let url =
        registry_url.unwrap_or_else(|| "https://github.com/dmend3z/forja.git".to_string());

    // Create .forja/ directory
    fs::create_dir_all(&paths.forja_root)?;

    // Link or clone registry
    let local_skills = cwd.join("skills");
    let is_local = local_skills.exists();

    if is_local {
        std::os::unix::fs::symlink(&cwd, &paths.registry)?;
    } else {
        crate::registry::git::clone(&url, &paths.registry)?;
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

    // In project mode, create .gitignore
    if mode == ForjaMode::Project {
        let gitignore_path = paths.forja_root.join(".gitignore");
        fs::write(
            &gitignore_path,
            "# Managed by forja - do not edit\nregistry/\nplans/\n",
        )?;
    }

    // Install skills filtered by selected phases
    let (installed, _skipped) =
        super::install::install_by_phases(&paths, &selected_phases)?;

    // Sync symlinks to ~/.claude/
    sync::sync_symlinks(&paths)?;

    // Detect project stack
    let stack = detect_stack(&cwd);

    // Output
    println!();
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

    if mode == ForjaMode::Project {
        println!(
            "  {} .forja/ created — commit config.json and state.json to git",
            "Tip:".cyan().bold()
        );
    }

    println!();
    if let Some(ref detected) = stack {
        println!("  Detected: {}", detected.bold());
    }
    println!(
        "  Try: {}",
        "forja task \"describe your task here\"".cyan()
    );
    println!();

    output::print_tip(
        "Run 'forja doctor' to verify your setup, or 'forja guide' for a walkthrough",
    );
    Ok(())
}

fn restore_flow(cwd: &Path, registry_url: Option<String>) -> Result<()> {
    println!(
        "  {} Existing .forja/ detected in this directory",
        "Found:".cyan().bold()
    );

    let config = config::load_config(&cwd.join(".forja").join("config.json"));
    if let Some(ref cfg) = config {
        println!(
            "  Mode: {}, local: {}",
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

        let local_skills = cwd.join("skills");
        if local_skills.exists() {
            std::os::unix::fs::symlink(cwd, &paths.registry)?;
        } else {
            crate::registry::git::clone(&url, &paths.registry)?;
        }
    }

    // Sync symlinks
    sync::sync_symlinks(&paths)?;

    println!();
    output::print_success("Restored — symlinks synced to ~/.claude/");
    output::print_tip("Run 'forja doctor' to verify your setup");

    Ok(())
}

fn all_phases() -> Vec<crate::models::phase::Phase> {
    use crate::models::phase::Phase;
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
