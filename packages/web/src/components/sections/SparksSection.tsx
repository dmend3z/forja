import { SectionHeader } from "@/components/ui/SectionHeader";
import { FileText, GitBranch, Code2, TestTube2, Rocket } from "lucide-react";

const phases = [
  { icon: FileText, label: "Spec", color: "bg-sparks/20 text-sparks" },
  { icon: GitBranch, label: "Plan", color: "bg-sparks/20 text-sparks" },
  { icon: Code2, label: "Code", color: "bg-sparks/20 text-sparks" },
  { icon: TestTube2, label: "Test", color: "bg-sparks/20 text-sparks" },
  { icon: Rocket, label: "Ship", color: "bg-sparks/20 text-sparks" },
];

export function SparksSection() {
  return (
    <section id="sparks" className="py-20 md:py-28">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="03 | Sparks"
          title="Write a spec. forja handles the rest."
          subtitle="Spec-driven execution pipeline. Describe your feature in markdown, forja plans it, codes it phase-by-phase with quality gates and auto-retry. Resume from any checkpoint."
        />

        {/* Pipeline diagram */}
        <div className="flex flex-wrap items-center justify-center gap-3 md:gap-4 mt-12">
          {phases.map((phase, i) => (
            <div key={phase.label} className="flex items-center gap-3 md:gap-4">
              <div className="flex flex-col items-center gap-2">
                <div className={`w-12 h-12 md:w-14 md:h-14 rounded-xl ${phase.color} flex items-center justify-center`}>
                  <phase.icon size={22} />
                </div>
                <span className="text-xs font-mono text-text-muted">{phase.label}</span>
              </div>
              {i < phases.length - 1 && (
                <div className="w-6 md:w-10 h-px bg-accent/40" />
              )}
            </div>
          ))}
        </div>

        <div className="mt-12 grid md:grid-cols-3 gap-4 max-w-3xl mx-auto">
          {[
            { title: "Markdown specs", desc: "Define features in plain markdown with acceptance criteria" },
            { title: "Quality gates", desc: "Each phase validates before advancing to the next" },
            { title: "Checkpoint resume", desc: "Pick up exactly where you left off if interrupted" },
          ].map((item) => (
            <div key={item.title} className="p-4 rounded-lg border border-border bg-bg-card text-center">
              <h3 className="text-sm font-semibold text-text mb-1">{item.title}</h3>
              <p className="text-xs text-text-muted">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
