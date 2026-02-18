use std::collections::HashMap;

use colored::Colorize;

use forja_core::analytics;
use forja_core::error::Result;
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::symlink::manager::load_installed_ids;

/// Show skill usage analytics and reports.
pub fn run() -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;
    let analytics_path = analytics::analytics_path(&paths.forja_root);
    let events = analytics::load(&analytics_path);

    println!("{}", "forja stats".bold());
    println!();

    if events.is_empty() {
        println!("  No usage data yet. Run some tasks to start tracking!");
        println!();
        println!(
            "  {} Usage is tracked automatically when you use {} or {}",
            "Tip:".dimmed(),
            "forja task".cyan(),
            "forja execute".cyan()
        );
        return Ok(());
    }

    println!("  {} events tracked", events.len());
    println!();

    // Most-used skills (top 10)
    let mut skill_counts: HashMap<&str, usize> = HashMap::new();
    for event in &events {
        *skill_counts.entry(&event.skill_id).or_insert(0) += 1;
    }
    let mut sorted: Vec<(&&str, &usize)> = skill_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("  {}", "Most used skills".bold().underline());
    println!();
    for (skill_id, count) in sorted.iter().take(10) {
        println!("    {:>4}x  {}", count, skill_id.cyan());
    }
    println!();

    // Usage by phase
    let mut phase_counts: HashMap<&str, usize> = HashMap::new();
    for event in &events {
        let phase = event.skill_id.split('/').next().unwrap_or("unknown");
        *phase_counts.entry(phase).or_insert(0) += 1;
    }
    let mut phase_sorted: Vec<(&&str, &usize)> = phase_counts.iter().collect();
    phase_sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("  {}", "Usage by phase".bold().underline());
    println!();
    for (phase, count) in &phase_sorted {
        println!("    {:>4}x  {}", count, phase);
    }
    println!();

    // Recent activity (last 10)
    println!("  {}", "Recent activity".bold().underline());
    println!();
    for event in events.iter().rev().take(10) {
        let short_ts = event
            .timestamp
            .split('T')
            .next()
            .unwrap_or(&event.timestamp);
        println!(
            "    {}  {}  {}",
            short_ts.dimmed(),
            event.command,
            event.skill_id.cyan()
        );
    }
    println!();

    // Installed but never used
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let used_skills: Vec<&str> = skill_counts.keys().copied().collect();
    let unused: Vec<&str> = registry
        .skills
        .iter()
        .filter(|s| s.installed)
        .filter(|s| !used_skills.contains(&s.id.as_str()))
        .map(|s| s.id.as_str())
        .collect();

    if !unused.is_empty() {
        println!("  {}", "Installed but never used".bold().underline());
        println!();
        for id in unused.iter().take(10) {
            println!("    {}", id.dimmed());
        }
        if unused.len() > 10 {
            println!("    {} more...", unused.len() - 10);
        }
        println!();
    }

    Ok(())
}
