use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::error::Result;
use crate::models::decision;
use crate::models::spec;
use crate::models::track;
use crate::models::vision;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub file: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
        };
        write!(f, "[{prefix}] {}: {}", self.file, self.message)
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        !self.errors.iter().any(|e| e.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::Warning)
            .count()
    }
}

/// Validate all .forja/ files: schema, references, completeness, consistency.
pub fn validate_project(forja_dir: &Path) -> Result<ValidationResult> {
    let mut errors = Vec::new();

    // --- Vision ---
    let vision_path = forja_dir.join("docs").join("vision.md");
    let vision = if vision_path.exists() {
        match vision::load_vision(&vision_path) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(ValidationError {
                    file: "docs/vision.md".into(),
                    message: format!("Failed to parse: {e}"),
                    severity: Severity::Error,
                });
                None
            }
        }
    } else {
        None
    };

    // Completeness: vision must have project and description
    if let Some(ref v) = vision {
        if v.frontmatter.project.is_empty() {
            errors.push(ValidationError {
                file: "docs/vision.md".into(),
                message: "project field is empty".into(),
                severity: Severity::Error,
            });
        }
        if v.frontmatter.description.is_empty() {
            errors.push(ValidationError {
                file: "docs/vision.md".into(),
                message: "description field is empty".into(),
                severity: Severity::Error,
            });
        }
    }

    // --- Tracks ---
    let tracks_dir = forja_dir.join("tracks");
    let tracks = track::discover_tracks(&tracks_dir).unwrap_or_default();

    for t in &tracks {
        // Completeness: every track must have at least one item
        if t.items.is_empty() {
            errors.push(ValidationError {
                file: format!("tracks/track-{}.md", t.id()),
                message: "track has no items in its table".into(),
                severity: Severity::Warning,
            });
        }
    }

    let track_ids: HashSet<&str> = tracks.iter().map(|t| t.id()).collect();

    // --- Specs ---
    let specs_dir = forja_dir.join("specs");
    let specs = if specs_dir.exists() {
        spec::discover_specs(&specs_dir).unwrap_or_default()
    } else {
        Vec::new()
    };

    let spec_ids: HashSet<&str> = specs.iter().map(|s| s.id()).collect();

    for s in &specs {
        // Completeness: every spec should have at least one acceptance criterion
        if s.frontmatter.acceptance_criteria.is_empty() && s.frontmatter.success_criteria.is_empty()
        {
            errors.push(ValidationError {
                file: format!("specs/{}.md", s.id()),
                message: "spec has no acceptance criteria or success criteria".into(),
                severity: Severity::Warning,
            });
        }

        // Reference: track must exist
        if let Some(ref track_ref) = s.frontmatter.track
            && !track_ids.contains(track_ref.as_str()) {
                errors.push(ValidationError {
                    file: format!("specs/{}.md", s.id()),
                    message: format!("references non-existent track '{track_ref}'"),
                    severity: Severity::Error,
                });
            }

        // Reference: blocked_by must exist
        for dep in &s.frontmatter.blocked_by {
            if !spec_ids.contains(dep.as_str()) {
                errors.push(ValidationError {
                    file: format!("specs/{}.md", s.id()),
                    message: format!("blocked_by references non-existent spec '{dep}'"),
                    severity: Severity::Error,
                });
            }
        }

        // Reference: depends_on must exist
        for dep in &s.frontmatter.depends_on {
            if !spec_ids.contains(dep.as_str()) {
                errors.push(ValidationError {
                    file: format!("specs/{}.md", s.id()),
                    message: format!("depends_on references non-existent spec '{dep}'"),
                    severity: Severity::Error,
                });
            }
        }
    }

    // Consistency: check for circular dependencies
    check_circular_deps(&specs, &mut errors);

    // Reference: track table items must reference existing spec IDs
    for t in &tracks {
        for item in &t.items {
            if !item.spec.is_empty() && item.spec != "-" && !spec_ids.contains(item.spec.as_str())
            {
                errors.push(ValidationError {
                    file: format!("tracks/track-{}.md", t.id()),
                    message: format!(
                        "item '{}' references non-existent spec '{}'",
                        item.id, item.spec
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    // --- Decisions ---
    let decisions_dir = forja_dir.join("decisions");
    let decisions = decision::discover_decisions(&decisions_dir).unwrap_or_default();

    let decision_ids: HashSet<&str> = decisions.iter().map(|d| d.id()).collect();

    for d in &decisions {
        // Reference: related_specs must exist
        for spec_ref in &d.frontmatter.related_specs {
            if !spec_ids.contains(spec_ref.as_str()) {
                errors.push(ValidationError {
                    file: format!("decisions/{}.md", d.id()),
                    message: format!("related_specs references non-existent spec '{spec_ref}'"),
                    severity: Severity::Error,
                });
            }
        }

        // Reference: superseded_by must exist
        if let Some(ref sup) = d.frontmatter.superseded_by
            && !decision_ids.contains(sup.as_str()) {
                errors.push(ValidationError {
                    file: format!("decisions/{}.md", d.id()),
                    message: format!("superseded_by references non-existent decision '{sup}'"),
                    severity: Severity::Error,
                });
            }
    }

    // --- Plans ---
    let plans_dir = forja_dir.join("plans");
    if plans_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&plans_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "md")
                    && let Ok(content) = std::fs::read_to_string(&path)
                        && let Ok((yaml, _)) = crate::frontmatter::split_frontmatter(&content) {
                            // Check source_spec reference
                            if let Ok(fm) =
                                serde_yaml::from_str::<serde_yaml::Value>(yaml)
                                && let Some(source_spec) =
                                    fm.get("source_spec").and_then(|v| v.as_str())
                                    && !spec_ids.contains(source_spec) {
                                        let filename = path
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy();
                                        errors.push(ValidationError {
                                            file: format!("plans/{filename}"),
                                            message: format!(
                                                "source_spec references non-existent spec '{source_spec}'"
                                            ),
                                            severity: Severity::Error,
                                        });
                                    }
                        }
            }
        }

    Ok(ValidationResult { errors })
}

/// Check for circular dependencies in spec depends_on/blocked_by.
fn check_circular_deps(
    specs: &[spec::SpecFile],
    errors: &mut Vec<ValidationError>,
) {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for s in specs {
        let mut deps = Vec::new();
        for d in &s.frontmatter.depends_on {
            deps.push(d.as_str());
        }
        for d in &s.frontmatter.blocked_by {
            deps.push(d.as_str());
        }
        graph.insert(s.id(), deps);
    }

    // DFS cycle detection
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    for spec_id in graph.keys() {
        if !visited.contains(spec_id) {
            let mut path = Vec::new();
            if has_cycle(spec_id, &graph, &mut visited, &mut in_stack, &mut path) {
                errors.push(ValidationError {
                    file: format!("specs/{}.md", spec_id),
                    message: format!(
                        "circular dependency detected: {}",
                        path.join(" -> ")
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }
}

fn has_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
    path: &mut Vec<String>,
) -> bool {
    visited.insert(node);
    in_stack.insert(node);
    path.push(node.to_string());

    if let Some(deps) = graph.get(node) {
        for dep in deps {
            if !visited.contains(dep) {
                if has_cycle(dep, graph, visited, in_stack, path) {
                    return true;
                }
            } else if in_stack.contains(dep) {
                path.push(dep.to_string());
                return true;
            }
        }
    }

    in_stack.remove(node);
    path.pop();
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_forja_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let forja = dir.path();
        fs::create_dir_all(forja.join("docs")).unwrap();
        fs::create_dir_all(forja.join("tracks")).unwrap();
        fs::create_dir_all(forja.join("specs")).unwrap();
        fs::create_dir_all(forja.join("decisions")).unwrap();
        fs::create_dir_all(forja.join("plans")).unwrap();
        fs::create_dir_all(forja.join("runs")).unwrap();

        // Minimal valid vision
        fs::write(
            forja.join("docs").join("vision.md"),
            "---\nproject: Test\ndescription: A test project\n---\n# Vision\n",
        )
        .unwrap();

        dir
    }

    #[test]
    fn valid_project_passes() {
        let dir = setup_forja_dir();
        let forja = dir.path();

        // Add a track
        fs::write(
            forja.join("tracks").join("track-mvp.md"),
            "---\nid: mvp\ntitle: MVP\ndescription: D\nstatus: draft\ncreated: \"2026-01-01\"\n---\n| ID | Task | Status | Spec |\n|---|---|---|---|\n| M1 | Auth | todo | auth |\n",
        ).unwrap();

        // Add a spec
        fs::write(
            forja.join("specs").join("auth.md"),
            "---\nid: auth\ntitle: Auth\ndescription: D\ntrack: mvp\nacceptance_criteria:\n  - \"Users can log in\"\n---\n# Auth\n",
        ).unwrap();

        let result = validate_project(forja).unwrap();
        assert!(result.is_valid(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn invalid_spec_track_reference() {
        let dir = setup_forja_dir();
        let forja = dir.path();

        fs::write(
            forja.join("specs").join("auth.md"),
            "---\nid: auth\ntitle: Auth\ndescription: D\ntrack: nonexistent\n---\n",
        )
        .unwrap();

        let result = validate_project(forja).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("nonexistent")));
    }

    #[test]
    fn invalid_spec_depends_on_reference() {
        let dir = setup_forja_dir();
        let forja = dir.path();

        fs::write(
            forja.join("specs").join("api.md"),
            "---\nid: api\ntitle: API\ndescription: D\ndepends_on:\n  - missing\n---\n",
        )
        .unwrap();

        let result = validate_project(forja).unwrap();
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("depends_on") && e.message.contains("missing")));
    }

    #[test]
    fn circular_dependency_detected() {
        let dir = setup_forja_dir();
        let forja = dir.path();

        fs::write(
            forja.join("specs").join("a.md"),
            "---\nid: a\ntitle: A\ndescription: D\ndepends_on:\n  - b\n---\n",
        )
        .unwrap();
        fs::write(
            forja.join("specs").join("b.md"),
            "---\nid: b\ntitle: B\ndescription: D\ndepends_on:\n  - a\n---\n",
        )
        .unwrap();

        let result = validate_project(forja).unwrap();
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("circular")));
    }

    #[test]
    fn invalid_track_item_spec_reference() {
        let dir = setup_forja_dir();
        let forja = dir.path();

        fs::write(
            forja.join("tracks").join("track-mvp.md"),
            "---\nid: mvp\ntitle: MVP\ndescription: D\nstatus: draft\ncreated: \"2026-01-01\"\n---\n| ID | Task | Status | Spec |\n|---|---|---|---|\n| M1 | Auth | todo | missing-spec |\n",
        ).unwrap();

        let result = validate_project(forja).unwrap();
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("missing-spec")));
    }

    #[test]
    fn invalid_decision_spec_reference() {
        let dir = setup_forja_dir();
        let forja = dir.path();

        fs::write(
            forja.join("decisions").join("001.md"),
            "---\nid: \"001\"\ntitle: T\nstatus: accepted\ndate: \"2026-01-01\"\nrelated_specs:\n  - missing\n---\nBody.",
        ).unwrap();

        let result = validate_project(forja).unwrap();
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("related_specs")));
    }

    #[test]
    fn warning_for_spec_without_criteria() {
        let dir = setup_forja_dir();
        let forja = dir.path();

        fs::write(
            forja.join("specs").join("bare.md"),
            "---\nid: bare\ntitle: Bare\ndescription: D\n---\n",
        )
        .unwrap();

        let result = validate_project(forja).unwrap();
        assert!(result.is_valid()); // warnings don't fail
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn empty_project_is_valid() {
        let dir = setup_forja_dir();
        let result = validate_project(dir.path()).unwrap();
        assert!(result.is_valid());
    }
}
