use std::fs;
use std::process::Command;

use colored::Colorize;

use crate::error::{ForjaError, Result};
use crate::paths::ForjaPaths;

pub fn run(task: &str) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    // 1. Ensure plans dir exists
    fs::create_dir_all(&paths.plans)?;

    // 2. Read prompt template from registry skill
    let template_path = paths
        .registry
        .join("skills/research/planning/forja-plan/commands/forja-plan.md");
    let template = fs::read_to_string(&template_path)
        .map_err(|_| ForjaError::SkillNotFound("research/planning/forja-plan".into()))?;

    // 3. Strip frontmatter and replace $ARGUMENTS
    let prompt = strip_frontmatter(&template).replace("$ARGUMENTS", task);

    // 4. Check claude CLI
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    // 5. Print header and launch
    println!("{}", "forja plan".bold());
    println!();
    println!("  Task:  {}", task.cyan());
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

fn strip_frontmatter(content: &str) -> &str {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            return content[3 + end + 3..].trim_start();
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_frontmatter_removes_yaml_block() {
        let input = "---\ndescription: some desc\nargument-hint: a hint\n---\n\n# Title\n\nBody";
        let result = strip_frontmatter(input);
        assert_eq!(result, "# Title\n\nBody");
    }

    #[test]
    fn strip_frontmatter_returns_full_content_without_frontmatter() {
        let input = "# Title\n\nBody without frontmatter";
        let result = strip_frontmatter(input);
        assert_eq!(result, input);
    }

    #[test]
    fn strip_frontmatter_handles_empty_frontmatter() {
        let input = "---\n---\n\n# Title";
        let result = strip_frontmatter(input);
        assert_eq!(result, "# Title");
    }

    #[test]
    fn prompt_replaces_arguments_placeholder() {
        let template = "---\ndescription: test\n---\n\nTask: $ARGUMENTS\n\nDo the thing.";
        let prompt = strip_frontmatter(template).replace("$ARGUMENTS", "add user auth");
        assert!(prompt.contains("Task: add user auth"));
        assert!(!prompt.contains("$ARGUMENTS"));
    }
}
