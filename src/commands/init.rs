use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::symlink::manager::save_installed_ids;
use colored::Colorize;
use serde_json::json;
use std::fs;
use std::path::Path;

/// Initialize forja: create `~/.forja/`, clone the registry, and set up state.
pub fn run(registry_url: Option<String>) -> Result<()> {
    let paths = ForjaPaths::new()?;

    if paths.forja_root.exists() {
        println!(
            "{}",
            "forja is already initialized. Use `forja update` to refresh the registry.".yellow()
        );
        return Ok(());
    }

    let url = registry_url.unwrap_or_else(|| "https://github.com/dmend3z/forja.git".to_string());

    // Create ~/.forja/
    fs::create_dir_all(&paths.forja_root)?;

    // Link or clone registry
    let cwd = std::env::current_dir()?;
    let local_skills = cwd.join("skills");

    if local_skills.exists() {
        std::os::unix::fs::symlink(&cwd, &paths.registry)?;
    } else {
        crate::registry::git::clone(&url, &paths.registry)?;
    }

    // Write config
    let config = json!({
        "registry_url": url,
        "local": local_skills.exists(),
    });
    fs::write(&paths.config, serde_json::to_string_pretty(&config)?)?;

    // Create plans directory
    fs::create_dir_all(&paths.plans)?;

    // Write empty state
    save_installed_ids(&paths.state, &[])?;

    // Ensure ~/.claude/agents/ exists
    fs::create_dir_all(&paths.claude_agents)?;

    // Auto-install all skills
    let (installed, _skipped) = super::install::install_all_quiet(&paths)?;

    // Detect project stack
    let stack = detect_stack(&cwd);

    // Minimal output
    println!();
    println!("  {} forja initialized", "✓".green());
    println!(
        "  {} {} skills installed (research, code, test, review, deploy)",
        "✓".green(),
        installed
    );

    println!();
    if let Some(ref detected) = stack {
        println!("  Detected: {}", detected.bold());
    }
    println!("  Try: {}", "forja task \"describe your task here\"".cyan());
    println!();

    Ok(())
}

fn detect_stack(cwd: &Path) -> Option<String> {
    let mut components = Vec::new();

    // Framework detection (order: most specific first)
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

    // Language / runtime detection
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
