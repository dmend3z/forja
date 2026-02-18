mod events;
mod server;
mod state;
mod watcher;

use std::sync::Arc;

use colored::Colorize;

use forja_core::error::{ForjaError, Result};

use state::DashboardState;

pub async fn run(port: u16, auto_open: bool) -> Result<()> {
    let home = dirs::home_dir().ok_or(ForjaError::NoHomeDir)?;
    let claude_dir = home.join(".claude");
    let teams_dir = claude_dir.join("teams");
    let tasks_dir = claude_dir.join("tasks");

    // Ensure directories exist (they may not if no team has been created yet)
    std::fs::create_dir_all(&teams_dir).ok();
    std::fs::create_dir_all(&tasks_dir).ok();

    let state = Arc::new(DashboardState::new());

    // Initial scan to populate state with existing data
    state.initial_scan(&teams_dir, &tasks_dir).await;

    // Start the file watcher in the background
    let watcher_state = Arc::clone(&state);
    let watcher_teams = teams_dir.clone();
    let watcher_tasks = tasks_dir.clone();
    tokio::spawn(async move {
        if let Err(e) = watcher::watch(watcher_state, watcher_teams, watcher_tasks).await {
            eprintln!("{} File watcher error: {}", "Warning:".yellow().bold(), e);
        }
    });

    let app = server::create_router(state);
    let addr = format!("127.0.0.1:{port}");
    let url = format!("http://{addr}");

    println!();
    println!(
        "  {} {}",
        "forja monitor".bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
    println!("  {} {}", "Dashboard:".cyan().bold(), url.bold());
    println!("  {}", "Press Ctrl+C to stop".dimmed());
    println!();

    if auto_open && let Err(e) = open::that(&url) {
        eprintln!(
            "  {} Could not open browser: {}",
            "Warning:".yellow().bold(),
            e
        );
    }

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| ForjaError::Monitor(format!("Failed to bind to {addr}: {e}")))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| ForjaError::Monitor(format!("Server error: {e}")))?;

    println!();
    println!("  {} Dashboard stopped", "✓".green());

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
}
