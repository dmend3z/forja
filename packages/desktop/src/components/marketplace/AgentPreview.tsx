import { Badge } from "@/components/ui/badge";
import type { AgentFile } from "@/lib/tauri";

interface AgentPreviewProps {
  agent: AgentFile;
}

export function AgentPreview({ agent }: AgentPreviewProps) {
  const { frontmatter, body, filename } = agent;
  const tools = frontmatter.tools
    ?.split(",")
    .map((t) => t.trim())
    .filter(Boolean);

  return (
    <div className="rounded-lg border overflow-hidden">
      {/* Frontmatter panel */}
      <div className="bg-muted/50 px-4 py-3 border-b space-y-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">{frontmatter.name}</span>
            <span className="text-xs text-muted-foreground font-mono">
              {filename}
            </span>
          </div>
          {frontmatter.model && (
            <Badge variant="outline" className="text-[10px]">
              {frontmatter.model}
            </Badge>
          )}
        </div>
        {frontmatter.description && (
          <p className="text-xs text-muted-foreground">
            {frontmatter.description}
          </p>
        )}
        {tools && tools.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {tools.map((tool) => (
              <span
                key={tool}
                className="px-1.5 py-0.5 text-[10px] rounded bg-secondary text-secondary-foreground"
              >
                {tool}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Markdown body */}
      <pre className="px-4 py-3 text-xs font-mono whitespace-pre-wrap overflow-auto max-h-96 text-muted-foreground">
        {body || "(empty body)"}
      </pre>
    </div>
  );
}
