use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::models::{Confidence, DetectedTech, DetectionSource};

/// Scan a project directory for technologies using filesystem markers.
pub fn scan(project_root: &Path) -> Vec<DetectedTech> {
    let mut techs: HashMap<String, DetectedTech> = HashMap::new();

    // Check marker files
    for rule in MARKER_RULES {
        let path = project_root.join(rule.path);
        if path.exists() {
            add_tech(
                &mut techs,
                rule.tech,
                rule.category,
                &format!("Found {}", rule.path),
                Confidence::High,
            );
        }
    }

    // Check glob-style markers (files with any extension)
    for rule in GLOB_MARKER_RULES {
        if has_file_matching(project_root, rule.prefix) {
            add_tech(
                &mut techs,
                rule.tech,
                rule.category,
                &format!("Found {}.*", rule.prefix),
                Confidence::High,
            );
        }
    }

    // Check directory markers
    for rule in DIR_MARKER_RULES {
        let path = project_root.join(rule.path);
        if path.is_dir() {
            add_tech(
                &mut techs,
                rule.tech,
                rule.category,
                &format!("Found {} directory", rule.path),
                rule.confidence,
            );
        }
    }

    // Parse package.json dependencies
    if let Some(deps) = parse_package_json_deps(project_root) {
        for rule in NPM_DEP_RULES {
            if deps.contains_key(rule.dep) {
                let version = deps.get(rule.dep).cloned();
                let tech = add_tech(
                    &mut techs,
                    rule.tech,
                    rule.category,
                    &format!("package.json dependency: {}", rule.dep),
                    Confidence::High,
                );
                if let Some(v) = version
                    && let Some(t) = techs.get_mut(tech.as_str())
                {
                    t.version = Some(v);
                }
            }
        }
    }

    // Parse Cargo.toml dependencies
    if let Some(deps) = parse_cargo_toml_deps(project_root) {
        for rule in CARGO_DEP_RULES {
            if deps.iter().any(|d| d == rule.dep) {
                add_tech(
                    &mut techs,
                    rule.tech,
                    rule.category,
                    &format!("Cargo.toml dependency: {}", rule.dep),
                    Confidence::High,
                );
            }
        }
    }

    // Shallow scan for file extensions (top-level + one level deep)
    scan_file_extensions(project_root, &mut techs);

    techs.into_values().collect()
}

// --- Rules tables ---

struct MarkerRule {
    path: &'static str,
    tech: &'static str,
    category: &'static str,
}

const MARKER_RULES: &[MarkerRule] = &[
    MarkerRule { path: "Cargo.toml", tech: "rust", category: "language" },
    MarkerRule { path: "tsconfig.json", tech: "typescript", category: "language" },
    MarkerRule { path: "go.mod", tech: "golang", category: "language" },
    MarkerRule { path: "requirements.txt", tech: "python", category: "language" },
    MarkerRule { path: "pyproject.toml", tech: "python", category: "language" },
    MarkerRule { path: "nest-cli.json", tech: "nestjs", category: "framework" },
    MarkerRule { path: "prisma/schema.prisma", tech: "prisma", category: "database" },
    MarkerRule { path: "package.json", tech: "nodejs", category: "runtime" },
];

struct GlobMarkerRule {
    prefix: &'static str,
    tech: &'static str,
    category: &'static str,
}

const GLOB_MARKER_RULES: &[GlobMarkerRule] = &[
    GlobMarkerRule { prefix: "next.config", tech: "nextjs", category: "framework" },
    GlobMarkerRule { prefix: "playwright.config", tech: "playwright", category: "testing" },
    GlobMarkerRule { prefix: "tailwind.config", tech: "tailwind", category: "styling" },
];

struct DirMarkerRule {
    path: &'static str,
    tech: &'static str,
    category: &'static str,
    confidence: Confidence,
}

const DIR_MARKER_RULES: &[DirMarkerRule] = &[
    DirMarkerRule { path: ".github/workflows", tech: "ci", category: "devops", confidence: Confidence::High },
    DirMarkerRule { path: ".gitlab-ci.yml", tech: "ci", category: "devops", confidence: Confidence::High },
];

