use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn forja() -> Command {
    let mut cmd = Command::cargo_bin("forja").unwrap();
    // Use NO_COLOR for predictable output in assertions
    cmd.env("NO_COLOR", "1");
    cmd
}

#[test]
fn help_contains_all_subcommands() {
    forja().arg("--help").assert().success().stdout(
        predicate::str::contains("init")
            .and(predicate::str::contains("install"))
            .and(predicate::str::contains("uninstall"))
            .and(predicate::str::contains("search"))
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("update"))
            .and(predicate::str::contains("info"))
            .and(predicate::str::contains("doctor"))
            .and(predicate::str::contains("plan"))
            .and(predicate::str::contains("task"))
            .and(predicate::str::contains("execute"))
            .and(predicate::str::contains("team"))
            .and(predicate::str::contains("guide")),
    );
}

#[test]
fn help_shows_workflow_phases() {
    forja().arg("--help").assert().success().stdout(
        predicate::str::contains("Research")
            .and(predicate::str::contains("Code"))
            .and(predicate::str::contains("Test"))
            .and(predicate::str::contains("Review"))
            .and(predicate::str::contains("Deploy")),
    );
}

#[test]
fn guide_runs_without_error() {
    forja().arg("guide").assert().success();
}

#[test]
fn guide_phase_code_shows_code_phase() {
    forja()
        .args(["guide", "--phase", "code"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Code"));
}

#[test]
fn guide_invalid_phase_fails() {
    forja()
        .args(["guide", "--phase", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Hint:"));
}

#[test]
fn version_flag_works() {
    forja()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("forja"));
}

#[test]
fn install_help_shows_examples() {
    forja()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("EXAMPLES:").and(predicate::str::contains("forja install")),
        );
}

#[test]
fn search_help_shows_examples() {
    forja()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn team_help_shows_examples() {
    forja()
        .args(["team", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"));
}
