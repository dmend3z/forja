use std::process::Command;

use colored::Colorize;

use forja_core::error::{ForjaError, Result};
use forja_core::paths::ForjaPaths;
use forja_core::symlink::auto_install;

const COMMIT_SKILL: &str = "deploy/git/commit";
const PR_SKILL: &str = "deploy/git/pr";
const CHRONICLER_SKILL: &str = "review/documentation/chronicler";

/// Commit + PR shortcut. Commits changes, then optionally creates a PR.
pub fn run(message: Option<&str>, commit_only: bool, no_chronicle: bool) -> Result<()> {
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    let changes = git_changes()?;
    if changes.is_empty() {
        return Err(ForjaError::NoChangesToReview);
    }

    let paths = ForjaPaths::ensure_initialized()?;

    let skills_needed: Vec<&str> = if commit_only {
        vec![COMMIT_SKILL]
    } else {
        vec![COMMIT_SKILL, PR_SKILL]
    };
    auto_install::auto_install_missing(&paths, &skills_needed)?;

    println!("{}", "forja ship".bold());
    println!();
    println!(
        "  Changes: {} file(s)",
        changes.lines().filter(|l| !l.is_empty()).count()
    );
    if let Some(msg) = message {
        println!("  Message: {}", msg.cyan());
    }
    println!(
        "  Mode:    {}",
        if commit_only {
            "commit only"
        } else {
            "commit + PR"
        }
    );
    println!();

    // Step 1: Commit
    let commit_symlink = format!("forja--{}", COMMIT_SKILL.replace('/', "--"));
    let mut commit_prompt = String::from("Commit the uncommitted changes in this repository.\n\n");
    commit_prompt.push_str(&format!(
        "Use the `{}` agent for the commit.\n\n",
        commit_symlink
    ));
    if let Some(msg) = message {
        commit_prompt.push_str(&format!("Commit message hint: {msg}\n\n"));
    }
    commit_prompt.push_str("## Changes\n\n");
    commit_prompt.push_str(&changes);
    commit_prompt.push_str("\n\n## Rules\n\n");
    commit_prompt.push_str("- Use conventional commits: type(scope): subject\n");
    commit_prompt.push_str("- Stage only relevant files\n");
    commit_prompt.push_str("- Do NOT push to remote\n");

    println!("{}", "Step 1: Committing changes...".bold());
    println!();

    let commit_status = Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&commit_prompt)
        .status()?;

    if !commit_status.success() {
        println!(
            "  {} Commit step failed (exit code {})",
            "Warning:".yellow().bold(),
            commit_status.code().unwrap_or(-1)
        );
        return Ok(());
    }

    if commit_only {
        println!();
        println!("{} Changes committed.", "Done:".green().bold());
        if !no_chronicle {
            run_chronicle_step(&paths)?;
        }
        return Ok(());
    }

    // Step 2: PR
    let pr_symlink = format!("forja--{}", PR_SKILL.replace('/', "--"));
    let mut pr_prompt = String::from("Create a pull request for the latest commit(s).\n\n");
    pr_prompt.push_str(&format!("Use the `{}` agent for the PR.\n\n", pr_symlink));
    pr_prompt.push_str("## Rules\n\n");
    pr_prompt.push_str("- Push to remote first if needed\n");
    pr_prompt.push_str("- Create PR with structured description\n");
    pr_prompt.push_str("- Include summary and test plan sections\n");

    println!();
    println!("{}", "Step 2: Creating PR...".bold());
    println!();

    let pr_status = Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&pr_prompt)
        .status()?;

    println!();
    println!("{} Changes committed and PR created.", "Done:".green().bold());

    if pr_status.success() && !no_chronicle {
        run_chronicle_step(&paths)?;
    }

    Ok(())
}

fn run_chronicle_step(paths: &ForjaPaths) -> Result<()> {
    auto_install::auto_install_missing(paths, &[CHRONICLER_SKILL])?;

    let symlink_name = format!("forja--{}", CHRONICLER_SKILL.replace('/', "--"));

    let mut prompt = String::from(
        "Read the recent git diff and extract any significant decisions made during this ship.\n\n",
    );
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
    fn skill_paths_are_valid() {
        assert_eq!(COMMIT_SKILL, "deploy/git/commit");
        assert_eq!(PR_SKILL, "deploy/git/pr");
    }
}
