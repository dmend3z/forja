use std::fs;
use std::path::Path;

use colored::Colorize;
use dialoguer::{Confirm, MultiSelect, Select};

use crate::error::{ForjaError, Result};
use crate::models::phase::Phase;
use crate::models::profile::Profile;
use crate::models::skill::Skill;
use crate::models::state::{TeamEntry, TeamMember, load_state, save_state};
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::load_installed_ids;

// ── Frontmatter parsing ──────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct AgentFrontmatter {
    name: String,
    description: String,
    tools: String,
    #[allow(dead_code)]
    model: String,
    body: String,
}

pub(crate) fn parse_agent_md(path: &Path) -> Option<AgentFrontmatter> {
    let content = fs::read_to_string(path).ok()?;
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    let end = after_first.find("---")?;
    let frontmatter = &after_first[..end];
    let body = after_first[end + 3..].trim().to_string();

    let mut name = String::new();
    let mut description = String::new();
    let mut tools = String::new();
    let mut model = String::new();

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("name:") {
            name = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("description:") {
            description = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("tools:") {
            tools = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("model:") {
            model = val.trim().to_string();
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(AgentFrontmatter {
        name,
        description,
        tools,
        model,
        body,
    })
}

fn find_first_agent(skill: &Skill) -> Option<AgentFrontmatter> {
    let agents_dir = skill.path.join("agents");
    if !agents_dir.exists() {
        return None;
    }
    let mut entries: Vec<_> = fs::read_dir(&agents_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort();
    entries.into_iter().find_map(|p| parse_agent_md(&p))
}

// ── Environment check ────────────────────────────────────────────

fn ensure_teams_env_var(claude_dir: &std::path::Path) -> Result<()> {
    use crate::settings;

    if settings::has_teams_env_var(claude_dir) {
        return Ok(());
    }

    println!(
        "{} Agent teams require {} in settings.json.",
        "NOTE:".yellow().bold(),
        "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1".cyan()
    );

    let enable = Confirm::new()
        .with_prompt("Enable it now?")
        .default(true)
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    if enable {
        settings::enable_teams_env_var(claude_dir)?;
        println!(
            "  {} Env var added to ~/.claude/settings.json",
            "Done:".green().bold()
        );
    } else {
        println!(
            "  Add it manually: {}",
            r#"{ "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" } }"#.dimmed()
        );
    }
    println!();

    Ok(())
}

// ── Slash command generation ─────────────────────────────────────

pub(crate) fn generate_slash_command(
    team_name: &str,
    members: &[(AgentFrontmatter, Phase, &str)], // (agent, phase, resolved_model)
) -> String {
    let mut out = String::new();

    // Frontmatter
    out.push_str("---\n");
    out.push_str(&format!("description: Launch the {} team\n", team_name));
    out.push_str("argument-hint: Task description\n");
    out.push_str("---\n\n");

    // Title
    out.push_str(&format!("# Team: {}\n\n", team_name));
    out.push_str("## Team Structure\n\n");

    // Members
    for (i, (agent, phase, model)) in members.iter().enumerate() {
        out.push_str(&format!(
            "### {}. {} (Phase: {})\n",
            i + 1,
            capitalize(&agent.name),
            phase.as_str().to_uppercase()
        ));
        out.push_str(&format!(
            "Spawn a **{}** teammate with this prompt:\n\n",
            agent.name
        ));

        // Use the first paragraph of the body as prompt, or description if body is short
        let prompt = if agent.body.is_empty() {
            &agent.description
        } else {
            &agent.body
        };
        out.push_str(&format!("\"{}\"\n\n", first_paragraph(prompt)));
        out.push_str(&format!("Tools: {}\n", agent.tools));
        out.push_str(&format!("Model: {}\n\n", model));
    }

    // Orchestration — ordered by phase priority
    out.push_str("## Orchestration\n\n");

    let phase_order: &[Phase] = &[
        Phase::Research,
        Phase::Code,
        Phase::Test,
        Phase::Review,
        Phase::Deploy,
    ];

    let mut step = 1;
    for &phase in phase_order {
        let phase_members: Vec<_> = members.iter().filter(|(_, p, _)| *p == phase).collect();
        if phase_members.is_empty() {
            continue;
        }

        let parallel_hint = if phase_members.len() > 1 {
            " (in parallel)"
        } else {
            ""
        };

        for (agent, _, _) in &phase_members {
            out.push_str(&format!(
                "{}. Start the **{}**{}\n",
                step,
                capitalize(&agent.name),
                parallel_hint
            ));
            step += 1;
        }
    }

    // Tips
    out.push_str("\n## Tips\n\n");
    out.push_str("- Use delegate mode (Shift+Tab) to keep the lead focused on orchestration\n");
    out.push_str(
        "- Each teammate loads CLAUDE.md automatically — keep it concise and operational\n",
    );
    out.push_str("- Teammates communicate via messages + shared task list, not conversation\n");

    // Troubleshooting
    out.push_str("\n## Troubleshooting\n\n");
    out.push_str("- If teammates don't spawn: verify `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in settings.json\n");
    out.push_str(
        "- If agents can't find files: check CLAUDE.md has project structure documented\n",
    );
    out.push_str("- Run `forja doctor` to verify installation health\n");

    out
}

fn first_paragraph(text: &str) -> String {
    text.split("\n\n")
        .next()
        .unwrap_or(text)
        .lines()
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

pub(crate) fn command_file_name(team_name: &str) -> String {
    format!("forja--team--{}.md", team_name)
}

// ── Subcommands ──────────────────────────────────────────────────

/// Interactively create a custom team by selecting installed skills and a model profile.
pub fn create(name: &str) -> Result<()> {
    let paths = ForjaPaths::new()?;
    let mut state = load_state(&paths.state);

    if state.teams.contains_key(name) {
        return Err(ForjaError::TeamAlreadyExists(name.to_string()));
    }

    ensure_teams_env_var(&paths.claude_dir)?;

    // Scan catalog for installed skills (exclude Teams phase)
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let installed_skills: Vec<&Skill> = registry
        .skills
        .iter()
        .filter(|s| s.installed && s.phase != Phase::Teams)
        .collect();

    if installed_skills.is_empty() {
        eprintln!(
            "{} No installed skills found. Install skills first with `forja install`.",
            "Error:".red().bold()
        );
        return Ok(());
    }

    // Build display labels for MultiSelect
    let labels: Vec<String> = installed_skills
        .iter()
        .map(|s| {
            format!(
                "{} ({}) — {}",
                s.name,
                s.phase.as_str().to_uppercase(),
                truncate(&s.description, 50)
            )
        })
        .collect();

    // Pick agents
    println!("{}", "Create a custom team".bold());
    println!();

    let selected_indices = MultiSelect::new()
        .with_prompt("Select agents for your team")
        .items(&labels)
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    if selected_indices.is_empty() {
        return Err(ForjaError::PromptCancelled);
    }

    // Pick profile
    let profile_labels: Vec<String> = Profile::all()
        .iter()
        .map(|p| format!("{} — {}", p.as_str(), p.description()))
        .collect();

    let profile_idx = Select::new()
        .with_prompt("Select model profile")
        .items(&profile_labels)
        .default(1) // balanced
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    let profile = Profile::all()[profile_idx];

    // Resolve agents and models
    let mut members_display: Vec<(AgentFrontmatter, Phase, String)> = Vec::new();

    for &idx in &selected_indices {
        let skill = installed_skills[idx];
        if let Some(agent) = find_first_agent(skill) {
            let model = profile.resolve_model(skill.phase).to_string();
            members_display.push((agent, skill.phase, model));
        }
    }

    if members_display.is_empty() {
        eprintln!(
            "{} Selected skills have no agent .md files.",
            "Error:".red().bold()
        );
        return Ok(());
    }

    // Show summary
    println!();
    println!("{}", format!("Team: {name}").bold());
    println!("Profile: {}", profile.as_str().cyan());
    println!();
    for (agent, phase, model) in &members_display {
        println!(
            "  {} ({}) → model: {}",
            agent.name.bold(),
            phase.as_str().to_uppercase().dimmed(),
            model.cyan()
        );
    }
    println!();

    let confirmed = Confirm::new()
        .with_prompt("Create this team?")
        .default(true)
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    if !confirmed {
        return Err(ForjaError::PromptCancelled);
    }

    // Build members for state
    let team_members: Vec<TeamMember> = selected_indices
        .iter()
        .zip(members_display.iter())
        .map(|(&idx, (agent, _, model))| TeamMember {
            skill_id: installed_skills[idx].id.clone(),
            agent_name: agent.name.clone(),
            model: model.clone(),
        })
        .collect();

    // Generate slash command
    let cmd_refs: Vec<(AgentFrontmatter, Phase, &str)> = members_display
        .iter()
        .map(|(a, p, m)| (a.clone(), *p, m.as_str()))
        .collect();

    let command_md = generate_slash_command(name, &cmd_refs);

    fs::create_dir_all(&paths.claude_commands)?;
    let cmd_path = paths.claude_commands.join(command_file_name(name));
    fs::write(&cmd_path, &command_md)?;

    // Save to state
    state.teams.insert(
        name.to_string(),
        TeamEntry {
            members: team_members,
            profile: profile.as_str().to_string(),
        },
    );
    save_state(&paths.state, &state)?;

    println!();
    println!(
        "{} Team {} created!",
        "SUCCESS:".green().bold(),
        name.bold()
    );
    println!(
        "  Slash command: {}",
        format!("/forja--team--{name}").cyan()
    );
    println!(
        "  Command file: {}",
        cmd_path.display().to_string().dimmed()
    );

    Ok(())
}

/// Create a team from a built-in preset (full-product, solo-sprint, quick-fix).
pub fn preset(name: &str, profile_str: &str) -> Result<()> {
    let paths = ForjaPaths::new()?;
    let mut state = load_state(&paths.state);

    let profile: Profile = profile_str.parse().map_err(|_| {
        ForjaError::Dialoguer(format!(
            "Unknown profile '{}'. Use: fast, balanced, max",
            profile_str
        ))
    })?;

    ensure_teams_env_var(&paths.claude_dir)?;

    // Map preset name to skill directory
    let preset_mapping: &[(&str, &str)] = &[
        ("full-product", "teams/full-product/team"),
        ("solo-sprint", "teams/solo-sprint/team"),
        ("quick-fix", "teams/quick-fix/team"),
        ("dispatch", "teams/dispatch/team"),
        ("tech-council", "teams/technical-council/team"),
        ("biz-council", "teams/strategic-council/team"),
    ];

    let skill_id = preset_mapping
        .iter()
        .find(|(preset, _)| *preset == name)
        .map(|(_, id)| *id)
        .ok_or_else(|| {
            ForjaError::SkillNotFound(format!(
                "Unknown preset '{}'. Available: full-product, solo-sprint, quick-fix, dispatch, tech-council, biz-council",
                name
            ))
        })?;

    // Read the existing command .md from the preset
    let cmd_source = paths
        .registry
        .join("skills")
        .join(skill_id.replace('/', std::path::MAIN_SEPARATOR_STR))
        .join("commands");

    let source_md = find_first_md(&cmd_source)?;
    let content = fs::read_to_string(&source_md)?;

    // Apply profile overrides: replace Model: lines
    let modified = apply_profile_to_command(&content, &profile);

    // Write to commands dir
    fs::create_dir_all(&paths.claude_commands)?;
    let cmd_path = paths.claude_commands.join(command_file_name(name));
    fs::write(&cmd_path, &modified)?;

    // Build team members from the preset
    let members = resolve_preset_members(name, &profile)?;

    state.teams.insert(
        name.to_string(),
        TeamEntry {
            members,
            profile: profile.as_str().to_string(),
        },
    );
    save_state(&paths.state, &state)?;

    println!(
        "{} Preset team {} created with profile {}!",
        "SUCCESS:".green().bold(),
        name.bold(),
        profile.as_str().cyan()
    );
    println!(
        "  Slash command: {}",
        format!("/forja--team--{name}").cyan()
    );
    println!(
        "  Command file: {}",
        cmd_path.display().to_string().dimmed()
    );

    Ok(())
}

/// List all configured teams with their profile and agent count.
pub fn list() -> Result<()> {
    let paths = ForjaPaths::new()?;
    let state = load_state(&paths.state);

    if state.teams.is_empty() {
        println!(
            "No teams configured. Create one with `forja team create <name>` or `forja team preset <name>`."
        );
        return Ok(());
    }

    println!("{}", "Configured teams".bold());
    println!();

    for (name, entry) in &state.teams {
        let cmd_path = paths.claude_commands.join(command_file_name(name));
        let status = if cmd_path.exists() {
            "OK".green().bold()
        } else {
            "MISSING CMD".red().bold()
        };

        println!(
            "  {} {} (profile: {}, agents: {}) [{}]",
            "●".cyan(),
            name.bold(),
            entry.profile.dimmed(),
            entry.members.len(),
            status
        );
    }

    println!();
    Ok(())
}

/// Show detailed information about a team: members, models, and slash command path.
pub fn info(name: &str) -> Result<()> {
    let paths = ForjaPaths::new()?;
    let state = load_state(&paths.state);

    let entry = state
        .teams
        .get(name)
        .ok_or_else(|| ForjaError::TeamNotFound(name.to_string()))?;

    println!("{}", format!("Team: {name}").bold());
    println!("Profile: {}", entry.profile.cyan());
    println!("Members: {}", entry.members.len());
    println!();

    for (i, member) in entry.members.iter().enumerate() {
        println!(
            "  {}. {} (skill: {}) → model: {}",
            i + 1,
            member.agent_name.bold(),
            member.skill_id.dimmed(),
            member.model.cyan()
        );
    }

    let cmd_path = paths.claude_commands.join(command_file_name(name));
    println!();
    if cmd_path.exists() {
        println!(
            "  Slash command: {}",
            format!("/forja--team--{name}").cyan()
        );
    } else {
        println!(
            "  {} Command file missing at {}",
            "WARNING:".yellow().bold(),
            cmd_path.display().to_string().dimmed()
        );
    }

    println!();
    Ok(())
}

/// Delete a team and its slash command file, with confirmation prompt.
pub fn delete(name: &str, skip_confirm: bool) -> Result<()> {
    let paths = ForjaPaths::new()?;
    let mut state = load_state(&paths.state);

    if !state.teams.contains_key(name) {
        return Err(ForjaError::TeamNotFound(name.to_string()));
    }

    if !skip_confirm {
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete team '{}'?", name))
            .default(true)
            .interact()
            .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

        if !confirmed {
            return Err(ForjaError::PromptCancelled);
        }
    }

    // Remove command file
    let cmd_path = paths.claude_commands.join(command_file_name(name));
    if cmd_path.exists() {
        fs::remove_file(&cmd_path)?;
    }

    // Remove from state
    state.teams.remove(name);
    save_state(&paths.state, &state)?;

    println!(
        "{} Team {} deleted.",
        "SUCCESS:".green().bold(),
        name.bold()
    );

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

fn find_first_md(dir: &Path) -> Result<std::path::PathBuf> {
    if !dir.exists() {
        return Err(ForjaError::SkillNotFound(format!(
            "Commands directory not found: {}",
            dir.display()
        )));
    }

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort();

    entries
        .into_iter()
        .next()
        .ok_or_else(|| ForjaError::SkillNotFound("No .md files in commands dir".to_string()))
}

pub(crate) fn apply_profile_to_command(content: &str, profile: &Profile) -> String {
    // Determine phase context from each "Phase:" marker in the content
    // Replace "Model: xxx" lines with the profile's model for that phase
    let mut result = String::new();
    let mut current_phase: Option<Phase> = None;

    for line in content.lines() {
        // Detect phase from section headers like "### 1. Researcher (Phase: RESEARCH)"
        if let Some(start) = line.find("Phase:") {
            let after = &line[start + 6..];
            let phase_str = after.trim().trim_end_matches(')').trim().to_lowercase();
            // Handle "CODE + TEST" style -- take the first one
            let first_phase = phase_str.split('+').next().unwrap_or("").trim();
            current_phase = first_phase.parse::<Phase>().ok();
        }

        // Replace Model: lines
        if line.starts_with("Model:") || line.starts_with("Model: ") {
            if let Some(phase) = current_phase {
                result.push_str(&format!("Model: {}", profile.resolve_model(phase)));
            } else {
                // Fallback: use the profile's default for Deploy
                result.push_str(&format!("Model: {}", profile.resolve_model(Phase::Deploy)));
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    result
}

pub(crate) fn resolve_preset_members(
    preset_name: &str,
    profile: &Profile,
) -> Result<Vec<TeamMember>> {
    let members = match preset_name {
        "full-product" => vec![
            ("research/codebase/explorer", "researcher", Phase::Research),
            ("code/general/feature", "coder", Phase::Code),
            ("test/tdd/workflow", "tester", Phase::Test),
            (
                "review/code-simplifier/simplifier",
                "code-simplifier",
                Phase::Review,
            ),
            ("review/code-quality/reviewer", "reviewer", Phase::Review),
            ("deploy/git/commit", "deployer", Phase::Deploy),
        ],
        "solo-sprint" => vec![
            ("code/general/feature", "coder-tester", Phase::Code),
            (
                "review/code-simplifier/simplifier",
                "code-simplifier",
                Phase::Review,
            ),
            ("review/code-quality/reviewer", "reviewer", Phase::Review),
        ],
        "quick-fix" => vec![
            ("code/general/feature", "coder", Phase::Code),
            ("deploy/git/commit", "deployer", Phase::Deploy),
        ],
        "dispatch" => vec![("teams/dispatch/team", "dispatcher", Phase::Teams)],
        "tech-council" => vec![(
            "teams/technical-council/team",
            "council-facilitator",
            Phase::Review,
        )],
        "biz-council" => vec![(
            "teams/strategic-council/team",
            "strategic-facilitator",
            Phase::Review,
        )],
        _ => {
            return Err(ForjaError::SkillNotFound(format!(
                "Unknown preset '{}'. Available: full-product, solo-sprint, quick-fix, dispatch, tech-council, biz-council",
                preset_name
            )));
        }
    };

    Ok(members
        .into_iter()
        .map(|(sid, agent_name, phase)| TeamMember {
            skill_id: sid.to_string(),
            agent_name: agent_name.to_string(),
            model: profile.resolve_model(phase).to_string(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn parse_agent_md_valid_frontmatter() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("coder.md");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "---\nname: coder\ndescription: Writes code\ntools: Read, Write, Edit, Bash\nmodel: sonnet\n---\n\nYou are a coder agent."
        )
        .unwrap();

        let result = parse_agent_md(&path).unwrap();
        assert_eq!(result.name, "coder");
        assert_eq!(result.description, "Writes code");
        assert_eq!(result.tools, "Read, Write, Edit, Bash");
        assert_eq!(result.model, "sonnet");
        assert_eq!(result.body, "You are a coder agent.");
    }

    #[test]
    fn parse_agent_md_no_frontmatter_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("plain.md");
        std::fs::write(&path, "Just a regular markdown file.").unwrap();

        assert!(parse_agent_md(&path).is_none());
    }

    #[test]
    fn parse_agent_md_missing_name_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("noname.md");
        std::fs::write(&path, "---\ndescription: test\n---\nbody").unwrap();

        assert!(parse_agent_md(&path).is_none());
    }

    #[test]
    fn generate_slash_command_has_all_sections() {
        let members: Vec<(AgentFrontmatter, Phase, &str)> = vec![
            (
                AgentFrontmatter {
                    name: "researcher".to_string(),
                    description: "Explores code".to_string(),
                    tools: "Read, Grep, Glob".to_string(),
                    model: "opus".to_string(),
                    body: "You explore the codebase.".to_string(),
                },
                Phase::Research,
                "opus",
            ),
            (
                AgentFrontmatter {
                    name: "coder".to_string(),
                    description: "Writes code".to_string(),
                    tools: "Read, Write, Edit".to_string(),
                    model: "sonnet".to_string(),
                    body: "You write production code.".to_string(),
                },
                Phase::Code,
                "sonnet",
            ),
        ];

        let output = generate_slash_command("my-team", &members);

        // Frontmatter
        assert!(output.starts_with("---\n"));
        assert!(output.contains("description: Launch the my-team team"));

        // Team structure
        assert!(output.contains("## Team Structure"));
        assert!(output.contains("Researcher"));
        assert!(output.contains("Coder"));

        // Orchestration — research before code
        let research_pos = output.find("**Researcher**").unwrap();
        let code_pos = output.find("**Coder**").unwrap();
        assert!(
            research_pos < code_pos,
            "Researcher should appear before Coder in orchestration"
        );

        // Tips
        assert!(output.contains("## Tips"));
        assert!(output.contains("delegate mode"));

        // Troubleshooting
        assert!(output.contains("## Troubleshooting"));
        assert!(output.contains("forja doctor"));
    }

    #[test]
    fn apply_profile_replaces_model_lines() {
        let content = "### 1. Researcher (Phase: RESEARCH)\nModel: sonnet\n### 2. Coder (Phase: CODE)\nModel: sonnet\n";
        let result = apply_profile_to_command(content, &Profile::Balanced);

        assert!(
            result.contains("Model: opus"),
            "Research phase should get opus"
        );
        // CODE phase should stay sonnet
        let lines: Vec<&str> = result.lines().collect();
        let model_lines: Vec<&&str> = lines.iter().filter(|l| l.starts_with("Model:")).collect();
        assert_eq!(model_lines.len(), 2);
        assert_eq!(*model_lines[0], "Model: opus");
        assert_eq!(*model_lines[1], "Model: sonnet");
    }

    #[test]
    fn command_file_name_format() {
        assert_eq!(command_file_name("my-team"), "forja--team--my-team.md");
        assert_eq!(
            command_file_name("full-product"),
            "forja--team--full-product.md"
        );
    }

    #[test]
    fn resolve_preset_members_full_product() {
        let members = resolve_preset_members("full-product", &Profile::Balanced).unwrap();
        assert_eq!(members.len(), 6);
        assert_eq!(members[0].agent_name, "researcher");
        assert_eq!(members[1].agent_name, "coder");
        assert_eq!(members[2].agent_name, "tester");
        assert_eq!(members[3].agent_name, "code-simplifier");
        assert_eq!(members[4].agent_name, "reviewer");
        assert_eq!(members[5].agent_name, "deployer");
    }

    #[test]
    fn resolve_preset_members_solo_sprint() {
        let members = resolve_preset_members("solo-sprint", &Profile::Balanced).unwrap();
        assert_eq!(members.len(), 3);
        assert_eq!(members[0].agent_name, "coder-tester");
        assert_eq!(members[1].agent_name, "code-simplifier");
        assert_eq!(members[2].agent_name, "reviewer");
    }

    #[test]
    fn resolve_preset_members_quick_fix() {
        let members = resolve_preset_members("quick-fix", &Profile::Balanced).unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].agent_name, "coder");
        assert_eq!(members[1].agent_name, "deployer");
    }

    #[test]
    fn resolve_preset_members_tech_council() {
        let members = resolve_preset_members("tech-council", &Profile::Balanced).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].agent_name, "council-facilitator");
        assert_eq!(members[0].skill_id, "teams/technical-council/team");
    }

    #[test]
    fn resolve_preset_members_biz_council() {
        let members = resolve_preset_members("biz-council", &Profile::Balanced).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].agent_name, "strategic-facilitator");
        assert_eq!(members[0].skill_id, "teams/strategic-council/team");
    }

    #[test]
    fn resolve_preset_members_unknown_errors() {
        let result = resolve_preset_members("nonexistent", &Profile::Balanced);
        assert!(result.is_err());
    }
}
