import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/EmptyState";
import {
  listProjects,
  listDecisions,
  type Project,
  type DecisionFile,
  type DecisionStatus,
} from "@/lib/tauri";

const STATUS_STYLES: Record<DecisionStatus, { dot: string; label: string }> = {
  proposed: { dot: "bg-yellow-400", label: "Proposed" },
  accepted: { dot: "bg-green-400", label: "Accepted" },
  deprecated: { dot: "bg-gray-300", label: "Deprecated" },
  superseded: { dot: "bg-orange-400", label: "Superseded" },
};

export function DecisionsPage() {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [decisions, setDecisions] = useState<DecisionFile[]>([]);
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
    listDecisions(project.path)
      .then(setDecisions)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [project]);

  function toggleExpanded(decisionId: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(decisionId)) next.delete(decisionId);
      else next.add(decisionId);
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
        Architecture decisions for this project.
      </p>

      {loading ? (
        <p className="text-muted-foreground">Loading decisions...</p>
      ) : decisions.length === 0 ? (
        <EmptyState
          title="No decisions found"
          description="Add decision records to .forja/decisions/ in your project to see them here."
        />
      ) : (
        <div className="grid gap-3">
          {decisions.map((decision) => {
            const style =
              STATUS_STYLES[decision.status] ?? STATUS_STYLES.proposed;
            const isOpen = expanded.has(decision.id);

            return (
              <Card
                key={decision.id}
                className="cursor-pointer"
                onClick={() => toggleExpanded(decision.id)}
              >
                <CardHeader className="py-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <span
                        className={`inline-block size-2.5 rounded-full shrink-0 ${style.dot}`}
                      />
                      <div className="min-w-0">
                        <CardTitle className="text-sm truncate">
                          {decision.id}: {decision.title}
                        </CardTitle>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          {decision.date}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      {decision.superseded_by && (
                        <Badge variant="secondary">
                          superseded by {decision.superseded_by}
                        </Badge>
                      )}
                      <span className="text-xs text-muted-foreground">
                        {style.label}
                      </span>
                    </div>
                  </div>
                  {decision.related_specs.length > 0 && (
                    <div className="flex gap-1 mt-2 flex-wrap">
                      {decision.related_specs.map((spec) => (
                        <span
                          key={spec}
                          className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground"
                        >
                          {spec}
                        </span>
                      ))}
                    </div>
                  )}
                </CardHeader>
                {isOpen && decision.body.trim() && (
                  <CardContent className="pt-0 pb-3">
                    <pre className="text-xs bg-muted rounded p-3 overflow-auto max-h-64 whitespace-pre-wrap">
                      {decision.body.trim()}
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
