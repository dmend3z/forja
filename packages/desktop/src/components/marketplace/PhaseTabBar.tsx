import type { Phase } from "@/lib/tauri";
import { PHASE_LABELS, ALL_PHASES } from "@/lib/constants";

interface PhaseTabBarProps {
  activePhase: Phase | "all";
  onPhaseChange: (phase: Phase | "all") => void;
  phaseCounts: Record<string, number>;
}

export function PhaseTabBar({
  activePhase,
  onPhaseChange,
  phaseCounts,
}: PhaseTabBarProps) {
  const totalCount = Object.values(phaseCounts).reduce((a, b) => a + b, 0);

  return (
    <div className="flex gap-1 mb-4 border-b border-border pb-2 overflow-x-auto">
      <TabButton
        label="All"
        count={totalCount}
        active={activePhase === "all"}
        onClick={() => onPhaseChange("all")}
      />
      {ALL_PHASES.map((phase) => (
        <TabButton
          key={phase}
          label={PHASE_LABELS[phase]}
          count={phaseCounts[phase] ?? 0}
          active={activePhase === phase}
          onClick={() => onPhaseChange(phase)}
        />
      ))}
    </div>
  );
}

function TabButton({
  label,
  count,
  active,
  onClick,
}: {
  label: string;
  count: number;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`px-3 py-1.5 text-sm rounded-md transition-colors whitespace-nowrap ${
        active
          ? "bg-secondary text-secondary-foreground font-medium"
          : "text-muted-foreground hover:text-foreground hover:bg-secondary/50"
      }`}
    >
      {label}
      <span className="ml-1.5 text-xs opacity-60">{count}</span>
    </button>
  );
}
