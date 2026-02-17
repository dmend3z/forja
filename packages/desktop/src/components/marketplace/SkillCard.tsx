import { useNavigate } from "react-router";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "@/components/ui/card";
import type { Skill } from "@/lib/tauri";
import { PHASE_COLORS, PHASE_LABELS, CONTENT_TYPE_ICONS } from "@/lib/constants";

interface SkillCardProps {
  skill: Skill;
  onInstall: (skillId: string) => void;
  onUninstall: (skillId: string) => void;
  loading: boolean;
}

export function SkillCard({
  skill,
  onInstall,
  onUninstall,
  loading,
}: SkillCardProps) {
  const navigate = useNavigate();

  return (
    <Card
      className="cursor-pointer hover:border-primary/30 transition-colors"
      onClick={() => navigate(`/marketplace/${encodeURIComponent(skill.id)}`)}
    >
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <CardTitle className="text-sm">{skill.name}</CardTitle>
          {skill.installed && (
            <span className="inline-block size-2 rounded-full bg-green-400 shrink-0 mt-1" />
          )}
        </div>
        <CardDescription className="text-xs line-clamp-2">
          {skill.description}
        </CardDescription>
      </CardHeader>
      <CardContent className="pb-2">
        <div className="flex flex-wrap gap-1">
          <Badge className={PHASE_COLORS[skill.phase]}>
            {PHASE_LABELS[skill.phase]}
          </Badge>
          <Badge variant="outline">{skill.tech}</Badge>
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
      </CardContent>
      <CardFooter className="pt-0">
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
      </CardFooter>
    </Card>
  );
}
