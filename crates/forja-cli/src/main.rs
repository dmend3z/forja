//! forja — Skills marketplace CLI for Claude Code.
//!
//! Manages a catalog of agent skills organized into five workflow phases
//! (Research, Code, Test, Review, Deploy). Skills are installed as symlinks
//! into `~/.claude/agents/` and `~/.claude/commands/`, and can be composed
//! into multi-agent teams.

mod cli;
mod commands;
mod output;
mod tips;
mod wizard;

use clap::Parser;
use cli::{Cli, Commands, TeamCommands};

fn run() -> forja_core::error::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => commands::status::run(),
        Some(command) => dispatch(command),
    }
}

fn dispatch(command: Commands) -> forja_core::error::Result<()> {
    match command {
        Commands::Init {
            registry_url,
            wizard,
            global,
        } => commands::init::run(registry_url, wizard, global),
        Commands::Install { skill, all, global } => {
            if all {
                commands::install::run_all(global)
            } else {
                // safe: clap enforces `skill` is present when `--all` is not used
                commands::install::run(&skill.unwrap(), global)
            }
        }
        Commands::Uninstall {
            ref skill,
            yes,
            global,
        } => commands::uninstall::run(skill, yes, global),
        Commands::Search { ref query } => commands::search::run(query),
        Commands::List { available } => commands::list::run(available),
        Commands::Update => commands::update::run(),
        Commands::Info { ref skill } => commands::info::run(skill),
        Commands::Doctor => commands::doctor::run(),
        Commands::Guide { ref phase } => commands::guide::run(phase.as_deref()),
        Commands::Plan { ref task } => commands::plan::run(task.as_deref()),
        Commands::Task {
            ref task,
            print,
            ref team,
            ref profile,
        } => commands::task::run(task.as_deref(), print, team.as_deref(), profile.as_deref()),
        Commands::Execute {
            ref plan_id,
            ref profile,
            resume,
        } => commands::execute::run(plan_id.as_deref(), profile, resume),
        Commands::Fix {
            ref description,
            ref profile,
        } => commands::fix::run(description, profile.as_deref()),
        Commands::Build {
            ref description,
            ref profile,
        } => commands::build::run(description, profile.as_deref()),
        Commands::Chronicle { ref from } => commands::chronicle::run(from),
        Commands::Review {
            ref path,
            no_chronicle,
        } => commands::review::run(path.as_deref(), no_chronicle),
        Commands::Ship {
            ref message,
            commit_only,
            no_chronicle,
        } => commands::ship::run(message.as_deref(), commit_only, no_chronicle),
        Commands::Lint { ref path, warnings } => commands::lint::run(path.as_deref(), warnings),
        Commands::New {
            ref name,
            ref phase,
            ref tech,
            no_wizard,
        } => commands::new::run(
            name.as_deref(),
            phase.as_deref(),
            tech.as_deref(),
            no_wizard,
        ),
        Commands::Stats => commands::stats::run(),
        Commands::Diff => commands::diff::run(),
        Commands::Docs { ref scope } => commands::docs::run(scope.as_deref()),
        Commands::Upgrade { ref skill, yes } => commands::upgrade::run(skill.as_deref(), yes),
        Commands::Monitor { port, no_open } => {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| forja_core::error::ForjaError::Monitor(format!("Failed to start runtime: {e}")))?;
            rt.block_on(commands::monitor::run(port, !no_open))
        }
        Commands::Team { command } => match command {
            TeamCommands::Create { name } => commands::team::create(&name),
            TeamCommands::Preset { name, ref profile } => commands::team::preset(&name, profile),
            TeamCommands::List => commands::team::list(),
            TeamCommands::Info { ref name } => commands::team::info(name),
            TeamCommands::Delete { ref name, yes } => commands::team::delete(name, yes),
        },
    }
}

fn main() {
    if let Err(e) = run() {
        output::print_error_with_hint(&e.to_string(), e.hint());
        std::process::exit(e.exit_code());
    }
}
