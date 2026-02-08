mod cli;
mod commands;
mod error;
mod models;
mod paths;
mod registry;
mod settings;
mod symlink;

use clap::Parser;
use cli::{Cli, Commands, TeamCommands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => commands::status::run(),
        Some(command) => match command {
            Commands::Init { registry_url } => commands::init::run(registry_url),
            Commands::Install { ref skill, all } => {
                if all {
                    commands::install::run_all()
                } else if let Some(skill) = skill {
                    commands::install::run(skill)
                } else {
                    eprintln!("Error: provide a skill ID or use --all");
                    std::process::exit(1);
                }
            }
            Commands::Uninstall { ref skill } => commands::uninstall::run(skill),
            Commands::Search { ref query } => commands::search::run(query),
            Commands::List { available } => commands::list::run(available),
            Commands::Update => commands::update::run(),
            Commands::Info { ref skill } => commands::info::run(skill),
            Commands::Phases => commands::phases::run(),
            Commands::Doctor => commands::doctor::run(),
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
            Commands::Team { command } => match command {
                TeamCommands::Create { name } => commands::team::create(&name),
                TeamCommands::Preset { name, ref profile } => {
                    commands::team::preset(&name, profile)
                }
                TeamCommands::List => commands::team::list(),
                TeamCommands::Info { ref name } => commands::team::info(name),
                TeamCommands::Delete { ref name } => commands::team::delete(name),
            },
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
