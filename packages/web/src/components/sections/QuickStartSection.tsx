import { SectionHeader } from "@/components/ui/SectionHeader";
import { TerminalBlock } from "@/components/ui/TerminalBlock";

const steps = [
  {
    number: "1",
    title: "Install the CLI",
    command: "npm install -g forja-cli",
    alt: 'Or: brew install dmend3z/forja/forja',
  },
  {
    number: "2",
    title: "Initialize — all 25 agents installed",
    command: "forja init",
  },
  {
    number: "3",
    title: "Start building",
    command: 'forja plan "add user auth with JWT"',
    alt: 'With a team: forja task "..." --team full-product',
  },
];

export function QuickStartSection() {
  return (
    <section id="quickstart" className="py-20 md:py-28">
      <div className="max-w-4xl mx-auto px-4">
        <SectionHeader
          label="01 | Quick Start"
          title="Up and running in 60 seconds"
        />
        <div className="space-y-6">
          {steps.map((step) => (
            <div key={step.number} className="flex gap-4 md:gap-6">
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-accent-dim border border-accent/30 flex items-center justify-center text-sm font-mono font-medium text-accent">
                {step.number}
              </div>
              <div className="flex-1 min-w-0">
                <h3 className="text-lg font-semibold text-text mb-3">{step.title}</h3>
                <TerminalBlock copyText={step.command}>
                  <span className="text-text-subtle">$ </span>
                  <span className="text-text">{step.command}</span>
                </TerminalBlock>
                {step.alt && (
                  <p className="text-sm text-text-subtle mt-2">
                    {step.alt}
                  </p>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
