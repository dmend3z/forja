use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::{SymlinkManager, load_installed_ids};
use colored::Colorize;

pub fn run() -> Result<()> {
    let paths = ForjaPaths::new()?;

    if !paths.forja_root.exists() {
        print_welcome();
        return Ok(());
    }

    print_status(&paths)
}

fn print_welcome() {
    println!();
    println!("  {}", "forja".bold());
    println!("  Skills marketplace for Claude Code");
    println!();
    println!("  Curated skills for research, coding, testing,");
    println!("  code review, and deployment — installed in seconds.");
    println!();
    println!("  Get started:");
    println!("    {}", "forja init".cyan());
    println!();
    println!("  Run {} for all commands", "forja --help".dimmed());
    println!();
}

fn print_status(paths: &ForjaPaths) -> Result<()> {
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let total = registry.skills.len();
    let installed = installed_ids.len();

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    let (_healthy, broken) = manager.verify()?;

    let health = if broken.is_empty() {
        "all symlinks OK".green().to_string()
    } else {
        format!("{} broken symlink(s)", broken.len())
            .red()
            .to_string()
    };

    println!();
    println!("  {}", "forja".bold());
    println!();
    println!("  Skills:  {}/{} installed", installed, total);
    println!("  Health:  {}", health);
    if installed < total {
        println!("  Tip:     {}", "forja install --all".cyan());
    }
    println!();
    println!("  Next: {}", "forja task \"describe your task\"".cyan());
    println!();

    Ok(())
}
