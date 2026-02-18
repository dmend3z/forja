use std::process::Command;

use colored::Colorize;

use crate::error::{ForjaError, Result};
use crate::paths::ForjaPaths;
use crate::symlink::auto_install;

const CHRONICLER_SKILL: &str = "review/documentation/chronicler";

/// Document decisions by launching Claude Code with the chronicler agent.
pub fn run(from_files: &[String]) -> Result<()> {
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    let paths = ForjaPaths::ensure_initialized()?;

    auto_install::auto_install_missing(&paths, &[CHRONICLER_SKILL])?;

    let symlink_name = format!("forja--{}", CHRONICLER_SKILL.replace('/', "--"));

    let context = if from_files.is_empty() {
        gather_git_context()?
    } else {
        format!("Files to analyze:\n{}", from_files.join("\n"))
    };

    let mut prompt = String::from(
        "Extract and document any significant decisions from the context below.\n\n",
    );
    prompt.push_str(&format!(
        "Use the `{}` agent for this task.\n\n",
        symlink_name
    ));
    prompt.push_str("## Context\n\n");
    prompt.push_str(&context);
    prompt.push_str("\n\n## Rules\n\n");
    prompt.push_str("- Write decisions to docs/decisions/YYYY-MM-DD-{slug}.md\n");
    prompt.push_str("- Only document real decisions (approach choices, trade-offs, rejections)\n");
    prompt.push_str("- Skip routine implementation details\n");
    prompt.push_str("- Create the docs/decisions/ directory if it doesn't exist\n");

    println!("{}", "forja chronicle".bold());
    println!();
    if from_files.is_empty() {
        println!("  Source: recent git changes");
    } else {
        println!("  Source: {} file(s)", from_files.len());
    }
    println!();
    println!("{}", "Launching Claude Code session...".bold());
    println!();

    Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&prompt)
        .status()?;

    Ok(())
}

fn gather_git_context() -> Result<String> {
    let log = Command::new("git")
        .args(["log", "--oneline", "-10"])
        .output()
        .map_err(ForjaError::Io)?;

    let diff = Command::new("git")
        .args(["diff", "HEAD~1"])
        .output()
        .map_err(ForjaError::Io)?;

    let log_str = String::from_utf8_lossy(&log.stdout);
    let diff_str = String::from_utf8_lossy(&diff.stdout);

    let mut context = String::new();
    if !log_str.is_empty() {
        context.push_str("### Recent commits\n\n");
        context.push_str(&log_str);
        context.push_str("\n\n");
    }
    if !diff_str.is_empty() {
        context.push_str("### Recent diff\n\n");
        context.push_str(&diff_str);
    }

    if context.is_empty() {
        context.push_str("No recent git changes found.");
    }

    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chronicler_skill_path_is_valid() {
        assert_eq!(CHRONICLER_SKILL, "review/documentation/chronicler");
    }
}
