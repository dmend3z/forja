import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/EmptyState";
import {
  listProjects,
  listTracks,
  type Project,
  type TrackFile,
  type TrackStatus,
} from "@/lib/tauri";

const STATUS_STYLES: Record<TrackStatus, { dot: string; label: string }> = {
  draft: { dot: "bg-gray-400", label: "Draft" },
  "in-progress": { dot: "bg-blue-400 animate-pulse", label: "In Progress" },
  complete: { dot: "bg-green-400", label: "Complete" },
  archived: { dot: "bg-gray-300", label: "Archived" },
};

const ITEM_STATUS_COLORS: Record<string, string> = {
  done: "bg-green-100 text-green-700",
  "in-progress": "bg-blue-100 text-blue-700",
  todo: "bg-gray-100 text-gray-700",
  blocked: "bg-red-100 text-red-700",
};

export function TracksPage() {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [tracks, setTracks] = useState<TrackFile[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
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
    listTracks(project.path)
      .then(setTracks)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [project]);

  function toggleExpanded(trackId: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(trackId)) next.delete(trackId);
      else next.add(trackId);
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
        Work tracks for this project.
      </p>

      {loading ? (
        <p className="text-muted-foreground">Loading tracks...</p>
      ) : tracks.length === 0 ? (
        <EmptyState
          title="No tracks found"
          description="Add track files to .forja/tracks/ in your project to see them here."
        />
      ) : (
        <div className="grid gap-3">
          {tracks.map((track) => {
            const style =
              STATUS_STYLES[track.status] ?? STATUS_STYLES.draft;
            const isOpen = expanded.has(track.id);
            const done = track.items.filter(
              (i) => i.status === "done",
            ).length;
            const total = track.items.length;
            const pct = total > 0 ? Math.round((done / total) * 100) : 0;

            return (
              <Card
                key={track.id}
                className="cursor-pointer"
                onClick={() => toggleExpanded(track.id)}
              >
                <CardHeader className="py-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <span
                        className={`inline-block size-2.5 rounded-full shrink-0 ${style.dot}`}
                      />
                      <div className="min-w-0">
                        <CardTitle className="text-sm truncate">
                          {track.title}
                        </CardTitle>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          {track.description}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-3 shrink-0">
                      {total > 0 && (
                        <span className="text-xs text-muted-foreground">
                          {done}/{total} ({pct}%)
                        </span>
                      )}
                      {track.priority && (
                        <Badge variant="secondary">{track.priority}</Badge>
                      )}
                      <span className="text-xs text-muted-foreground">
                        {style.label}
                      </span>
                    </div>
                  </div>
                  {total > 0 && (
                    <div className="h-1.5 bg-muted rounded-full overflow-hidden mt-2">
                      <div
                        className="h-full bg-primary rounded-full transition-all"
                        style={{ width: `${pct}%` }}
                      />
                    </div>
                  )}
                </CardHeader>
                {isOpen && track.items.length > 0 && (
                  <CardContent className="pt-0 pb-3">
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="text-xs text-muted-foreground border-b">
                          <th className="text-left py-1 pr-3">ID</th>
                          <th className="text-left py-1 pr-3">Task</th>
                          <th className="text-left py-1 pr-3">Status</th>
                          <th className="text-left py-1">Spec</th>
                        </tr>
                      </thead>
                      <tbody>
                        {track.items.map((item) => (
                          <tr key={item.id} className="border-b last:border-0">
                            <td className="py-1.5 pr-3 text-xs font-mono">
                              {item.id}
                            </td>
                            <td className="py-1.5 pr-3">{item.task}</td>
                            <td className="py-1.5 pr-3">
                              <span
                                className={`text-xs font-medium px-1.5 py-0.5 rounded ${ITEM_STATUS_COLORS[item.status] ?? "bg-secondary text-secondary-foreground"}`}
                              >
                                {item.status}
                              </span>
                            </td>
                            <td className="py-1.5 text-xs font-mono text-muted-foreground">
                              {item.spec}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </CardContent>
                )}
                {isOpen && track.items.length === 0 && (
                  <CardContent className="pt-0 pb-3">
                    <p className="text-xs text-muted-foreground">
                      No items in this track.
                    </p>
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
