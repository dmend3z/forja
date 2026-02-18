use colored::Colorize;

use forja_core::error::{ForjaError, Result};
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::scanner::models::{ScanResult, SkillRecommendation};
use forja_core::scanner::{ai, deterministic, matcher};
use forja_core::symlink::manager::{SymlinkManager, load_installed_ids, save_installed_ids};

use crate::output;

pub fn run(yes: bool, json: bool, all: bool, basic: bool) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;

    let project_root = paths.project_root.clone().unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });

    // Step 1: Deterministic scan
    if !json {
        println!(
            "  {} Scanning {}...",
            "→".cyan(),
            project_root.display().to_string().dimmed()
        );
    }

    let det_techs = deterministic::scan(&project_root);

    // Step 2: AI analysis (optional)
    let (all_techs, ai_recs, ai_used) = if !basic && ai::claude_cli_available() {
        if !json {
            println!("  {} Running AI analysis...", "→".cyan());
        }

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| ForjaError::ScanError(format!("Failed to start runtime: {e}")))?;

        let (ai_techs, ai_recs) = rt.block_on(async {
            // Initial analysis
            let ai_output = ai::analyze_codebase(&project_root, &det_techs).await;

            let mut combined_techs = det_techs.clone();
            if let Ok(ref output) = ai_output {
                for ai_tech in &output.technologies {
                    if !combined_techs.iter().any(|t| t.name == ai_tech.name) {
                        combined_techs.push(forja_core::scanner::models::DetectedTech {
                            name: ai_tech.name.clone(),
                            category: ai_tech.category.clone(),
                            evidence: ai_tech.evidence.clone(),
                            source: forja_core::scanner::models::DetectionSource::Ai,
                            confidence: ai_tech.confidence,
                            version: None,
                        });
                    }
                }
            }

            // Deep dive per technology
            let dive_results =
                ai::deep_dive_technologies(&project_root, &combined_techs, &registry.skills).await;

            let mut all_ai_recs: Vec<SkillRecommendation> = Vec::new();
            for result in dive_results {
                match result {
                    Ok(recs) => all_ai_recs.extend(recs),
                    Err(e) => {
                        eprintln!(
                            "  {} AI deep-dive warning: {}",
                            "!".yellow(),
                            e.to_string().dimmed()
                        );
                    }
                }
            }

            (combined_techs, all_ai_recs)
        });

        (ai_techs, ai_recs, true)
    } else {
        if !basic && !json {
            println!(
                "  {} Claude CLI not found, using deterministic scan only",
                "!".yellow()
            );
        }
        (det_techs.clone(), Vec::new(), false)
    };

    // Step 3: Match and merge
    let det_recs = matcher::match_skills(&all_techs, &registry, &installed_ids);
    let mut recommendations = matcher::merge_recommendations(det_recs, ai_recs);
    matcher::sort_recommendations(&mut recommendations);

    // Enrich AI recommendations with registry data
    for rec in &mut recommendations {
        if rec.description.is_empty()
            && let Some(skill) = registry.find_by_id(&rec.skill_id)
        {
            rec.name = skill.name.clone();
            rec.description = skill.description.clone();
            rec.phase = skill.phase.as_str().to_string();
            rec.installed = installed_ids.contains(&rec.skill_id);
        }
    }

    // Remove recommendations for skills not in the registry
    recommendations.retain(|r| registry.find_by_id(&r.skill_id).is_some());

    let tech_count = all_techs.len();

    if !json {
        println!(
            "  {} {} technologies detected, {} skills recommended",
            "✓".green(),
            tech_count,
            recommendations.len()
        );
        println!();
    }

    // Step 4: Output
    if json {
        let result = ScanResult {
            project_root: project_root.clone(),
            detected_techs: all_techs,
            recommendations,
            ai_analysis_used: ai_used,
            scanned_at: chrono::Utc::now(),
        };
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return Ok(());
    }

    // Filter installed if not --all
    if !all {
        recommendations.retain(|r| !r.installed);
    }

    if recommendations.is_empty() {
        println!("  {} All recommended skills are already installed!", "✓".green());
        output::print_tip("Use 'forja scan --all' to see installed skills too");
        return Ok(());
    }

    // Step 5: Install
    if yes {
        // Auto-install all non-installed
        let to_install: Vec<String> = recommendations
            .iter()
            .filter(|r| !r.installed)
            .map(|r| r.skill_id.clone())
            .collect();

        if to_install.is_empty() {
            println!("  {} Nothing new to install", "✓".green());
            return Ok(());
        }

        return install_skills(&paths, &to_install);
    }

    // TUI selection
    match forja_tui::launch_scan(recommendations, all, tech_count)? {
        Some(selected_ids) if !selected_ids.is_empty() => {
            install_skills(&paths, &selected_ids)?;
        }
        _ => {
            println!("  {}", "Cancelled".dimmed());
        }
    }

    Ok(())
}

fn install_skills(paths: &ForjaPaths, skill_ids: &[String]) -> Result<()> {
    let mut installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut count = 0;

    for skill_id in skill_ids {
        if installed_ids.contains(skill_id) {
            continue;
        }

        if let Some(skill) = registry.find_by_id(skill_id) {
            match manager.install(skill) {
                Ok(_) => {
                    installed_ids.push(skill.id.clone());
                    rows.push(vec![
                        skill.phase.as_str().to_string(),
                        skill.name.clone(),
                        "installed".to_string(),
                    ]);
                    count += 1;
                }
                Err(e) => {
                    rows.push(vec![
                        skill.phase.as_str().to_string(),
                        skill.name.clone(),
                        format!("failed: {e}"),
                    ]);
                }
            }
        }
    }

    save_installed_ids(&paths.state, &installed_ids)?;

    println!(
        "  {} Installed {} skill{}:",
        "✓".green().bold(),
        count,
        if count == 1 { "" } else { "s" }
    );
    println!();
    output::print_table(&["Phase", "Skill", "Status"], &rows);

    println!();
    println!("  {} Next steps:", "→".cyan().bold());
    output::print_command_hint("forja list", "See all installed skills");
    output::print_command_hint("forja task \"your task\"", "Run a task with installed skills");
    output::print_command_hint("forja team preset solo-sprint", "Set up multi-agent workflows");

    Ok(())
}
