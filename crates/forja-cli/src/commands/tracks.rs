use colored::Colorize;

use forja_core::error::Result;
use forja_core::models::track;

use crate::output;

pub fn list() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let tracks_dir = cwd.join(".forja").join("tracks");

    let tracks = track::discover_tracks(&tracks_dir)?;

    if tracks.is_empty() {
        println!("{}", "No tracks found.".dimmed());
        output::print_tip("Create a track in .forja/tracks/");
        return Ok(());
    }

    println!("{}", "Tracks".bold());
    println!();

    let rows: Vec<Vec<String>> = tracks
        .iter()
        .map(|t| {
            let (done, total) = t.progress();
            vec![
                t.id().to_string(),
                t.title().to_string(),
                t.frontmatter.status.as_str().to_string(),
                format!("{done}/{total}"),
                t.frontmatter
                    .owner
                    .as_deref()
                    .unwrap_or("-")
                    .to_string(),
            ]
        })
        .collect();

    output::print_table(&["ID", "Title", "Status", "Progress", "Owner"], &rows);
    println!();
    output::print_tip("Show details: forja tracks show <id>");

    Ok(())
}

pub fn show(track_id: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let tracks_dir = cwd.join(".forja").join("tracks");

    let track = track::find_track(&tracks_dir, track_id)?;

    println!("{}", track.title().bold());
    println!("{}", track.frontmatter.description);
    println!();
    println!("  ID:       {}", track.id().cyan());
    println!(
        "  Status:   {}",
        track.frontmatter.status.as_str().cyan()
    );
    if let Some(ref owner) = track.frontmatter.owner {
        println!("  Owner:    {}", owner.cyan());
    }
    println!("  Created:  {}", track.frontmatter.created.dimmed());

    let (done, total) = track.progress();
    println!("  Progress: {done}/{total} done");

    if !track.items.is_empty() {
        println!();
        println!("  {}:", "Items".bold());
        println!();

        let rows: Vec<Vec<String>> = track
            .items
            .iter()
            .map(|item| {
                vec![
                    item.id.clone(),
                    item.task.clone(),
                    item.status.clone(),
                    item.spec.clone(),
                ]
            })
            .collect();

        output::print_table(&["ID", "Task", "Status", "Spec"], &rows);
    }

    if !track.body.is_empty()
        && !track.body.contains("| ID")
    {
        // Show non-table body content
        println!();
        println!("  {}", "── Notes ──".dimmed());
        for line in track.body.lines() {
            if !line.contains("| ID") && !line.contains("|---") && !line.starts_with('|') {
                println!("  {line}");
            }
        }
    }

    Ok(())
}
