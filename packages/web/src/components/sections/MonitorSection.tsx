import { SectionHeader } from "@/components/ui/SectionHeader";

const tasks = [
  { agent: "researcher", task: "Analyzing codebase patterns", status: "completed", color: "bg-monitor" },
  { agent: "coder", task: "Implementing auth middleware", status: "in_progress", color: "bg-accent" },
  { agent: "tester", task: "Waiting for code phase", status: "pending", color: "bg-text-subtle" },
  { agent: "reviewer", task: "Queued", status: "pending", color: "bg-text-subtle" },
  { agent: "deployer", task: "Queued", status: "pending", color: "bg-text-subtle" },
];

export function MonitorSection() {
  return (
    <section id="monitor" className="py-20 md:py-28 bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="06 | Monitor"
          title="Watch your agents work."
          subtitle="Real-time dashboard showing every agent team, task status, and execution log. Streams updates via SSE — no polling, no refresh."
        />

        {/* Status table mockup */}
        <div className="max-w-3xl mx-auto mt-12 rounded-lg border border-border bg-bg-card overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b border-border bg-bg-elevated">
            <span className="text-sm font-mono text-text-muted">full-product team</span>
            <span className="text-xs font-mono text-monitor">live</span>
          </div>
          <div className="divide-y divide-border">
            {tasks.map((t) => (
              <div key={t.agent} className="flex items-center gap-4 px-4 py-3">
                <div className={`w-2 h-2 rounded-full ${t.color} flex-shrink-0`} />
                <span className="text-sm font-mono text-accent w-24 flex-shrink-0">
                  {t.agent}
                </span>
                <span className="text-sm text-text-muted flex-1 truncate">
                  {t.task}
                </span>
                <span className={`text-xs font-mono ${
                  t.status === "completed" ? "text-monitor" :
                  t.status === "in_progress" ? "text-accent" :
                  "text-text-subtle"
                }`}>
                  {t.status}
                </span>
              </div>
            ))}
          </div>
          <div className="px-4 py-2 border-t border-border text-xs font-mono text-text-subtle text-right">
            last update: 2s ago
          </div>
        </div>
      </div>
    </section>
  );
}
