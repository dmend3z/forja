use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::symlink::manager::save_installed_ids;
use colored::Colorize;
use serde_json::json;
use std::fs;

pub fn run(registry_url: Option<String>) -> Result<()> {
    let paths = ForjaPaths::new()?;

    if paths.forja_root.exists() {
        println!(
            "{}",
            "forja is already initialized. Use `forja update` to refresh the registry.".yellow()
        );
        return Ok(());
    }

    let url =
        registry_url.unwrap_or_else(|| "https://github.com/forja-dev/forja-skills.git".to_string());

    println!("{}", "Initializing forja...".bold());

    // Create ~/.forja/
    fs::create_dir_all(&paths.forja_root)?;
    println!("  Created {}", paths.forja_root.display());

    // For local development: symlink to the monorepo instead of cloning
    // If the current directory has a skills/ folder, link to it
    let cwd = std::env::current_dir()?;
    let local_skills = cwd.join("skills");

    if local_skills.exists() {
        std::os::unix::fs::symlink(&cwd, &paths.registry)?;
        println!(
            "  Linked registry to local: {}",
            cwd.display().to_string().cyan()
        );
    } else {
        println!("  Cloning registry from {url}...");
        crate::registry::git::clone(&url, &paths.registry)?;
        println!("  Cloned registry to {}", paths.registry.display());
    }

    // Write config
    let config = json!({
        "registry_url": url,
        "local": local_skills.exists(),
    });
    fs::write(&paths.config, serde_json::to_string_pretty(&config)?)?;
    println!("  Created config: {}", paths.config.display());

    // Create plans directory
    fs::create_dir_all(&paths.plans)?;
    println!("  Created {}", paths.plans.display());

    // Write empty state
    save_installed_ids(&paths.state, &[])?;
    println!("  Created state: {}", paths.state.display());

    // Ensure ~/.claude/agents/ exists
    fs::create_dir_all(&paths.claude_agents)?;
    println!("  Ensured {}", paths.claude_agents.display());

    println!();
    println!("{}", "forja initialized successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!("  forja phases          Show available workflow phases");
    println!("  forja list --available List all skills");
    println!("  forja install <skill> Install a skill");

    Ok(())
}