struct NpmDepRule {
    dep: &'static str,
    tech: &'static str,
    category: &'static str,
}

const NPM_DEP_RULES: &[NpmDepRule] = &[
    NpmDepRule { dep: "@nestjs/core", tech: "nestjs", category: "framework" },
    NpmDepRule { dep: "next", tech: "nextjs", category: "framework" },
    NpmDepRule { dep: "@playwright/test", tech: "playwright", category: "testing" },
    NpmDepRule { dep: "drizzle-orm", tech: "drizzle", category: "database" },
    NpmDepRule { dep: "prisma", tech: "prisma", category: "database" },
    NpmDepRule { dep: "tailwindcss", tech: "tailwind", category: "styling" },
];

struct CargoDepRule {
    dep: &'static str,
    tech: &'static str,
    category: &'static str,
}

const CARGO_DEP_RULES: &[CargoDepRule] = &[
    CargoDepRule { dep: "sqlx", tech: "sqlx", category: "database" },
    CargoDepRule { dep: "tokio", tech: "tokio", category: "async" },
    CargoDepRule { dep: "actix-web", tech: "actix", category: "web" },
    CargoDepRule { dep: "axum", tech: "axum", category: "web" },
];

struct ExtensionRule {
    ext: &'static str,
    tech: &'static str,
    category: &'static str,
}

const EXTENSION_RULES: &[ExtensionRule] = &[
    ExtensionRule { ext: "rs", tech: "rust", category: "language" },
    ExtensionRule { ext: "ts", tech: "typescript", category: "language" },
    ExtensionRule { ext: "tsx", tech: "typescript", category: "language" },
    ExtensionRule { ext: "py", tech: "python", category: "language" },
    ExtensionRule { ext: "go", tech: "golang", category: "language" },
    ExtensionRule { ext: "sql", tech: "sql", category: "database" },
];

// --- Helpers ---

/// Add or merge a technology detection into the map.
/// Returns the tech name for further manipulation.
fn add_tech(
    techs: &mut HashMap<String, DetectedTech>,
    name: &str,
    category: &str,
    evidence: &str,
    confidence: Confidence,
) -> String {
    let entry = techs.entry(name.to_string()).or_insert_with(|| DetectedTech {
        name: name.to_string(),
        category: category.to_string(),
        evidence: Vec::new(),
        source: DetectionSource::Deterministic,
        confidence,
        version: None,
    });
    if !entry.evidence.contains(&evidence.to_string()) {
        entry.evidence.push(evidence.to_string());
    }
    // Upgrade confidence if higher
    if confidence > entry.confidence {
        entry.confidence = confidence;
    }
    name.to_string()
}

/// Check if a file matching `prefix.*` exists in the root directory.
fn has_file_matching(dir: &Path, prefix: &str) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries
        .filter_map(|e| e.ok())
        .any(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(prefix) && e.path().is_file()
        })
}

