import type { Phase, ContentType } from "./tauri";

export const PHASE_COLORS: Record<Phase, string> = {
  research: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  code: "bg-green-500/20 text-green-400 border-green-500/30",
  test: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
  review: "bg-purple-500/20 text-purple-400 border-purple-500/30",
  deploy: "bg-orange-500/20 text-orange-400 border-orange-500/30",
  teams: "bg-cyan-500/20 text-cyan-400 border-cyan-500/30",
};

export const PHASE_LABELS: Record<Phase, string> = {
  research: "Research",
  code: "Code",
  test: "Test",
  review: "Review",
  deploy: "Deploy",
  teams: "Teams",
};

export const CONTENT_TYPE_ICONS: Record<ContentType, string> = {
  agent: "A",
  skill: "S",
  command: "C",
};

export const ALL_PHASES: Phase[] = [
  "research",
  "code",
  "test",
  "review",
  "deploy",
  "teams",
];
