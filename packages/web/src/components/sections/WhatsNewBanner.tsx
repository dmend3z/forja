import { Sparkles, ScanSearch, Monitor, AppWindow } from "lucide-react";

const features = [
  { icon: Sparkles, label: "Sparks", href: "#sparks", color: "text-sparks" },
  { icon: ScanSearch, label: "Scan", href: "#scan", color: "text-scan" },
  { icon: AppWindow, label: "Desktop", href: "#desktop", color: "text-desktop" },
  { icon: Monitor, label: "Monitor", href: "#monitor", color: "text-monitor" },
];

export function WhatsNewBanner() {
  return (
    <div className="border-y border-dashed border-border bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4 py-4 flex flex-wrap items-center justify-center gap-6 md:gap-10">
        <span className="text-xs font-mono text-text-subtle uppercase tracking-wider">
          What&apos;s New
        </span>
        {features.map((f) => (
          <a
            key={f.label}
            href={f.href}
            className="inline-flex items-center gap-2 text-sm text-text-muted hover:text-text transition-colors group"
          >
            <f.icon size={16} className={f.color} />
            <span className="group-hover:underline">{f.label}</span>
          </a>
        ))}
      </div>
    </div>
  );
}
