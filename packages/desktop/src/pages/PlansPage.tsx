import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/EmptyState";
import {
  listProjects,
  listPlans,
  type Project,
  type PlanMetadata,
  type PlanStatus,
} from "@/lib/tauri";

const STATUS_STYLES: Record<PlanStatus, { dot: string; label: string }> = {
  pending: { dot: "bg-yellow-400", label: "Pending" },
  executed: { dot: "bg-green-400", label: "Executed" },
  archived: { dot: "bg-gray-300", label: "Archived" },
};

export function PlansPage() {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [plans, setPlans] = useState<PlanMetadata[]>([]);
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
    listPlans(project.path)
      .then(setPlans)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [project]);

  function toggleExpanded(planId: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(planId)) next.delete(planId);
      else next.add(planId);
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
        Execution plans for this project.
      </p>

      {loading ? (
        <p className="text-muted-foreground">Loading plans...</p>
      ) : plans.length === 0 ? (
        <EmptyState
          title="No plans found"
          description="Plans are generated from specs. Add JSON plan files to .forja/plans/ to see them here."
        />
      ) : (
        <div className="grid gap-3">
          {plans.map((plan) => {
            const style =
              STATUS_STYLES[plan.status] ?? STATUS_STYLES.pending;
            const isOpen = expanded.has(plan.id);

            return (
              <Card
                key={plan.id}
                className="cursor-pointer"
                onClick={() => toggleExpanded(plan.id)}
              >
                <CardHeader className="py-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <span
                        className={`inline-block size-2.5 rounded-full shrink-0 ${style.dot}`}
                      />
                      <div className="min-w-0">
                        <CardTitle className="text-sm truncate">
                          {plan.task}
                        </CardTitle>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          {plan.team_size} / {plan.profile}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      {plan.source_spec && (
                        <Badge variant="secondary">
                          {plan.source_spec}
                        </Badge>
                      )}
                      <span className="text-xs text-muted-foreground">
                        {style.label}
                      </span>
                    </div>
                  </div>
                </CardHeader>
                {isOpen && (
                  <CardContent className="pt-0 pb-3 space-y-3">
                    <div className="grid grid-cols-2 gap-2 text-xs">
                      <div>
                        <span className="text-muted-foreground">ID: </span>
                        <span className="font-mono">{plan.id}</span>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Created: </span>
                        <span>{plan.created}</span>
                      </div>
                      {plan.stack && (
                        <div>
                          <span className="text-muted-foreground">Stack: </span>
                          <span>
                            {plan.stack.language}
                            {plan.stack.framework
                              ? ` / ${plan.stack.framework}`
                              : ""}
                          </span>
                        </div>
                      )}
                    </div>

                    {plan.agents.length > 0 && (
                      <div>
                        <h4 className="text-xs font-medium mb-1">Agents</h4>
                        <div className="flex gap-1 flex-wrap">
                          {plan.agents.map((agent, i) => (
                            <Badge key={i} variant="secondary">
                              {agent.role}
                            </Badge>
                          ))}
                        </div>
                      </div>
                    )}

                    {plan.phases.length > 0 && (
                      <div>
                        <h4 className="text-xs font-medium mb-1">
                          Phases ({plan.phases.length})
                        </h4>
                        <div className="space-y-1">
                          {plan.phases.map((phase, i) => (
                            <div
                              key={i}
                              className="flex items-center justify-between py-1 px-2 rounded bg-muted/50 text-xs"
                            >
                              <span>{phase.name}</span>
                              <span className="text-muted-foreground">
                                {phase.agent_role}
                              </span>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {plan.quality_gates.length > 0 && (
                      <div>
                        <h4 className="text-xs font-medium mb-1">
                          Quality Gates
                        </h4>
                        <ul className="text-xs text-muted-foreground list-disc pl-4 space-y-0.5">
                          {plan.quality_gates.map((gate, i) => (
                            <li key={i}>{gate}</li>
                          ))}
                        </ul>
                      </div>
                    )}
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
