use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::models::decision;
use crate::models::spec;
use crate::models::spec::SpecStatus;
use crate::models::track;

/// Generate CONTEXT.md — a concise AI-readable summary of current project state.
pub fn generate_context(forja_dir: &Path) -> Result<String> {
    let mut out = String::new();

    out.push_str("# Forja Project Context\n");
    out.push_str("> Auto-generated. Add `Read .forja/CONTEXT.md` to your CLAUDE.md.\n\n");

    // Active work
    let tracks = track::discover_tracks(&forja_dir.join("tracks")).unwrap_or_default();
    let specs_dir = forja_dir.join("specs");
    let specs = if specs_dir.exists() {
        spec::discover_specs(&specs_dir).unwrap_or_default()
    } else {
        Vec::new()
    };

    out.push_str("## Active Work\n");

    for t in &tracks {
        let (done, total) = t.progress();
        out.push_str(&format!(
            "- Track: {} ({}/{} done)\n",
            t.title(),
            done,
            total
        ));
    }

    let in_progress: Vec<&spec::SpecFile> = specs
        .iter()
        .filter(|s| matches!(s.status, SpecStatus::InProgress | SpecStatus::Executing))
        .collect();

    if !in_progress.is_empty() {
        out.push_str(&format!(
            "- In-progress: {}\n",
            in_progress
                .iter()
                .map(|s| s.id())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let blocked: Vec<&spec::SpecFile> = specs
        .iter()
        .filter(|s| matches!(s.status, SpecStatus::Blocked))
        .collect();

    if blocked.is_empty() {
        out.push_str("- Blocked: none\n");
    } else {
        out.push_str(&format!(
            "- Blocked: {}\n",
            blocked
                .iter()
                .map(|s| s.id())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    out.push('\n');

    // Key decisions
    let decisions = decision::discover_decisions(&forja_dir.join("decisions")).unwrap_or_default();
    let accepted: Vec<_> = decisions
        .iter()
        .filter(|d| d.frontmatter.status == decision::DecisionStatus::Accepted)
        .collect();

    if !accepted.is_empty() {
        out.push_str("## Key Decisions\n");
        for d in &accepted {
            out.push_str(&format!("- {} (decision {})\n", d.title(), d.id()));
        }
        out.push('\n');
    }

    // Priorities — specs sorted by priority (high > medium > low)
    let mut prioritized: Vec<&spec::SpecFile> = specs
        .iter()
        .filter(|s| {
            !matches!(
                s.status,
                SpecStatus::Done | SpecStatus::Complete | SpecStatus::Failed
            )
        })
        .collect();

    prioritized.sort_by(|a, b| {
        let priority_rank = |s: &spec::SpecFile| match s.frontmatter.priority.as_deref() {
            Some("high") => 0,
            Some("medium") => 1,
            Some("low") => 2,
            _ => 3,
        };
        priority_rank(a).cmp(&priority_rank(b))
    });

    if !prioritized.is_empty() {
        out.push_str("## Priorities\n");
        for (i, s) in prioritized.iter().enumerate() {
            let priority = s.frontmatter.priority.as_deref().unwrap_or("none");
            out.push_str(&format!(
                "{}. [{}] {} - {}\n",
                i + 1,
                priority,
                s.id(),
                s.title()
            ));
        }
        out.push('\n');
    }

    Ok(out)
}

/// Write CONTEXT.md to .forja/CONTEXT.md.
pub fn write_context(forja_dir: &Path) -> Result<()> {
    let content = generate_context(forja_dir)?;
    fs::write(forja_dir.join("CONTEXT.md"), content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_forja() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let forja = dir.path();
        fs::create_dir_all(forja.join("tracks")).unwrap();
        fs::create_dir_all(forja.join("specs")).unwrap();
        fs::create_dir_all(forja.join("decisions")).unwrap();

        fs::write(
            forja.join("tracks").join("track-mvp.md"),
            "---\nid: mvp\ntitle: MVP\ndescription: D\nstatus: in-progress\ncreated: \"2026-01-01\"\n---\n| ID | Task | Status | Spec |\n|---|---|---|---|\n| M1 | Auth | done | auth |\n| M2 | API | todo | api |\n",
        ).unwrap();

        fs::write(
            forja.join("specs").join("auth.md"),
            "---\nid: auth\ntitle: Auth\ndescription: D\nstatus: done\npriority: high\n---\n",
        ).unwrap();

        fs::write(
            forja.join("specs").join("api.md"),
            "---\nid: api\ntitle: API\ndescription: D\nstatus: in-progress\npriority: high\n---\n",
        ).unwrap();

        fs::write(
            forja.join("decisions").join("001.md"),
            "---\nid: \"001\"\ntitle: JWT in cookies\nstatus: accepted\ndate: \"2026-02-18\"\n---\nBody.",
        ).unwrap();

        dir
    }

    #[test]
    fn generate_context_includes_sections() {
        let dir = setup_forja();
        let ctx = generate_context(dir.path()).unwrap();

        assert!(ctx.contains("# Forja Project Context"));
        assert!(ctx.contains("## Active Work"));
        assert!(ctx.contains("Track: MVP"));
        assert!(ctx.contains("1/2 done"));
        assert!(ctx.contains("In-progress: api"));
        assert!(ctx.contains("Blocked: none"));
        assert!(ctx.contains("## Key Decisions"));
        assert!(ctx.contains("JWT in cookies"));
        assert!(ctx.contains("## Priorities"));
        assert!(ctx.contains("[high] api - API"));
    }

    #[test]
    fn write_context_creates_file() {
        let dir = setup_forja();
        write_context(dir.path()).unwrap();

        let path = dir.path().join("CONTEXT.md");
        assert!(path.exists());
    }
}
