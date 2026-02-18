use std::process::Command;

use colored::Colorize;

use forja_core::error::{ForjaError, Result};
use forja_core::paths::ForjaPaths;
use forja_core::symlink::auto_install;

const DOC_GEN_SKILL: &str = "review/documentation/doc-gen";

/// Generate project documentation by launching Claude Code with the doc-gen agent.
pub fn run(scope: Option<&str>) -> Result<()> {
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    let paths = ForjaPaths::ensure_initialized()?;

    auto_install::auto_install_missing(&paths, &[DOC_GEN_SKILL])?;

    let symlink_name = format!("forja--{}", DOC_GEN_SKILL.replace('/', "--"));

    let mut prompt = String::from("Generate or update project documentation.\n\n");
    prompt.push_str(&format!(
        "Use the `{}` agent for this task.\n\n",
        symlink_name
    ));

    if let Some(s) = scope {
        let target = match s {
            "claude-md" => "CLAUDE.md",
            "agents-md" => "AGENTS.md",
            "readme" => "README.md",
            other => {
                return Err(ForjaError::InvalidArgument(format!(
                    "unknown scope '{other}' — valid values: claude-md, agents-md, readme"
                )));
            }
        };
        prompt.push_str(&format!("Generate only: {target}\n\n"));
    } else {
        prompt.push_str("Generate all three docs: CLAUDE.md, AGENTS.md, README.md\n\n");
    }

    prompt.push_str("## Rules\n\n");
    prompt.push_str("- Read CLAUDE.md first if it exists\n");
    prompt.push_str("- Read before writing — ground every claim in what you actually read\n");
    prompt.push_str("- Preserve custom content in existing docs\n");
    prompt.push_str("- Self-review: reread output and run git diff before finishing\n");

    let scope_label = scope.unwrap_or("all");

    println!("{}", "forja docs".bold());
    println!();
    println!("  Scope: {scope_label}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_gen_skill_path_is_valid() {
        assert_eq!(DOC_GEN_SKILL, "review/documentation/doc-gen");
    }

    #[test]
    fn symlink_name_format() {
        let name = format!("forja--{}", DOC_GEN_SKILL.replace('/', "--"));
        assert_eq!(name, "forja--review--documentation--doc-gen");
    }
}
