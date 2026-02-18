use std::collections::HashMap;
use std::process::Command;

use colored::Colorize;
use dialoguer::Select;

use forja_core::error::{ForjaError, Result};
use forja_core::models::profile::Profile;
use forja_core::models::state::{TeamEntry, TeamMember, load_state};
use forja_core::paths::ForjaPaths;
use forja_core::settings;
use forja_core::symlink::auto_install;

const PRESET_TEAMS: &[(&str, &str)] = &[
    ("quick-fix", "coder + deployer"),
    ("solo-sprint", "coder-tester + reviewer"),
    ("full-product", "5 agents"),
];

/// Run a task directly in Claude Code, optionally with a multi-agent team.
pub fn run(task: Option<&str>, print: bool, team: Option<&str>, profile: Option<&str>) -> Result<()> {
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    match task {
        Some(t) => run_with_task(t, print, team, profile),
        None => {
            if print {
                return Err(ForjaError::Dialoguer(
                    "--print requires a task description".to_string(),
                ));
            }
            let output = forja_tui::launch()?;
            match output {
                Some(o) => match o.team {
                    Some(ref team_name) => run_team(
                        &o.description,
                        false,
                        team_name,
                        Some(o.profile.as_str()),
                    ),
                    None => run_simple(&o.description, false),
                },
                None => Ok(()), // user pressed Esc
            }
        }
    }
}

fn run_with_task(task: &str, print: bool, team: Option<&str>, profile: Option<&str>) -> Result<()> {
    match team {
        // Explicit --team flag → use team directly (scripting path)
        Some(team_name) => run_team(task, print, team_name, profile),
        None => {
            // --print without --team → solo (non-interactive)
            if print {
                return run_simple(task, true);
            }

            // Interactive: offer team picker if forja is initialized
            match prompt_team_selection()? {
                None => run_simple(task, false),
                Some(team_name) => run_team(task, false, &team_name, profile),
            }
        }
    }
}

fn prompt_team_selection() -> Result<Option<String>> {
    let paths = match ForjaPaths::new() {
        Ok(p) if p.forja_root.exists() => p,
        _ => return Ok(None),
    };

    let state = load_state(&paths.state);
    let options = build_team_options(&state.teams);

    // "Solo (single agent)" is always the first/default option
    let mut labels: Vec<String> = vec!["Solo (single agent)".to_string()];
    labels.extend(options.iter().map(|(label, _)| label.clone()));

    let selection = Select::new()
        .with_prompt("How would you like to run this task?")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    if selection == 0 {
        return Ok(None);
    }

    Ok(Some(options[selection - 1].1.clone()))
}

fn build_team_options(teams: &HashMap<String, TeamEntry>) -> Vec<(String, String)> {
    let mut options: Vec<(String, String)> = Vec::new();

    // Presets first (skip if already configured as a custom team)
    for &(name, description) in PRESET_TEAMS {
        if teams.contains_key(name) {
            // Show the configured version instead (will be added below)
            continue;
        }
        options.push((format!("{} ({})", name, description), name.to_string()));
    }

    // Configured teams (sorted for stable ordering)
    let mut configured: Vec<(&String, &TeamEntry)> = teams.iter().collect();
    configured.sort_by_key(|(name, _)| name.to_string());

    for (name, entry) in configured {
        let label = format!("{} ({} agents)", name, entry.members.len());
        options.push((label, name.clone()));
    }

    options
}

fn run_simple(task: &str, print: bool) -> Result<()> {
    println!("{}", "forja task".bold());
    println!();
    println!("  Task:  {}", task.cyan());
    println!();
    println!("{}", "Launching Claude Code session...".bold());
    println!();

    let mut cmd = Command::new("claude");
    cmd.arg("--dangerously-skip-permissions");
    if print {
        cmd.arg("--print");
    }
    cmd.arg("--").arg(task);
    cmd.status()?;

    Ok(())
}

fn parse_profile(s: &str) -> Result<Profile> {
    s.parse().map_err(|_| {
        ForjaError::Dialoguer(format!("Unknown profile '{}'. Use: fast, balanced, max", s))
    })
}

