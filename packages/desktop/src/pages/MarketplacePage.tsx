import { useCallback, useEffect, useMemo, useState } from "react";
import { EmptyState } from "@/components/EmptyState";
import { MarketplaceHeader } from "@/components/marketplace/MarketplaceHeader";
import { PhaseTabBar } from "@/components/marketplace/PhaseTabBar";
import { FilterBar } from "@/components/marketplace/FilterBar";
import { SkillGrid } from "@/components/marketplace/SkillGrid";
import { SkillTable } from "@/components/marketplace/SkillTable";
import { CreateSkillWizard } from "@/components/marketplace/CreateSkillWizard";
import type { Skill, Phase, ContentType } from "@/lib/tauri";
import {
  getForjaPaths,
  listSkills,
  installSkill,
  uninstallSkill,
} from "@/lib/tauri";

export function MarketplacePage() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [registryPath, setRegistryPath] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [activePhase, setActivePhase] = useState<Phase | "all">("all");
  const [viewMode, setViewMode] = useState<"grid" | "table">("grid");
  const [contentTypeFilter, setContentTypeFilter] = useState<ContentType[]>([]);
  const [installFilter, setInstallFilter] = useState<
    "all" | "installed" | "available"
  >("all");
  const [loadingSkillId, setLoadingSkillId] = useState<string | null>(null);
  const [showCreateWizard, setShowCreateWizard] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSkills = useCallback(async () => {
    try {
      const paths = await getForjaPaths();
      setRegistryPath(paths.registry);
      const result = await listSkills(paths.registry);
      setSkills(result);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadSkills();
  }, [loadSkills]);

  const filteredSkills = useMemo(() => {
    let result = skills;

    if (activePhase !== "all") {
      result = result.filter((s) => s.phase === activePhase);
    }

    if (contentTypeFilter.length > 0) {
      result = result.filter((s) =>
        contentTypeFilter.some((ct) => s.content_types.includes(ct)),
      );
    }

    if (installFilter === "installed") {
      result = result.filter((s) => s.installed);
    } else if (installFilter === "available") {
      result = result.filter((s) => !s.installed);
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          s.description.toLowerCase().includes(q) ||
          s.id.toLowerCase().includes(q) ||
          s.tech.toLowerCase().includes(q) ||
          s.keywords.some((k) => k.toLowerCase().includes(q)),
      );
    }

    return result;
  }, [skills, activePhase, contentTypeFilter, installFilter, searchQuery]);

  const phaseCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const skill of skills) {
      counts[skill.phase] = (counts[skill.phase] ?? 0) + 1;
    }
    return counts;
  }, [skills]);

  async function handleInstall(skillId: string) {
    if (!registryPath) return;
    setLoadingSkillId(skillId);
    try {
      await installSkill(registryPath, skillId);
      await loadSkills();
    } catch (e) {
      console.error("Install failed:", e);
    } finally {
      setLoadingSkillId(null);
    }
  }

  async function handleUninstall(skillId: string) {
    setLoadingSkillId(skillId);
    try {
      await uninstallSkill(skillId);
      await loadSkills();
    } catch (e) {
      console.error("Uninstall failed:", e);
    } finally {
      setLoadingSkillId(null);
    }
  }

  function handleContentTypeToggle(type: ContentType) {
    setContentTypeFilter((prev) =>
      prev.includes(type) ? prev.filter((t) => t !== type) : [...prev, type],
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <EmptyState
          title="Could not load marketplace"
          description={error}
        />
      </div>
    );
  }

  return (
    <div className="p-6">
      <MarketplaceHeader
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
        onCreateClick={() => setShowCreateWizard(true)}
        totalCount={skills.length}
        filteredCount={filteredSkills.length}
      />

      <PhaseTabBar
        activePhase={activePhase}
        onPhaseChange={setActivePhase}
        phaseCounts={phaseCounts}
      />

      <FilterBar
        contentTypeFilter={contentTypeFilter}
        onContentTypeToggle={handleContentTypeToggle}
        installFilter={installFilter}
        onInstallFilterChange={setInstallFilter}
      />

      {filteredSkills.length === 0 ? (
        <EmptyState
          title="No skills found"
          description={
            searchQuery
              ? "Try a different search query or clear filters."
              : "No skills match the current filters."
          }
        />
      ) : viewMode === "grid" ? (
        <SkillGrid
          skills={filteredSkills}
          onInstall={handleInstall}
          onUninstall={handleUninstall}
          loadingSkillId={loadingSkillId}
        />
      ) : (
        <SkillTable
          skills={filteredSkills}
          onInstall={handleInstall}
          onUninstall={handleUninstall}
          loadingSkillId={loadingSkillId}
        />
      )}

      {showCreateWizard && (
        <CreateSkillWizard
          onClose={() => setShowCreateWizard(false)}
          onCreated={() => {
            setShowCreateWizard(false);
            loadSkills();
          }}
        />
      )}
    </div>
  );
}
