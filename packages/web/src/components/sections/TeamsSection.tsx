import { SectionHeader } from "@/components/ui/SectionHeader";
import { ArrowRight } from "lucide-react";

const teams = [
  {
    name: "Full Product",
    agents: ["researcher", "coder", "tester", "reviewer", "deployer"],
  },
  {
    name: "Solo Sprint",
    agents: ["coder-tester", "quick-reviewer"],
  },
  {
    name: "Quick Fix",
    agents: ["coder", "deployer"],
  },
  {
    name: "Refactor",
    agents: ["analyzer", "refactorer", "reviewer"],
  },
];

export function TeamsSection() {
  return (
    <section id="teams" className="py-20 md:py-28 bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="10 | Agent Teams"
          title="Multi-agent configurations"
          subtitle="Pre-built team configurations for common workflows. Each team assigns specialized agents to coordinate through the pipeline."
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {teams.map((team) => (
            <div key={team.name} className="p-5 rounded-lg border border-border bg-bg-card hover:bg-bg-elevated transition-colors">
              <h3 className="text-lg font-semibold text-text mb-4">{team.name}</h3>
              <div className="flex flex-wrap items-center gap-2">
                {team.agents.map((agent, i) => (
                  <div key={agent} className="flex items-center gap-2">
                    <span className="text-xs font-mono px-2.5 py-1 rounded-md bg-accent-dim text-accent">
                      {agent}
                    </span>
                    {i < team.agents.length - 1 && (
                      <ArrowRight size={14} className="text-text-subtle" />
                    )}
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