fn run_team(
    task: &str,
    print: bool,
    team_name: &str,
    profile_override: Option<&str>,
) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;
    let state = load_state(&paths.state);

    // Track usage analytics for team members
    let analytics_path = forja_core::analytics::analytics_path(&paths.forja_root);
    if let Some(entry) = state.teams.get(team_name) {
        for member in &entry.members {
            let _ = forja_core::analytics::track(&analytics_path, &member.skill_id, "task");
        }
    }

    // Resolve team members and profile: check state first, then try preset fallback
    let (members, profile) = if let Some(entry) = state.teams.get(team_name) {
        let profile_str = profile_override.unwrap_or(&entry.profile);
        (entry.members.clone(), parse_profile(profile_str)?)
    } else {
        let profile = parse_profile(profile_override.unwrap_or("balanced"))?;
        let members = crate::commands::team::resolve_preset_members(team_name, &profile)?;
        (members, profile)
    };

    println!("{}", "forja task (team mode)".bold());
    println!();
    println!("  Task:    {}", task.cyan());
    println!("  Team:    {}", team_name.cyan());
    println!("  Profile: {}", profile.as_str().cyan());
    println!();

    // Ensure agent teams env var (always in ~/.claude/settings.json)
    let global_claude = forja_core::paths::ForjaPaths::global_claude_dir()?;
    if !settings::has_teams_env_var(&global_claude) {
        settings::enable_teams_env_var(&global_claude)?;
        println!(
            "  {} Agent teams env var enabled in settings.json",
            "NOTE:".yellow().bold()
        );
        println!();
    }

    // Auto-install missing agent symlinks
    auto_install_agents(&paths, &members)?;

    // Build team prompt
    let prompt = build_team_prompt(task, team_name, profile.as_str(), &members);

    println!("{}", "Launching Claude Code session...".bold());
    println!();

    let mut cmd = Command::new("claude");
    cmd.arg("--dangerously-skip-permissions");
    if print {
        cmd.arg("--print");
    }
    cmd.arg("--").arg(&prompt);
    cmd.status()?;

    Ok(())
}

fn auto_install_agents(paths: &ForjaPaths, members: &[TeamMember]) -> Result<()> {
    let skill_ids: Vec<&str> = members.iter().map(|m| m.skill_id.as_str()).collect();
    auto_install::auto_install_missing(paths, &skill_ids)?;
    Ok(())
}

fn build_team_prompt(task: &str, team_name: &str, profile: &str, members: &[TeamMember]) -> String {
    let mut prompt = String::new();

    prompt.push_str("Execute this task as a team orchestrator.\n\n");

    prompt.push_str("## Task\n\n");
    prompt.push_str(task);
    prompt.push_str("\n\n");

    prompt.push_str("## Team Configuration\n\n");
    prompt.push_str(&format!("Team: {}\n", team_name));
    prompt.push_str(&format!("Profile: {}\n\n", profile));

    prompt.push_str("Spawn these agents as teammates:\n\n");
    for member in members {
        let symlink_name = format!("forja--{}", member.skill_id.replace('/', "--"));
        prompt.push_str(&format!(
            "- **{}**: use the `{}` agent\n",
            member.agent_name, symlink_name
        ));
    }

    prompt.push_str("\n## Execution Rules\n\n");
    prompt.push_str("- Read CLAUDE.md before starting implementation\n");
    prompt.push_str("- Pass results between phases as context\n");
    prompt.push_str("- Stop and report if an agent is blocked or encounters errors\n");
    prompt.push_str("- Use delegate mode for efficiency\n");

    prompt
}

