import type { ContentType } from "@/lib/tauri";
import { CONTENT_TYPE_ICONS } from "@/lib/constants";

interface FilterBarProps {
  contentTypeFilter: ContentType[];
  onContentTypeToggle: (type: ContentType) => void;
  installFilter: "all" | "installed" | "available";
  onInstallFilterChange: (filter: "all" | "installed" | "available") => void;
}

const CONTENT_TYPES: ContentType[] = ["agent", "skill", "command"];
const INSTALL_FILTERS = [
  { value: "all" as const, label: "All" },
  { value: "installed" as const, label: "Installed" },
  { value: "available" as const, label: "Available" },
];

export function FilterBar({
  contentTypeFilter,
  onContentTypeToggle,
  installFilter,
  onInstallFilterChange,
}: FilterBarProps) {
  return (
    <div className="flex items-center gap-4 mb-4">
      <div className="flex items-center gap-1.5">
        <span className="text-xs text-muted-foreground mr-1">Type:</span>
        {CONTENT_TYPES.map((type) => {
          const active = contentTypeFilter.includes(type);
          return (
            <button
              key={type}
              onClick={() => onContentTypeToggle(type)}
              className={`inline-flex items-center gap-1 px-2 py-1 text-xs rounded-md border transition-colors ${
                active
                  ? "bg-primary/10 text-primary border-primary/30"
                  : "text-muted-foreground border-border hover:text-foreground"
              }`}
            >
              <span className="font-mono text-[10px]">
                {CONTENT_TYPE_ICONS[type]}
              </span>
              {type}
            </button>
          );
        })}
      </div>
      <div className="flex items-center gap-1.5">
        <span className="text-xs text-muted-foreground mr-1">Status:</span>
        {INSTALL_FILTERS.map(({ value, label }) => (
          <button
            key={value}
            onClick={() => onInstallFilterChange(value)}
            className={`px-2 py-1 text-xs rounded-md transition-colors ${
              installFilter === value
                ? "bg-secondary text-secondary-foreground"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
}
