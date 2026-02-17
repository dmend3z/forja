use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[allow(deprecated)]
fn forja() -> Command {
    let mut cmd = Command::cargo_bin("forja").unwrap();
    cmd.env("NO_COLOR", "1");
    cmd
}

fn create_specs_dir(dir: &TempDir) {
    let specs = dir.path().join("docs").join("specs");
    fs::create_dir_all(&specs).unwrap();

    fs::write(
        specs.join("user-auth.md"),
        "---\nid: user-auth\ntitle: User Authentication\ndescription: Add JWT auth\npriority: high\ntags:\n  - auth\nrequirements:\n  - Login endpoint\nsuccess_criteria:\n  - Users can log in\n---\n\n# Auth\n\nDetails here.\n",
    )
    .unwrap();

    fs::write(
        specs.join("search-api.md"),
        "---\nid: search-api\ntitle: Search API\ndescription: Full-text search\npriority: medium\n---\n\n# Search\n\nElasticsearch integration.\n",
    )
    .unwrap();
}

// --- Help tests ---

#[test]
fn sparks_help_shows_all_subcommands() {
    forja()
        .args(["sparks", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("show"))
                .and(predicate::str::contains("plan"))
                .and(predicate::str::contains("execute"))
                .and(predicate::str::contains("status")),
        );
}

#[test]
fn sparks_help_shows_examples() {
    forja()
        .args(["sparks", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn sparks_list_help() {
    forja()
        .args(["sparks", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--path"));
}

#[test]
fn sparks_execute_help_shows_args() {
    forja()
        .args(["sparks", "execute", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("--profile")
                .and(predicate::str::contains("--resume"))
                .and(predicate::str::contains("SPEC_ID")),
        );
}

// --- List tests ---

#[test]
fn sparks_list_with_specs() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    forja()
        .args(["sparks", "list", "--path", dir.path().join("docs/specs").to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("search-api")
                .and(predicate::str::contains("user-auth"))
                .and(predicate::str::contains("User Authentication"))
                .and(predicate::str::contains("Search API")),
        );
}

#[test]
fn sparks_list_empty_dir() {
    let dir = TempDir::new().unwrap();
    let specs = dir.path().join("empty-specs");
    fs::create_dir_all(&specs).unwrap();

    forja()
        .args(["sparks", "list", "--path", specs.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No specs found"));
}

#[test]
fn sparks_list_missing_dir() {
    forja()
        .args(["sparks", "list", "--path", "/nonexistent/specs/path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("specs directory not found"));
}

// --- Show tests ---

#[test]
fn sparks_show_displays_spec() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    forja()
        .args(["sparks", "show", "user-auth"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("User Authentication")
                .and(predicate::str::contains("Add JWT auth"))
                .and(predicate::str::contains("high"))
                .and(predicate::str::contains("Login endpoint"))
                .and(predicate::str::contains("Users can log in"))
                .and(predicate::str::contains("Auth")),
        );
}

#[test]
fn sparks_show_not_found() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    forja()
        .args(["sparks", "show", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

// --- Status tests ---

#[test]
fn sparks_status_summary_shows_specs() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    forja()
        .args(["sparks", "status"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("user-auth")
                .and(predicate::str::contains("search-api"))
                .and(predicate::str::contains("Status")),
        );
}

#[test]
fn sparks_status_detail_no_plan() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    forja()
        .args(["sparks", "status", "user-auth"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("User Authentication")
                .and(predicate::str::contains("No plan generated yet")),
        );
}

#[test]
fn sparks_status_not_found() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    forja()
        .args(["sparks", "status", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

// --- Execute tests ---

#[test]
fn sparks_execute_no_plan_fails() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    // Create .forja/config.json so ensure_initialized passes
    let forja_dir = dir.path().join(".forja");
    fs::create_dir_all(&forja_dir).unwrap();
    fs::write(forja_dir.join("config.json"), "{}").unwrap();
    fs::create_dir_all(forja_dir.join("plans")).unwrap();

    forja()
        .args(["sparks", "execute", "user-auth"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no plan found for spec"));
}

#[test]
fn sparks_execute_missing_spec_fails() {
    let dir = TempDir::new().unwrap();
    create_specs_dir(&dir);

    forja()
        .args(["sparks", "execute", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}
