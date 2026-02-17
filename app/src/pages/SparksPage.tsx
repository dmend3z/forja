import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/EmptyState";
import {
  listProjects,
  listSparks,
  startSpark,
  type Project,
  type SparkInfo,
  type SparkType,
} from "@/lib/tauri";

const SPARK_TYPE_LABELS: Record<SparkType, string> = {
  task: "Run Task",
  quick_fix: "Quick Fix",
  plan: "Plan",
};

const STATUS_STYLES: Record<string, { dot: string; label: string }> = {
  starting: { dot: "bg-yellow-400", label: "Starting" },
  running: { dot: "bg-blue-400 animate-pulse", label: "Running" },
  idle: { dot: "bg-gray-400", label: "Idle" },
  stopped: { dot: "bg-green-400", label: "Completed" },
  failed: { dot: "bg-red-400", label: "Failed" },
};

function isTerminal(status: string) {
  return status === "stopped" || status === "failed";
}

export function SparksPage() {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [sparks, setSparks] = useState<SparkInfo[]>([]);
  const [description, setDescription] = useState("");
  const [loading, setLoading] = useState(false);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const pollingRef = useRef(true);

  // Load project on mount
  useEffect(() => {
    if (!id) return;
    listProjects()
      .then((result) => {
        const found = result.projects.find((p) => p.id === id);
        setProject(found ?? null);
      })
      .catch(console.error);
  }, [id]);

  // Poll sparks
  useEffect(() => {
    if (!id) return;

    let timer: ReturnType<typeof setInterval>;

    function poll() {
      listSparks(id!).then(setSparks).catch(console.error);
    }

    poll();
    timer = setInterval(() => {
      if (pollingRef.current) poll();
    }, 3000);

    return () => clearInterval(timer);
  }, [id]);

  // Pause polling when all sparks are terminal
  useEffect(() => {
    if (sparks.length === 0) {
      pollingRef.current = false;
      return;
    }
    pollingRef.current = !sparks.every((s) => isTerminal(s.status));
  }, [sparks]);

  async function handleStart(sparkType: SparkType) {
    if (!project || !description.trim()) return;

    setLoading(true);
    try {
      await startSpark(project.id, sparkType, description.trim(), project.path);
      setDescription("");
      pollingRef.current = true;
      // Immediate refresh
      const updated = await listSparks(project.id);
      setSparks(updated);
    } catch (e) {
      console.error("Failed to start spark:", e);
    } finally {
      setLoading(false);
    }
  }

  function toggleExpanded(sparkId: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(sparkId)) next.delete(sparkId);
      else next.add(sparkId);
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
        Run a spark to execute a task in this project.
      </p>

      {/* Input form */}
      <div className="mb-8">
        <textarea
          className="w-full rounded-lg border bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring resize-none"
          rows={3}
          placeholder="Describe what you want to do..."
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          disabled={loading}
        />
        <div className="flex gap-2 mt-2">
          {(Object.entries(SPARK_TYPE_LABELS) as [SparkType, string][]).map(
            ([type, label]) => (
              <Button
                key={type}
                size="sm"
                variant={type === "task" ? "default" : "outline"}
                disabled={loading || !description.trim()}
                onClick={() => handleStart(type)}
              >
                {label}
              </Button>
            ),
          )}
        </div>
      </div>

      {/* Spark list */}
      {sparks.length === 0 ? (
        <EmptyState
          title="No sparks yet"
          description="Enter a description above and click a button to run your first spark."
        />
      ) : (
        <div className="grid gap-3">
          {sparks.map((spark) => {
            const style = STATUS_STYLES[spark.status] ?? STATUS_STYLES.starting;
            const hasOutput = spark.output || spark.error;
            const isOpen = expanded.has(spark.id);

            return (
              <Card
                key={spark.id}
                className={hasOutput ? "cursor-pointer" : ""}
                onClick={() => hasOutput && toggleExpanded(spark.id)}
              >
                <CardHeader className="py-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <span
                        className={`inline-block size-2.5 rounded-full shrink-0 ${style.dot}`}
                      />
                      <div className="min-w-0">
                        <CardTitle className="text-sm truncate">
                          {spark.description}
                        </CardTitle>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          {new Date(spark.created_at).toLocaleTimeString()}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="text-xs font-medium px-2 py-0.5 rounded bg-secondary text-secondary-foreground">
                        {SPARK_TYPE_LABELS[spark.spark_type]}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {style.label}
                      </span>
                    </div>
                  </div>
                </CardHeader>
                {isOpen && hasOutput && (
                  <CardContent className="pt-0 pb-3">
                    <pre className="text-xs bg-muted rounded p-3 overflow-auto max-h-64 whitespace-pre-wrap">
                      {spark.error ?? spark.output}
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
