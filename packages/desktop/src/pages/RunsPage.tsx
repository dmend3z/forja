import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/EmptyState";
import {
  listProjects,
  listRuns,
  type Project,
  type RunLog,
  type RunStatus,
} from "@/lib/tauri";

const STATUS_STYLES: Record<RunStatus, { dot: string; label: string }> = {
  running: { dot: "bg-blue-400 animate-pulse", label: "Running" },
  complete: { dot: "bg-green-400", label: "Complete" },
  failed: { dot: "bg-red-400", label: "Failed" },
};

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}m ${secs}s`;
}

export function RunsPage() {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [runs, setRuns] = useState<RunLog[]>([]);
  const [expanded, setExpanded] = useState<Set<number>>(new Set());
  const [loading, setLoading] = useState(true);

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
    setLoading(true);
    listRuns(project.path)
      .then(setRuns)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [project]);

  function toggleExpanded(index: number) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return next;
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
      <p className="text-sm text-muted-foreground mb-6">
        Agent execution runs for this project.
      </p>

      {loading ? (
        <p className="text-muted-foreground">Loading runs...</p>
      ) : runs.length === 0 ? (
        <EmptyState
          title="No runs found"
          description="Run logs are created when agents execute specs. Check .forja/runs/ for run history."
        />
      ) : (
        <div className="grid gap-3">
          {runs.map((run, i) => {
            const style =
              STATUS_STYLES[run.status] ?? STATUS_STYLES.running;
            const isOpen = expanded.has(i);

            return (
              <Card
                key={i}
                className={run.body.trim() ? "cursor-pointer" : ""}
                onClick={() => run.body.trim() && toggleExpanded(i)}
              >
                <CardHeader className="py-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <span
                        className={`inline-block size-2.5 rounded-full shrink-0 ${style.dot}`}
                      />
                      <div className="min-w-0">
                        <CardTitle className="text-sm truncate">
                          {run.spec_id}
                        </CardTitle>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          {run.agent} - {new Date(run.started_at).toLocaleString()}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      {run.plan_id && (
                        <Badge variant="secondary">{run.plan_id}</Badge>
                      )}
                      {run.duration_seconds != null && (
                        <span className="text-xs text-muted-foreground">
                          {formatDuration(run.duration_seconds)}
                        </span>
                      )}
                      {run.exit_code != null && (
                        <span
                          className={`text-xs font-mono ${run.exit_code === 0 ? "text-green-600" : "text-red-600"}`}
                        >
                          exit {run.exit_code}
                        </span>
                      )}
                      <span className="text-xs text-muted-foreground">
                        {style.label}
                      </span>
                    </div>
                  </div>
                </CardHeader>
                {isOpen && run.body.trim() && (
                  <CardContent className="pt-0 pb-3">
                    <pre className="text-xs bg-muted rounded p-3 overflow-auto max-h-64 whitespace-pre-wrap">
                      {run.body.trim()}
                    </pre>
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
