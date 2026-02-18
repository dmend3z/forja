import { SectionHeader } from "@/components/ui/SectionHeader";
import { FeatureCard } from "@/components/ui/FeatureCard";
import { Boxes, Users, Terminal, Link } from "lucide-react";

const features = [
  {
    icon: Boxes,
    title: "25 Curated Agents",
    description:
      "Organized into 5 dev phases. Research, code, test, review, and deploy with purpose-built agents for each step.",
  },
  {
    icon: Users,
    title: "Agent Team Configs",
    description:
      "Multi-agent development pipelines. From full 5-agent product teams to quick 2-agent hotfix crews.",
  },
  {
    icon: Terminal,
    title: "One CLI",
    description:
      "Install, uninstall, search, and manage everything with a single command. No manual file copying or configuration.",
  },
  {
    icon: Link,
    title: "Symlink-Based",
    description:
      "Skills live in a central registry, installed via symlinks into ~/.claude/agents/. Clean, reversible, zero duplication.",
  },
];

export function WhySection() {
  return (
    <section id="why" className="py-20 md:py-28 bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="02 | Why forja?"
          title="Everything you need, nothing you don't"
          subtitle="Claude Code supports agents and skills — but managing them across projects is manual. forja gives you a single CLI to install, search, and manage all of it."
        />
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {features.map((f) => (
            <FeatureCard key={f.title} {...f} />
          ))}
        </div>
      </div>
    </section>
  );
}
