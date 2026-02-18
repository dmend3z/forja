import type { LucideIcon } from "lucide-react";

interface FeatureCardProps {
  icon: LucideIcon;
  title: string;
  description: string;
}

export function FeatureCard({ icon: Icon, title, description }: FeatureCardProps) {
  return (
    <div className="p-6 rounded-lg border border-border bg-bg-card hover:bg-bg-elevated transition-colors">
      <div className="w-10 h-10 rounded-lg bg-accent-dim flex items-center justify-center mb-4">
        <Icon size={20} className="text-accent" />
      </div>
      <h3 className="text-lg font-semibold text-text mb-2">{title}</h3>
      <p className="text-text-muted text-sm leading-relaxed">{description}</p>
    </div>
  );
}
