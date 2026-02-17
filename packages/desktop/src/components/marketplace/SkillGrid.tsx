import type { Skill } from "@/lib/tauri";
import { SkillCard } from "./SkillCard";

interface SkillGridProps {
  skills: Skill[];
  onInstall: (skillId: string) => void;
  onUninstall: (skillId: string) => void;
  loadingSkillId: string | null;
}

export function SkillGrid({
  skills,
  onInstall,
  onUninstall,
  loadingSkillId,
}: SkillGridProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
      {skills.map((skill) => (
        <SkillCard
          key={skill.id}
          skill={skill}
          onInstall={onInstall}
          onUninstall={onUninstall}
          loading={loadingSkillId === skill.id}
        />
      ))}
    </div>
  );
}
