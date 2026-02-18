import type { Skill } from "@/lib/tauri";
import { SkillRow } from "./SkillRow";

interface SkillTableProps {
  skills: Skill[];
  onInstall: (skillId: string) => void;
  onUninstall: (skillId: string) => void;
  loadingSkillId: string | null;
}

export function SkillTable({
  skills,
  onInstall,
  onUninstall,
  loadingSkillId,
}: SkillTableProps) {
  return (
    <div className="rounded-lg border overflow-hidden">
      <table className="w-full">
        <thead>
          <tr className="border-b border-border bg-muted/50">
            <th className="text-left py-2 px-3 text-xs font-medium text-muted-foreground">
              Name
            </th>
            <th className="text-left py-2 px-3 text-xs font-medium text-muted-foreground">
              Description
            </th>
            <th className="text-left py-2 px-3 text-xs font-medium text-muted-foreground">
              Phase
            </th>
            <th className="text-left py-2 px-3 text-xs font-medium text-muted-foreground">
              Tech
            </th>
            <th className="text-left py-2 px-3 text-xs font-medium text-muted-foreground">
              Types
            </th>
            <th className="text-left py-2 px-3 text-xs font-medium text-muted-foreground">
              Action
            </th>
          </tr>
        </thead>
        <tbody>
          {skills.map((skill) => (
            <SkillRow
              key={skill.id}
              skill={skill}
              onInstall={onInstall}
              onUninstall={onUninstall}
              loading={loadingSkillId === skill.id}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}
