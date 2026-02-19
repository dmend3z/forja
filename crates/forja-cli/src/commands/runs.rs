use colored::Colorize;

use forja_core::error::Result;
use forja_core::models::run_log;

use crate::output;

pub fn list() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let runs_dir = cwd.join(".forja").join("runs");

    let runs = run_log::discover_runs(&runs_dir)?;

    if runs.is_empty() {
        println!("{}", "No runs found.".dimmed());
        output::print_tip("Execute a spec: forja execute <spec-id>");
        return Ok(());
    }

    println!("{}", "Execution Runs".bold());
    println!();

    let rows: Vec<Vec<String>> = runs
        .iter()
        .map(|r| {
            let date = r
                .frontmatter
                .started_at
                .split('T')
                .next()
                .unwrap_or(&r.frontmatter.started_at);
            let duration = r
                .frontmatter
                .duration_seconds
                .map(|s| format!("{s}s"))
                .unwrap_or_else(|| "-".to_string());
            vec![
                r.filename.clone(),
                r.spec_id().to_string(),
                r.frontmatter.agent.clone(),
                r.frontmatter.status.as_str().to_string(),
                duration,
                date.to_string(),
            ]
        })
        .collect();

    output::print_table(
        &["Run", "Spec", "Agent", "Status", "Duration", "Date"],
        &rows,
    );
    println!();
    output::print_tip("Show details: forja runs show <run-id>");

    Ok(())
}

pub fn show(run_id: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let runs_dir = cwd.join(".forja").join("runs");

    // Try to find the run by ID (filename stem)
    let path = runs_dir.join(format!("{run_id}.md"));
    let run = run_log::load_run_log(&path).map_err(|_| {
        forja_core::error::ForjaError::InvalidArgument(format!("run not found: {run_id}"))
    })?;

    println!("{}", format!("Run: {}", run.filename).bold());
    println!();
    println!("  Spec:     {}", run.spec_id().cyan());
    if let Some(ref plan_id) = run.frontmatter.plan_id {
        println!("  Plan:     {}", plan_id.cyan());
    }
    println!("  Agent:    {}", run.frontmatter.agent);
    println!(
        "  Status:   {}",
        run.frontmatter.status.as_str().cyan()
    );
    println!(
        "  Started:  {}",
        run.frontmatter.started_at.dimmed()
    );
    if let Some(ref completed) = run.frontmatter.completed_at {
        println!("  Completed: {}", completed.dimmed());
    }
    if let Some(duration) = run.frontmatter.duration_seconds {
        println!("  Duration: {}s", duration);
    }
    if let Some(code) = run.frontmatter.exit_code {
        println!("  Exit:     {code}");
    }

    if !run.body.is_empty() {
        println!();
        println!("  {}", "── Output ──".dimmed());
        for line in run.body.lines().take(50) {
            println!("  {line}");
        }
        let total_lines = run.body.lines().count();
        if total_lines > 50 {
            println!(
                "  {} ({} more lines)",
                "...".dimmed(),
                total_lines - 50
            );
        }
    }

    Ok(())
}
