use std::path::Path;

use colored::Colorize;

use forja_core::error::{ForjaError, Result};
use forja_core::lint;
use forja_core::models::lint::LintLevel;
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::symlink::manager::load_installed_ids;

/// Validate skill structure and manifest integrity.
///
/// If `path` is given, lint that specific skill directory.
/// Otherwise, lint all skills in the registry.
pub fn run(path: Option<&str>, show_warnings: bool) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let results = match path {
        Some(p) => lint_single(Path::new(p))?,
        None => lint_all(&paths)?,
    };

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_pass = 0;

    for result in &results {
        let errors = result.error_count();
        let warnings = result.warning_count();
        total_errors += errors;
        total_warnings += warnings;

        if errors == 0 && warnings == 0 {
            total_pass += 1;
            continue;
        }

        if errors == 0 && !show_warnings {
            total_pass += 1;
            continue;
        }

        // Print skill header
        let status = if errors > 0 {
            "FAIL".red().bold().to_string()
        } else {
            "WARN".yellow().bold().to_string()
        };
        println!("  {} {}", status, result.skill_id);

        for issue in &result.issues {
            if issue.level == LintLevel::Warning && !show_warnings {
                continue;
            }
            let prefix = match issue.level {
                LintLevel::Error => "error".red().to_string(),
                LintLevel::Warning => "warn".yellow().to_string(),
            };
            println!("    {} [{}] {}", prefix, issue.rule.dimmed(), issue.message);
        }
        println!();
    }

    // Summary
    println!(
        "  {} {} skills: {} passed, {} error(s), {} warning(s)",
        "Summary:".bold(),
        results.len(),
        total_pass.to_string().green(),
        total_errors.to_string().red(),
        total_warnings.to_string().yellow(),
    );

    if !show_warnings && total_warnings > 0 {
        println!(
            "  {} Use {} to see warnings",
            "Tip:".dimmed(),
            "--warnings".cyan()
        );
    }

    if total_errors > 0 {
        return Err(ForjaError::LintFailed(total_errors));
    }

    Ok(())
}

fn lint_single(path: &Path) -> Result<Vec<forja_core::models::lint::LintResult>> {
    if !path.exists() {
        return Err(ForjaError::SkillNotFound(
            path.display().to_string(),
        ));
    }

    let id = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(vec![lint::lint_skill(path, &id)])
}

fn lint_all(paths: &ForjaPaths) -> Result<Vec<forja_core::models::lint::LintResult>> {
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;

    println!("{}", "forja lint".bold());
    println!();
    println!("  Scanning {} skills...", registry.skills.len());
    println!();

    let results: Vec<_> = registry
        .skills
        .iter()
        .map(|skill| lint::lint_skill(&skill.path, &skill.id))
        .collect();

    Ok(results)
}
