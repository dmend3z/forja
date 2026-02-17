import { useNavigate } from "react-router";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { Skill } from "@/lib/tauri";
import { PHASE_COLORS, PHASE_LABELS, CONTENT_TYPE_ICONS } from "@/lib/constants";

interface SkillRowProps {
  skill: Skill;
  onInstall: (skillId: string) => void;
  onUninstall: (skillId: string) => void;
  loading: boolean;
}

export function SkillRow({
  skill,
  onInstall,
  onUninstall,
  loading,
}: SkillRowProps) {
  const navigate = useNavigate();

  return (
    <tr
      className="border-b border-border cursor-pointer hover:bg-secondary/30 transition-colors"
      onClick={() => navigate(`/marketplace/${encodeURIComponent(skill.id)}`)}
    >
      <td className="py-2 px-3">
        <div className="flex items-center gap-2">
          {skill.installed && (
            <span className="inline-block size-2 rounded-full bg-green-400 shrink-0" />
          )}
          <span className="text-sm font-medium">{skill.name}</span>
        </div>
      </td>
      <td className="py-2 px-3">
        <span className="text-xs text-muted-foreground line-clamp-1">
          {skill.description}
        </span>
      </td>
      <td className="py-2 px-3">
        <Badge className={`${PHASE_COLORS[skill.phase]} text-[10px]`}>
          {PHASE_LABELS[skill.phase]}
        </Badge>
      </td>
      <td className="py-2 px-3">
        <span className="text-xs text-muted-foreground">{skill.tech}</span>
      </td>
      <td className="py-2 px-3">
        <div className="flex gap-0.5">
          {skill.content_types.map((ct) => (
            <span
              key={ct}
              className="inline-flex items-center justify-center size-5 rounded text-[10px] font-mono bg-muted text-muted-foreground"
              title={ct}
            >
              {CONTENT_TYPE_ICONS[ct]}
            </span>
          ))}
        </div>
      </td>
      <td className="py-2 px-3">
        <Button
          size="xs"
          variant={skill.installed ? "outline" : "default"}
          disabled={loading}
          onClick={(e) => {
            e.stopPropagation();
            skill.installed ? onUninstall(skill.id) : onInstall(skill.id);
          }}
        >
          {skill.installed ? "Uninstall" : "Install"}
        </Button>
      </td>
    </tr>
  );
}
