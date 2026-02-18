import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

interface MarketplaceHeaderProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  viewMode: "grid" | "table";
  onViewModeChange: (mode: "grid" | "table") => void;
  onCreateClick: () => void;
  totalCount: number;
  filteredCount: number;
}

export function MarketplaceHeader({
  searchQuery,
  onSearchChange,
  viewMode,
  onViewModeChange,
  onCreateClick,
  totalCount,
  filteredCount,
}: MarketplaceHeaderProps) {
  return (
    <div className="flex items-center justify-between gap-4 mb-4">
      <div>
        <h2 className="text-xl font-bold">Marketplace</h2>
        <p className="text-sm text-muted-foreground">
          {filteredCount} of {totalCount} skills
        </p>
      </div>
      <div className="flex items-center gap-2">
        <Input
          placeholder="Search skills..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="w-64"
        />
        <div className="flex rounded-md border">
          <button
            onClick={() => onViewModeChange("grid")}
            className={`px-2.5 py-1.5 text-xs rounded-l-md transition-colors ${
              viewMode === "grid"
                ? "bg-secondary text-secondary-foreground"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <rect width="7" height="7" x="3" y="3" rx="1" />
              <rect width="7" height="7" x="14" y="3" rx="1" />
              <rect width="7" height="7" x="3" y="14" rx="1" />
              <rect width="7" height="7" x="14" y="14" rx="1" />
            </svg>
          </button>
          <button
            onClick={() => onViewModeChange("table")}
            className={`px-2.5 py-1.5 text-xs rounded-r-md transition-colors ${
              viewMode === "table"
                ? "bg-secondary text-secondary-foreground"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <line x1="3" y1="6" x2="21" y2="6" />
              <line x1="3" y1="12" x2="21" y2="12" />
              <line x1="3" y1="18" x2="21" y2="18" />
            </svg>
          </button>
        </div>
        <Button size="sm" onClick={onCreateClick}>
          + Create Skill
        </Button>
      </div>
    </div>
  );
}
