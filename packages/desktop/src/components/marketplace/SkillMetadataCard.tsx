import { Badge } from "@/components/ui/badge";
import type { Skill } from "@/lib/tauri";
import { PHASE_COLORS, PHASE_LABELS, CONTENT_TYPE_ICONS } from "@/lib/constants";

interface SkillMetadataCardProps {
  skill: Skill;
}

export function SkillMetadataCard({ skill }: SkillMetadataCardProps) {
  return (
    <div className="space-y-3">
      <div>
        <h2 className="text-xl font-bold">{skill.name}</h2>
        <p className="text-sm text-muted-foreground mt-1">
          {skill.description}
        </p>
        <p className="text-xs text-muted-foreground mt-1 font-mono">
          {skill.id}
        </p>
      </div>

      <div className="flex flex-wrap gap-1.5">
        <Badge className={PHASE_COLORS[skill.phase]}>
          {PHASE_LABELS[skill.phase]}
        </Badge>
        <Badge variant="outline">{skill.tech}</Badge>
        {skill.content_types.map((ct) => (
          <Badge key={ct} variant="secondary">
            {CONTENT_TYPE_ICONS[ct]} {ct}
          </Badge>
        ))}
      </div>

      {skill.keywords.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {skill.keywords.map((kw) => (
            <span
              key={kw}
              className="px-1.5 py-0.5 text-[10px] rounded bg-muted text-muted-foreground"
            >
              {kw}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
