import { SectionHeader } from "@/components/ui/SectionHeader";

const profiles = [
  { name: "fast", thinking: "sonnet", execution: "sonnet", isDefault: false },
  { name: "balanced", thinking: "opus", execution: "sonnet", isDefault: true },
  { name: "max", thinking: "opus", execution: "opus", isDefault: false },
];

export function ProfilesSection() {
  return (
    <section id="profiles" className="py-20 md:py-28">
      <div className="max-w-4xl mx-auto px-4">
        <SectionHeader
          label="11 | Profiles"
          title="Control model allocation"
          subtitle="Profiles let you balance cost and quality. Assign stronger models to thinking phases and faster models to execution."
        />

        <div className="rounded-lg border border-border overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="bg-bg-card border-b border-border">
                <th className="text-left text-sm font-medium text-text-muted px-4 py-3">Profile</th>
                <th className="text-left text-sm font-medium text-text-muted px-4 py-3">Thinking (Research, Review)</th>
                <th className="text-left text-sm font-medium text-text-muted px-4 py-3">Execution (Code, Test, Deploy)</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {profiles.map((p) => (
                <tr key={p.name} className="hover:bg-bg-card transition-colors">
                  <td className="px-4 py-3">
                    <span className="font-mono text-sm font-medium text-accent">{p.name}</span>
                    {p.isDefault && (
                      <span className="ml-2 text-xs font-mono px-1.5 py-0.5 rounded bg-accent-dim text-accent">
                        default
                      </span>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <span className="text-xs font-mono px-2 py-1 rounded bg-bg-card border border-border text-text-muted">
                      {p.thinking}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <span className="text-xs font-mono px-2 py-1 rounded bg-bg-card border border-border text-text-muted">
                      {p.execution}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
