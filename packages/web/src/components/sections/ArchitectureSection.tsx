import { SectionHeader } from "@/components/ui/SectionHeader";

const steps = [
  {
    number: "1",
    title: "Clone Registry",
    desc: "forja init clones the skills repo into ~/.forja/registry/",
  },
  {
    number: "2",
    title: "Auto-Install Agents",
    desc: "forja init auto-installs all agents as symlinks into ~/.claude/agents/",
  },
  {
    number: "3",
    title: "Auto-Detected",
    desc: "Claude Code discovers the new agents automatically. No restart needed.",
  },
  {
    number: "4",
    title: "Clean Uninstall",
    desc: "forja uninstall removes symlinks and updates state. No leftover files.",
  },
];

export function ArchitectureSection() {
  return (
    <section id="architecture" className="py-20 md:py-28 bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="12 | Architecture"
          title="Simple symlink architecture"
          subtitle="No magic. forja clones a skills registry and creates symlinks. Claude Code picks up the agents automatically."
        />

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {steps.map((s) => (
            <div key={s.number} className="p-5 rounded-lg border border-border bg-bg-card hover:bg-bg-elevated transition-colors">
              <span className="inline-block w-8 h-8 rounded-full bg-accent-dim text-accent font-mono font-semibold text-sm flex items-center justify-center mb-3">
                {s.number}
              </span>
              <h3 className="text-base font-semibold text-text mb-2">{s.title}</h3>
              <p className="text-sm text-text-muted leading-relaxed">{s.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
