import { SectionHeader } from "@/components/ui/SectionHeader";
import { Search, Compass, Settings, Wrench, CheckCircle, Shield, BarChart2, Rocket } from "lucide-react";

const stats = [
  { number: "25", label: "Agents" },
  { number: "5", label: "Phases" },
  { number: "8", label: "Stacks" },
  { number: "4", label: "Team Configs" },
];

const highlights = [
  { icon: Search, name: "Codebase Explorer", id: "research/codebase/explorer" },
  { icon: Compass, name: "Architecture Planner", id: "research/architecture/planner" },
  { icon: Settings, name: "Rust Feature Agent", id: "code/rust/feature" },
  { icon: Wrench, name: "TypeScript Feature Agent", id: "code/typescript/feature" },
  { icon: CheckCircle, name: "TDD Workflow", id: "test/tdd/workflow" },
  { icon: Shield, name: "Security Auditor", id: "review/security/auditor" },
  { icon: BarChart2, name: "Performance Analyzer", id: "review/performance/analyzer" },
  { icon: Rocket, name: "Deploy Verify", id: "deploy/verify/checker" },
];

export function CatalogSection() {
  return (
    <section id="catalog" className="py-20 md:py-28 bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="08 | Skill Catalog"
          title="25 agents, ready to install"
          subtitle="Purpose-built agents for every language and every phase. Browse the catalog or search by keyword."
        />

        {/* Stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-12">
          {stats.map((s) => (
            <div key={s.label} className="text-center p-4 rounded-lg border border-border bg-bg-card">
              <div className="text-3xl font-bold text-accent">{s.number}</div>
              <div className="text-sm text-text-muted mt-1">{s.label}</div>
            </div>
          ))}
        </div>

        {/* Highlights grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
          {highlights.map((h) => (
            <div key={h.id} className="flex items-center gap-3 p-3 rounded-lg border border-border bg-bg-card hover:bg-bg-elevated transition-colors">
              <div className="w-8 h-8 rounded-md bg-accent-dim flex items-center justify-center flex-shrink-0">
                <h.icon size={16} className="text-accent" />
              </div>
              <div className="min-w-0">
                <div className="text-sm font-medium text-text truncate">{h.name}</div>
                <div className="text-xs font-mono text-text-subtle truncate">{h.id}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