#[cfg(test)]
fn build_args(task: &str, print: bool) -> Vec<&str> {
    let mut args = Vec::new();
    if print {
        args.push("--print");
    }
    args.push("--");
    args.push(task);
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_args_basic() {
        let args = build_args("fix the bug", false);
        assert_eq!(args, vec!["--", "fix the bug"]);
    }

    #[test]
    fn build_args_with_print() {
        let args = build_args("fix the bug", true);
        assert_eq!(args, vec!["--print", "--", "fix the bug"]);
    }

    fn test_members() -> Vec<TeamMember> {
        vec![
            TeamMember {
                skill_id: "code/general/feature".to_string(),
                agent_name: "coder".to_string(),
                model: "sonnet".to_string(),
            },
            TeamMember {
                skill_id: "deploy/git/commit".to_string(),
                agent_name: "deployer".to_string(),
                model: "sonnet".to_string(),
            },
        ]
    }

    #[test]
    fn build_team_prompt_contains_task() {
        let members = test_members();
        let prompt = build_team_prompt("fix the login bug", "quick-fix", "balanced", &members);
        assert!(prompt.contains("fix the login bug"));
        assert!(prompt.contains("## Task"));
    }

    #[test]
    fn build_team_prompt_contains_agents() {
        let members = test_members();
        let prompt = build_team_prompt("fix bug", "quick-fix", "balanced", &members);
        assert!(prompt.contains("**coder**"));
        assert!(prompt.contains("**deployer**"));
        assert!(prompt.contains("forja--code--general--feature"));
        assert!(prompt.contains("forja--deploy--git--commit"));
    }

    #[test]
    fn build_team_prompt_contains_rules() {
        let members = test_members();
        let prompt = build_team_prompt("fix bug", "quick-fix", "balanced", &members);
        assert!(prompt.contains("## Execution Rules"));
        assert!(prompt.contains("CLAUDE.md"));
        assert!(prompt.contains("delegate mode"));
    }

    #[test]
    fn build_team_options_empty_state() {
        let teams = HashMap::new();
        let options = build_team_options(&teams);
        assert_eq!(options.len(), 3);
        assert_eq!(options[0].1, "quick-fix");
        assert_eq!(options[1].1, "solo-sprint");
        assert_eq!(options[2].1, "full-product");
        // Labels include descriptions
        assert!(options[0].0.contains("coder + deployer"));
    }

    #[test]
    fn build_team_options_with_configured_team() {
        let mut teams = HashMap::new();
        teams.insert(
            "my-custom".to_string(),
            TeamEntry {
                members: vec![
                    TeamMember {
                        skill_id: "code/general/feature".to_string(),
                        agent_name: "coder".to_string(),
                        model: "sonnet".to_string(),
                    },
                    TeamMember {
                        skill_id: "test/tdd/workflow".to_string(),
                        agent_name: "tester".to_string(),
                        model: "sonnet".to_string(),
                    },
                    TeamMember {
                        skill_id: "review/code-quality/reviewer".to_string(),
                        agent_name: "reviewer".to_string(),
                        model: "sonnet".to_string(),
                    },
                ],
                profile: "balanced".to_string(),
            },
        );
        let options = build_team_options(&teams);
        // 3 presets + 1 configured = 4
        assert_eq!(options.len(), 4);
        // Configured team appears after presets
        let names: Vec<&str> = options.iter().map(|(_, n)| n.as_str()).collect();
        assert!(names.contains(&"my-custom"));
        assert!(options.last().unwrap().0.contains("3 agents"));
    }

    #[test]
    fn build_team_options_preset_configured_no_duplicate() {
        let mut teams = HashMap::new();
        teams.insert(
            "quick-fix".to_string(),
            TeamEntry {
                members: vec![TeamMember {
                    skill_id: "code/general/feature".to_string(),
                    agent_name: "coder".to_string(),
                    model: "sonnet".to_string(),
                }],
                profile: "fast".to_string(),
            },
        );
        let options = build_team_options(&teams);
        // quick-fix is configured, so only 2 presets + 1 configured = 3
        assert_eq!(options.len(), 3);
        // quick-fix should appear once (as configured, not preset)
        let qf_count = options.iter().filter(|(_, n)| n == "quick-fix").count();
        assert_eq!(qf_count, 1);
        // The configured version shows agent count, not preset description
        let qf = options.iter().find(|(_, n)| n == "quick-fix").unwrap();
        assert!(qf.0.contains("1 agents"));
    }
}