/// Parse package.json to extract dependency names and versions.
fn parse_package_json_deps(root: &Path) -> Option<HashMap<String, String>> {
    let content = fs::read_to_string(root.join("package.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mut deps = HashMap::new();
    for section in ["dependencies", "devDependencies"] {
        if let Some(obj) = json.get(section).and_then(|v| v.as_object()) {
            for (k, v) in obj {
                deps.insert(k.clone(), v.as_str().unwrap_or("*").to_string());
            }
        }
    }
    Some(deps)
}

/// Parse Cargo.toml to extract dependency names (simple text scan, no TOML parser).
fn parse_cargo_toml_deps(root: &Path) -> Option<Vec<String>> {
    let content = fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let mut deps = Vec::new();
    let mut in_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[dependencies")
            || trimmed.starts_with("[workspace.dependencies")
            || trimmed.starts_with("[dev-dependencies")
        {
            in_deps = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_deps = false;
            continue;
        }
        if in_deps
            && let Some(name) = trimmed.split('=').next()
        {
            let name = name.trim();
            if !name.is_empty() && !name.starts_with('#') {
                deps.push(name.to_string());
            }
        }
    }
    Some(deps)
}

/// Shallow scan for file extensions (root + one level deep).
fn scan_file_extensions(root: &Path, techs: &mut HashMap<String, DetectedTech>) {
    let dirs_to_scan = collect_scan_dirs(root);

    for dir in dirs_to_scan {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            for rule in EXTENSION_RULES {
                if ext == rule.ext {
                    add_tech(
                        techs,
                        rule.tech,
                        rule.category,
                        &format!("Found .{} files", rule.ext),
                        Confidence::Medium,
                    );
                    break;
                }
            }
        }
    }
}

/// Collect root + immediate subdirectories for shallow scanning.
fn collect_scan_dirs(root: &Path) -> Vec<std::path::PathBuf> {
    let mut dirs = vec![root.to_path_buf()];
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            // Skip hidden dirs, node_modules, target, vendor, dist
            if path.is_dir()
                && !name.starts_with('.')
                && !matches!(name.as_ref(), "node_modules" | "target" | "vendor" | "dist" | "build" | "__pycache__")
            {
                dirs.push(path);
            }
        }
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detect_rust_from_cargo_toml() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n[dependencies]\ntokio = \"1\"\n",
        ).unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "rust"), "should detect rust");
        assert!(techs.iter().any(|t| t.name == "tokio"), "should detect tokio dep");
    }

    #[test]
    fn detect_typescript_from_tsconfig() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "typescript"));
    }

    #[test]
    fn detect_python_from_requirements() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "flask\n").unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "python"));
    }

    #[test]
    fn detect_nextjs_from_package_json() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"next":"14.0.0"}}"#,
        ).unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "nextjs"), "should detect nextjs");
        assert!(techs.iter().any(|t| t.name == "nodejs"), "should detect nodejs from package.json");
    }

    #[test]
    fn detect_nextjs_from_config_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("next.config.mjs"), "export default {}").unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "nextjs"));
    }

    #[test]
    fn detect_golang_from_go_mod() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("go.mod"), "module example.com/test\n").unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "golang"));
    }

    #[test]
    fn detect_ci_from_github_workflows() {
        let dir = TempDir::new().unwrap();
        let workflows = dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join("ci.yml"), "name: CI\n").unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "ci"));
    }

    #[test]
    fn detect_prisma_from_schema() {
        let dir = TempDir::new().unwrap();
        let prisma_dir = dir.path().join("prisma");
        fs::create_dir_all(&prisma_dir).unwrap();
        fs::write(prisma_dir.join("schema.prisma"), "generator client {}\n").unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "prisma"));
    }

    #[test]
    fn detect_file_extensions_shallow() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.py"), "print('hello')\n").unwrap();

        let techs = scan(dir.path());
        assert!(techs.iter().any(|t| t.name == "python"));
    }

    #[test]
    fn empty_dir_returns_empty() {
        let dir = TempDir::new().unwrap();
        let techs = scan(dir.path());
        assert!(techs.is_empty());
    }

    #[test]
    fn multiple_evidence_merged() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "flask\n").unwrap();
        fs::write(dir.path().join("pyproject.toml"), "[tool.poetry]\n").unwrap();

        let techs = scan(dir.path());
        let python = techs.iter().find(|t| t.name == "python").unwrap();
        assert!(python.evidence.len() >= 2, "should have multiple evidence entries");
    }

    #[test]
    fn skips_node_modules() {
        let dir = TempDir::new().unwrap();
        let nm = dir.path().join("node_modules/some-pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("index.ts"), "export default {}").unwrap();

        // Only node_modules .ts files, no tsconfig — should not detect typescript
        let techs = scan(dir.path());
        assert!(!techs.iter().any(|t| t.name == "typescript"),
            "should not detect typescript from node_modules");
    }

    #[test]
    fn all_detections_are_deterministic() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"t\"").unwrap();

        let techs = scan(dir.path());
        for tech in &techs {
            assert_eq!(tech.source, DetectionSource::Deterministic);
        }
    }
}
