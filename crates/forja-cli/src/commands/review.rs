use std::process::Command;

use colored::Colorize;

use forja_core::error::{ForjaError, Result};
use forja_core::paths::ForjaPaths;
use forja_core::symlink::auto_install;

const REVIEWER_SKILL: &str = "review/code-quality/reviewer";
const CHRONICLER_SKILL: &str = "review/documentation/chronicler";

/// Review uncommitted changes by launching Claude Code with the reviewer agent.
pub fn run(path_filter: Option<&str>, no_chronicle: bool) -> Result<()> {
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    let changes = git_changes()?;
    if changes.is_empty() {
        return Err(ForjaError::NoChangesToReview);
    }

    let paths = ForjaPaths::ensure_initialized()?;

    auto_install::auto_install_missing(&paths, &[REVIEWER_SKILL])?;

    let symlink_name = format!("forja--{}", REVIEWER_SKILL.replace('/', "--"));

    let mut prompt = String::from("Review the uncommitted changes in this repository.\n\n");
    prompt.push_str(&format!(
        "Use the `{}` agent for the review.\n\n",
        symlink_name
    ));

    if let Some(filter) = path_filter {
        prompt.push_str(&format!("Focus the review on files matching: {filter}\n\n"));
    }

    prompt.push_str("## Changes detected\n\n");
    prompt.push_str(&changes);
    prompt.push_str("\n\n## Rules\n\n");
    prompt.push_str("- Read CLAUDE.md before starting\n");
    prompt.push_str("- Check for security issues (OWASP Top 10)\n");
    prompt.push_str("- Check for performance issues\n");
    prompt.push_str("- Every finding needs a specific fix example\n");

    println!("{}", "forja review".bold());
    println!();
    println!(
        "  Changes: {} file(s)",
        changes.lines().filter(|l| !l.is_empty()).count()
    );
    println!();
    println!("{}", "Launching Claude Code session...".bold());
    println!();

    let review_status = Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&prompt)
        .status()?;

    if review_status.success() && !no_chronicle {
        run_chronicle_step(&paths)?;
    }

    Ok(())
}

fn run_chronicle_step(paths: &ForjaPaths) -> Result<()> {
    auto_install::auto_install_missing(paths, &[CHRONICLER_SKILL])?;

    let symlink_name = format!("forja--{}", CHRONICLER_SKILL.replace('/', "--"));

    let mut prompt =
        String::from("Read the recent git diff and extract any significant decisions made during this review.\n\n");
    prompt.push_str(&format!(
        "Use the `{}` agent for this task.\n\n",
        symlink_name
    ));
    prompt.push_str("- Write decisions to docs/decisions/YYYY-MM-DD-{slug}.md\n");
    prompt.push_str("- Only document real decisions, skip routine details\n");
    prompt.push_str("- Create the docs/decisions/ directory if it doesn't exist\n");

    println!();
    println!("{}", "Chronicling decisions...".bold());
    println!();

    Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&prompt)
        .status()?;

    Ok(())
}

fn git_changes() -> Result<String> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .output()
        .map_err(ForjaError::Io)?;

    if !output.status.success() {
        return Err(ForjaError::Git(
            "git status failed — are you in a git repository?".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewer_skill_path_is_valid() {
        assert_eq!(REVIEWER_SKILL, "review/code-quality/reviewer");
    }
}
