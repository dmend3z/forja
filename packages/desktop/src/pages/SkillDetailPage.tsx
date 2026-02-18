import { useCallback, useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/EmptyState";
import { SkillMetadataCard } from "@/components/marketplace/SkillMetadataCard";
import { AgentPreview } from "@/components/marketplace/AgentPreview";
import { InstallButton } from "@/components/marketplace/InstallButton";
import type { SkillDetail } from "@/lib/tauri";
import {
  getForjaPaths,
  getSkillDetail,
  installSkill,
  uninstallSkill,
} from "@/lib/tauri";

export function SkillDetailPage() {
  const { skillId } = useParams<{ skillId: string }>();
  const navigate = useNavigate();
  const [detail, setDetail] = useState<SkillDetail | null>(null);
  const [registryPath, setRegistryPath] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const decodedId = skillId ? decodeURIComponent(skillId) : "";

  const loadDetail = useCallback(async () => {
    if (!decodedId) return;
    try {
      const paths = await getForjaPaths();
      setRegistryPath(paths.registry);
      const result = await getSkillDetail(paths.registry, decodedId);
      setDetail(result);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [decodedId]);

  useEffect(() => {
    loadDetail();
  }, [loadDetail]);

  async function handleInstall() {
    if (!registryPath || !decodedId) return;
    setLoading(true);
    try {
      await installSkill(registryPath, decodedId);
      await loadDetail();
    } catch (e) {
      console.error("Install failed:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleUninstall() {
    if (!decodedId) return;
    setLoading(true);
    try {
      await uninstallSkill(decodedId);
      await loadDetail();
    } catch (e) {
      console.error("Uninstall failed:", e);
    } finally {
      setLoading(false);
    }
  }

  if (error) {
    return (
      <div className="p-6">
        <EmptyState title="Error" description={error} />
      </div>
    );
  }

  if (!detail) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Loading...</p>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-3xl">
      <Button
        variant="ghost"
        size="sm"
        className="mb-4"
        onClick={() => navigate("/marketplace")}
      >
        &larr; Back to Marketplace
      </Button>

      <div className="flex items-start justify-between gap-4 mb-6">
        <SkillMetadataCard skill={detail.skill} />
        <InstallButton
          installed={detail.skill.installed}
          loading={loading}
          onInstall={handleInstall}
          onUninstall={handleUninstall}
        />
      </div>

      {/* Agent previews */}
      {detail.agents.length > 0 && (
        <div className="space-y-4 mb-6">
          <h3 className="text-sm font-medium text-muted-foreground">
            Agents ({detail.agents.length})
          </h3>
          {detail.agents.map((agent) => (
            <AgentPreview key={agent.filename} agent={agent} />
          ))}
        </div>
      )}

      {/* File listings */}
      {(detail.skill_files.length > 0 || detail.command_files.length > 0) && (
        <div className="space-y-3">
          {detail.skill_files.length > 0 && (
            <div>
              <h3 className="text-sm font-medium text-muted-foreground mb-1">
                Skill Files
              </h3>
              <div className="flex flex-wrap gap-1">
                {detail.skill_files.map((f) => (
                  <span
                    key={f}
                    className="px-2 py-0.5 text-xs rounded bg-muted text-muted-foreground font-mono"
                  >
                    {f}
                  </span>
                ))}
              </div>
            </div>
          )}
          {detail.command_files.length > 0 && (
            <div>
              <h3 className="text-sm font-medium text-muted-foreground mb-1">
                Command Files
              </h3>
              <div className="flex flex-wrap gap-1">
                {detail.command_files.map((f) => (
                  <span
                    key={f}
                    className="px-2 py-0.5 text-xs rounded bg-muted text-muted-foreground font-mono"
                  >
                    {f}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
