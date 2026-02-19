import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/EmptyState";
import {
  listProjects,
  listTracks,
  listSpecs,
  listRuns,
  validateProject,
  type Project,
  type TrackFile,
  type SpecFile,
  type RunLog,
  type ValidationResult,
} from "@/lib/tauri";

export function DashboardPage() {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [tracks, setTracks] = useState<TrackFile[]>([]);
  const [specs, setSpecs] = useState<SpecFile[]>([]);
  const [runs, setRuns] = useState<RunLog[]>([]);
  const [validation, setValidation] = useState<ValidationResult | null>(null);
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
    Promise.allSettled([
      listTracks(project.path).then(setTracks),
      listSpecs(project.path).then(setSpecs),
      listRuns(project.path).then(setRuns),
      validateProject(project.path).then(setValidation),
    ]).finally(() => setLoading(false));
  }, [project]);

  if (!id) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">No project selected.</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Loading dashboard...</p>
      </div>
    );
  }

  if (!project) {
    return (
      <div className="p-6">
        <EmptyState
          title="Project not found"
          description="The selected project could not be loaded."
        />
      </div>
    );
  }

  const activeSpecs = specs.filter(
    (s) => s.status !== "complete" && s.status !== "failed",
  );
  const recentRuns = runs.slice(0, 5);

  return (
    <div className="p-6 max-w-3xl">
      <h2 className="text-xl font-bold mb-1">{project.name}</h2>
      <p className="text-sm text-muted-foreground mb-6">Project overview</p>

      <div className="grid grid-cols-2 gap-4 mb-6">
        <Card>
          <CardHeader className="py-3">
            <CardTitle className="text-sm">Tracks</CardTitle>
          </CardHeader>
          <CardContent className="pt-0 pb-3">
            <p className="text-2xl font-bold">{tracks.length}</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="py-3">
            <CardTitle className="text-sm">Active Specs</CardTitle>
          </CardHeader>
          <CardContent className="pt-0 pb-3">
            <p className="text-2xl font-bold">
              {activeSpecs.length}
              <span className="text-sm font-normal text-muted-foreground ml-1">
                / {specs.length}
              </span>
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="py-3">
            <CardTitle className="text-sm">Runs</CardTitle>
          </CardHeader>
          <CardContent className="pt-0 pb-3">
            <p className="text-2xl font-bold">{runs.length}</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="py-3">
            <CardTitle className="text-sm">Validation</CardTitle>
          </CardHeader>
          <CardContent className="pt-0 pb-3">
            {validation ? (
              <div className="flex items-center gap-2">
                <span
                  className={`inline-block size-2.5 rounded-full ${validation.is_valid ? "bg-green-400" : "bg-red-400"}`}
                />
                <span className="text-sm">
                  {validation.is_valid ? "Valid" : `${validation.error_count} errors`}
                </span>
                {validation.warning_count > 0 && (
                  <span className="text-xs text-muted-foreground">
                    ({validation.warning_count} warnings)
                  </span>
                )}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">Not checked</p>
            )}
          </CardContent>
        </Card>
      </div>

      {tracks.length > 0 && (
        <div className="mb-6">
          <h3 className="text-sm font-medium mb-3">Track Progress</h3>
          <div className="space-y-3">
            {tracks.map((track) => {
              const done = track.items.filter(
                (i) => i.status === "done",
              ).length;
              const total = track.items.length;
              const pct = total > 0 ? Math.round((done / total) * 100) : 0;

              return (
                <div key={track.id}>
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm">{track.title}</span>
                    <span className="text-xs text-muted-foreground">
                      {done}/{total}
                    </span>
                  </div>
                  <div className="h-2 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full bg-primary rounded-full transition-all"
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {recentRuns.length > 0 && (
        <div>
          <h3 className="text-sm font-medium mb-3">Recent Runs</h3>
          <div className="space-y-2">
            {recentRuns.map((run, i) => (
              <div
                key={i}
                className="flex items-center justify-between py-2 px-3 rounded bg-muted/50"
              >
                <div className="flex items-center gap-2">
                  <span
                    className={`inline-block size-2 rounded-full ${
                      run.status === "complete"
                        ? "bg-green-400"
                        : run.status === "failed"
                          ? "bg-red-400"
                          : "bg-blue-400 animate-pulse"
                    }`}
                  />
                  <span className="text-sm">{run.spec_id}</span>
                </div>
                <div className="flex items-center gap-2">
                  {run.duration_seconds != null && (
                    <span className="text-xs text-muted-foreground">
                      {run.duration_seconds}s
                    </span>
                  )}
                  <Badge variant="secondary">{run.status}</Badge>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {tracks.length === 0 && specs.length === 0 && runs.length === 0 && (
        <EmptyState
          title="No data yet"
          description="Initialize .forja/ in your project to start tracking specs, tracks, and runs."
        />
      )}
    </div>
  );
}
