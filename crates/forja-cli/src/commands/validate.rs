use colored::Colorize;

use forja_core::error::{ForjaError, Result};
use forja_core::models::validate::{self, Severity};

use crate::output;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let forja_dir = cwd.join(".forja");

    if !forja_dir.exists() {
        return Err(ForjaError::NotInitialized);
    }

    println!("{}", "forja validate".bold());
    println!();

    let result = validate::validate_project(&forja_dir)?;

    if result.errors.is_empty() {
        output::print_success("All files valid. No errors found.");
        return Ok(());
    }

    // Print errors grouped by severity
    let errors: Vec<_> = result
        .errors
        .iter()
        .filter(|e| e.severity == Severity::Error)
        .collect();
    let warnings: Vec<_> = result
        .errors
        .iter()
        .filter(|e| e.severity == Severity::Warning)
        .collect();

    if !errors.is_empty() {
        println!("  {} ({}):", "Errors".red().bold(), errors.len());
        for e in &errors {
            println!("    {} {}: {}", "✗".red(), e.file.dimmed(), e.message);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("  {} ({}):", "Warnings".yellow().bold(), warnings.len());
        for w in &warnings {
            println!("    {} {}: {}", "!".yellow(), w.file.dimmed(), w.message);
        }
        println!();
    }

    if result.is_valid() {
        output::print_success(&format!(
            "Valid ({} warning(s))",
            result.warning_count()
        ));
        Ok(())
    } else {
        Err(ForjaError::ValidationFailed(result.error_count()))
    }
}
