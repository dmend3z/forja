import { SectionHeader } from "@/components/ui/SectionHeader";

const phases = [
  { number: "1", name: "Research", count: "4 skills", desc: "Codebase exploration, docs research, architecture planning, plan orchestration" },
  { number: "2", name: "Code", count: "8 skills", desc: "Language-specific agents for TypeScript, Python, Go, Rust, Next.js, NestJS, database, general" },
  { number: "3", name: "Test", count: "4 skills", desc: "TDD workflow, test generation, E2E with Playwright, coverage analysis" },
  { number: "4", name: "Review", count: "5 skills", desc: "Code quality, security audit, performance analysis, PR workflow, code simplification" },
  { number: "5", name: "Deploy", count: "3 skills", desc: "Conventional git commits, PR creation via gh CLI, post-deploy verification" },
  { number: "+", name: "Teams", count: "4 configs", desc: "Multi-agent team configurations for full product, solo sprint, quick fix, and refactoring" },
];

export function PhasesSection() {
  return (
    <section id="phases" className="py-20 md:py-28">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="07 | The 5-Phase Workflow"
          title="Agents for every stage of development"
          subtitle="forja organizes agents around the dev phases that matter — from initial research to production deploy."
        />

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {phases.map((p) => (
            <div key={p.number} className="p-5 rounded-lg border border-border bg-bg-card hover:bg-bg-elevated transition-colors">
              <div className="flex items-center gap-3 mb-3">
                <span className="w-8 h-8 rounded-full bg-accent-dim flex items-center justify-center text-sm font-mono font-semibold text-accent">
                  {p.number}
                </span>
                <h3 className="text-lg font-semibold text-text">{p.name}</h3>
              </div>
              <span className="inline-block text-xs font-mono text-accent mb-2">{p.count}</span>
              <p className="text-sm text-text-muted leading-relaxed">{p.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
