use crate::error::{ForjaError, Result};
use crate::output;
use colored::Colorize;

struct PhaseGuide {
    name: &'static str,
    key: &'static str,
    color: &'static str,
    description: &'static str,
    commands: &'static [(&'static str, &'static str)],
    example: &'static str,
}

const PHASES: &[PhaseGuide] = &[
    PhaseGuide {
        name: "Research",
        key: "research",
        color: "blue",
        description: "Explore the codebase and plan before writing any code. \
            Research skills help you understand existing patterns, architecture, \
            and external documentation.",
        commands: &[
            (
                "forja task \"explore the auth module\"",
                "Research a specific area",
            ),
            (
                "forja plan \"add user roles\"",
                "Create an implementation plan",
            ),
        ],
        example: "forja task \"how does the API layer handle errors?\"",
    },
    PhaseGuide {
        name: "Code",
        key: "code",
        color: "green",
        description: "Write production-ready code that follows existing project patterns. \
            Code skills are organized by language and framework — Rust, TypeScript, \
            Python, Go, Next.js, NestJS, and more.",
        commands: &[
            (
                "forja task \"implement the login endpoint\"",
                "Code a feature",
            ),
            (
                "forja task \"fix the null check bug\" --team quick-fix",
                "Quick fix with a team",
            ),
        ],
        example: "forja task \"add a REST endpoint for user profiles\"",
    },
    PhaseGuide {
        name: "Test",
        key: "test",
        color: "yellow",
        description: "Follow TDD workflow: write tests first, then make them pass. \
            Test skills generate comprehensive test suites with edge cases \
            and target 80%+ coverage.",
        commands: &[
            (
                "forja task \"write tests for the auth module\"",
                "Generate tests",
            ),
            (
                "forja task \"add integration tests\"",
                "Integration testing",
            ),
        ],
        example: "forja task \"write unit tests for the user service\"",
    },
    PhaseGuide {
        name: "Review",
        key: "review",
        color: "magenta",
        description: "Review code for quality, security, and performance before deploying. \
            Review skills check for OWASP Top 10, N+1 queries, bundle size, \
            and provide specific fix examples.",
        commands: &[
            (
                "forja task \"review the latest changes\"",
                "Code quality review",
            ),
            ("forja task \"security audit the API\"", "Security audit"),
        ],
        example: "forja task \"review PR #42 for security issues\"",
    },
    PhaseGuide {
        name: "Deploy",
        key: "deploy",
        color: "cyan",
        description: "Create conventional commits and structured pull requests. \
            Deploy skills handle git workflow, PR descriptions, and CI verification.",
        commands: &[
            ("forja task \"commit and create a PR\"", "Commit + PR flow"),
            ("forja task \"prepare release v1.2\"", "Release preparation"),
        ],
        example: "forja task \"create a PR for the auth feature\"",
    },
];

/// Print the getting-started guide, optionally filtered to a single phase.
pub fn run(phase: Option<&str>) -> Result<()> {
    if let Some(phase_key) = phase {
        let guide = PHASES
            .iter()
            .find(|p| p.key.eq_ignore_ascii_case(phase_key))
            .ok_or_else(|| {
                ForjaError::SkillNotFound(format!(
                    "phase '{}' (valid: research, code, test, review, deploy)",
                    phase_key
                ))
            })?;

        print_phase(guide);
        return Ok(());
    }

    println!();
    println!("  {} {}", "forja".bold(), "Getting Started Guide".dimmed());
    println!();
    println!(
        "  forja organizes your development into {} workflow phases.",
        "5".bold()
    );
    println!("  Each phase has specialized AI skills that help you work faster.");
    println!();

    for guide in PHASES {
        print_phase(guide);
    }

    // Quick Start box
    output::print_section_header("Quick Start");
    output::print_command_hint("forja init", "Set up forja and install all skills");
    output::print_command_hint("forja install --all", "Install every available skill");
    output::print_command_hint(
        "forja task \"your task\"",
        "Run any task with the right skills",
    );

    println!();
    output::print_tip("Run 'forja guide --phase <name>' to focus on a specific phase");
    println!();

    Ok(())
}

fn print_phase(guide: &PhaseGuide) {
    let colored_name = match guide.color {
        "blue" => guide.name.blue().bold(),
        "green" => guide.name.green().bold(),
        "yellow" => guide.name.yellow().bold(),
        "magenta" => guide.name.magenta().bold(),
        "cyan" => guide.name.cyan().bold(),
        _ => guide.name.bold(),
    };

    println!("  {}", colored_name);
    println!("  {}", guide.description.dimmed());
    println!();

    for (cmd, desc) in guide.commands {
        output::print_command_hint(cmd, desc);
    }

    println!();
    println!("  {} {}", "Try:".dimmed(), guide.example.cyan());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_phases_present() {
        assert_eq!(PHASES.len(), 5);

        let keys: Vec<&str> = PHASES.iter().map(|p| p.key).collect();
        assert!(keys.contains(&"research"));
        assert!(keys.contains(&"code"));
        assert!(keys.contains(&"test"));
        assert!(keys.contains(&"review"));
        assert!(keys.contains(&"deploy"));
    }

    #[test]
    fn phase_filter_valid() {
        assert!(run(Some("code")).is_ok());
        assert!(run(Some("CODE")).is_ok());
        assert!(run(Some("Research")).is_ok());
    }

    #[test]
    fn phase_filter_invalid() {
        let result = run(Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn full_guide_runs() {
        assert!(run(None).is_ok());
    }
}
