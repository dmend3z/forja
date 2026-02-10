//! forja — Skills marketplace CLI for Claude Code.
//!
//! Manages a catalog of agent skills organized into five workflow phases
//! (Research, Code, Test, Review, Deploy). Skills are installed as symlinks
//! into `~/.claude/agents/` and `~/.claude/commands/`, and can be composed
//! into multi-agent teams.

mod cli;
mod commands;
mod error;
mod models;
mod output;
mod paths;
mod registry;
mod settings;
mod symlink;
mod tips;
mod wizard;

use clap::Parser;
use cli::{Cli, Commands, TeamCommands};

fn run() -> error::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => commands::status::run(),
        Some(command) => dispatch(command),
    }
}

fn dispatch(command: Commands) -> error::Result<()> {
    match command {
        Commands::Init {
            registry_url,
            global,
        } => commands::init::run(registry_url, global),
        Commands::Install { skill, all } => {
            if all {
                commands::install::run_all()
            } else {
                // safe: clap enforces `skill` is present when `--all` is not used
                commands::install::run(&skill.unwrap())
            }
        }
        Commands::Uninstall { ref skill, yes } => commands::uninstall::run(skill, yes),
        Commands::Search { ref query } => commands::search::run(query),
        Commands::List { available } => commands::list::run(available),
        Commands::Update => commands::update::run(),
        Commands::Info { ref skill } => commands::info::run(skill),
        Commands::Phases => commands::phases::run(),
        Commands::Doctor => commands::doctor::run(),
        Commands::Guide { ref phase } => commands::guide::run(phase.as_deref()),
        Commands::Plan { ref task } => commands::plan::run(task),
        Commands::Task {
            ref task,
            print,
            ref team,
            ref profile,
        } => commands::task::run(task, print, team.as_deref(), profile.as_deref()),
        Commands::Execute {
            ref plan_id,
            ref profile,
        } => commands::execute::run(plan_id.as_deref(), profile),
        Commands::Monitor { port, no_open } => {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| error::ForjaError::Monitor(format!("Failed to start runtime: {e}")))?;
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
