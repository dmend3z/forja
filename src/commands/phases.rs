use crate::error::Result;
use crate::models::phase::Phase;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::load_installed_ids;
use colored::Colorize;

/// Display the 5 workflow phases and their available skills.
pub fn run() -> Result<()> {
    println!("{}", "forja workflow phases".bold());
    println!();

    // Try to show catalog data if initialized
    let paths = ForjaPaths::new().ok();
    let registry = paths.as_ref().and_then(|p| {
        if p.forja_root.exists() {
            let ids = load_installed_ids(&p.state);
            catalog::scan(&p.registry, &ids).ok()
        } else {
            None
        }
    });

    for (i, phase) in Phase::all().iter().enumerate() {
        let num = if *phase == Phase::Teams {
            " ".to_string()
        } else {
            format!("{}", i + 1)
        };

        let name = format!("{num}. {}", phase.as_str().to_uppercase());
        let desc = phase.description();

        let count = registry
            .as_ref()
            .map(|r| {
                let n = r.skills.iter().filter(|s| s.phase == *phase).count();
                format!(" ({n} skills)")
            })
            .unwrap_or_default();

        println!(
            "  {}{}  {}",
            name.bold().cyan(),
            count.dimmed(),
            desc.dimmed()
        );
    }

    println!();
    println!(
        "{}",
        "Use `forja list --available` to see skills per phase".dimmed()
    );
    Ok(())
}
