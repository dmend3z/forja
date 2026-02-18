import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/EmptyState";
import { ProjectTabBar } from "@/components/ProjectTabBar";
import {
  listProjects,
  listSpecs,
  type Project,
  type SpecFile,
  type SpecStatus,
} from "@/lib/tauri";

const STATUS_STYLES: Record<SpecStatus, { dot: string; label: string }> = {
  draft: { dot: "bg-gray-400", label: "Draft" },
  planning: { dot: "bg-yellow-400", label: "Planning" },
  ready: { dot: "bg-blue-400", label: "Ready" },
  executing: { dot: "bg-blue-400 animate-pulse", label: "Executing" },
  complete: { dot: "bg-green-400", label: "Complete" },
  failed: { dot: "bg-red-400", label: "Failed" },
};

const PRIORITY_COLORS: Record<string, string> = {
  high: "bg-red-100 text-red-700",
  medium: "bg-yellow-100 text-yellow-700",
  low: "bg-green-100 text-green-700",
};

function buildSparkPrompt(spec: SpecFile): string {
  const parts = [`# ${spec.title}`, spec.description];

  if (spec.requirements.length > 0) {
    parts.push(
      `\n## Requirements\n${spec.requirements.map((r) => `- ${r}`).join("\n")}`,
    );
  }
  if (spec.constraints.length > 0) {
    parts.push(
      `\n## Constraints\n${spec.constraints.map((c) => `- ${c}`).join("\n")}`,
    );
  }
  if (spec.success_criteria.length > 0) {
    parts.push(
      `\n## Success Criteria\n${spec.success_criteria.map((c) => `- ${c}`).join("\n")}`,
    );
  }
  if (spec.body.trim()) {
    parts.push(`\n## Details\n${spec.body.trim()}`);
  }

  return parts.join("\n");
}

export function SpecsPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [specs, setSpecs] = useState<SpecFile[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!id) return;
    listProjects()
      .then((result) => {
        const found = result.projects.find((p) => p.id === id);
        setProject(found ?? null);
      })
      .catch(console.error);
  }, [id]);

  useEffect(() => {
    if (!project) return;
    listSpecs(project.path)
      .then(setSpecs)
      .catch(console.error);
  }, [project]);

  function toggleExpanded(specId: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(specId)) next.delete(specId);
      else next.add(specId);
      return next;
    });
  }

  function handleRunAsSpark(spec: SpecFile) {
    navigate(`/project/${id}/sparks`, {
      state: { prefill: buildSparkPrompt(spec) },
    });
  }

  if (!id) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">No project selected.</p>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-xl font-bold mb-1">
        {project ? project.name : "Loading..."}
      </h2>
      <p className="text-sm text-muted-foreground mb-4">
        Implementation specs for this project.
      </p>

      <ProjectTabBar />

      {specs.length === 0 ? (
        <EmptyState
          title="No specs found"
          description="Add markdown spec files to docs/specs/ in your project to see them here."
        />
      ) : (
        <div className="grid gap-3">
          {specs.map((spec) => {
            const style = STATUS_STYLES[spec.status] ?? STATUS_STYLES.draft;
            const isOpen = expanded.has(spec.id);

            return (
              <Card
                key={spec.id}
                className="cursor-pointer"
                onClick={() => toggleExpanded(spec.id)}
              >
                <CardHeader className="py-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <span
                        className={`inline-block size-2.5 rounded-full shrink-0 ${style.dot}`}
                      />
                      <div className="min-w-0">
                        <CardTitle className="text-sm truncate">
                          {spec.title}
                        </CardTitle>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          {spec.description}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      {spec.priority && (
                        <span
                          className={`text-xs font-medium px-2 py-0.5 rounded ${PRIORITY_COLORS[spec.priority] ?? "bg-secondary text-secondary-foreground"}`}
                        >
                          {spec.priority}
                        </span>
                      )}
                      <span className="text-xs text-muted-foreground">
                        {style.label}
                      </span>
                    </div>
                  </div>
                  {spec.tags.length > 0 && (
                    <div className="flex gap-1 mt-2 flex-wrap">
                      {spec.tags.map((tag) => (
                        <span
                          key={tag}
                          className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </CardHeader>
                {isOpen && (
                  <CardContent className="pt-0 pb-3 space-y-3">
                    {spec.requirements.length > 0 && (
                      <div>
                        <h4 className="text-xs font-medium mb-1">
                          Requirements
                        </h4>
                        <ul className="text-xs text-muted-foreground list-disc pl-4 space-y-0.5">
                          {spec.requirements.map((r, i) => (
                            <li key={i}>{r}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {spec.constraints.length > 0 && (
                      <div>
                        <h4 className="text-xs font-medium mb-1">
                          Constraints
                        </h4>
                        <ul className="text-xs text-muted-foreground list-disc pl-4 space-y-0.5">
                          {spec.constraints.map((c, i) => (
                            <li key={i}>{c}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {spec.success_criteria.length > 0 && (
                      <div>
                        <h4 className="text-xs font-medium mb-1">
                          Success Criteria
                        </h4>
                        <ul className="text-xs text-muted-foreground list-disc pl-4 space-y-0.5">
                          {spec.success_criteria.map((c, i) => (
                            <li key={i}>{c}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {spec.body.trim() && (
                      <pre className="text-xs bg-muted rounded p-3 overflow-auto max-h-48 whitespace-pre-wrap">
                        {spec.body.trim()}
                      </pre>
                    )}
                    <Button
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRunAsSpark(spec);
                      }}
                    >
                      Run as Spark
                    </Button>
                  </CardContent>
                )}
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}
