use colored::Colorize;

use forja_core::error::Result;
use forja_core::models::decision;

use crate::output;

pub fn list() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let decisions_dir = cwd.join(".forja").join("decisions");

    let decisions = decision::discover_decisions(&decisions_dir)?;

    if decisions.is_empty() {
        println!("{}", "No decisions found.".dimmed());
        output::print_tip("Create a decision in .forja/decisions/");
        return Ok(());
    }

    println!("{}", "Decisions".bold());
    println!();

    let rows: Vec<Vec<String>> = decisions
        .iter()
        .map(|d| {
            vec![
                d.id().to_string(),
                d.title().to_string(),
                d.frontmatter.status.as_str().to_string(),
                d.frontmatter.date.clone(),
            ]
        })
        .collect();

    output::print_table(&["ID", "Title", "Status", "Date"], &rows);
    println!();
    output::print_tip("Show details: forja decisions show <id>");

    Ok(())
}

pub fn show(decision_id: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let decisions_dir = cwd.join(".forja").join("decisions");

    let decision = decision::find_decision(&decisions_dir, decision_id)?;

    println!("{}", decision.title().bold());
    println!();
    println!("  ID:       {}", decision.id().cyan());
    println!(
        "  Status:   {}",
        decision.frontmatter.status.as_str().cyan()
    );
    println!("  Date:     {}", decision.frontmatter.date.dimmed());

    if !decision.frontmatter.related_specs.is_empty() {
        println!(
            "  Specs:    {}",
            decision.frontmatter.related_specs.join(", ").dimmed()
        );
    }

    if let Some(ref sup) = decision.frontmatter.superseded_by {
        println!("  Superseded by: {}", sup.yellow());
    }

    if !decision.body.is_empty() {
        println!();
        for line in decision.body.lines() {
            println!("  {line}");
        }
    }

    Ok(())
}
