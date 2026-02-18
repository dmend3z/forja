import { NavLink, useParams } from "react-router";

const TABS = [
  { label: "Sparks", path: "sparks" },
  { label: "Specs", path: "specs" },
];

export function ProjectTabBar() {
  const { id } = useParams<{ id: string }>();

  return (
    <div className="flex gap-1 mb-6 border-b border-border">
      {TABS.map((tab) => (
        <NavLink
          key={tab.path}
          to={`/project/${id}/${tab.path}`}
          className={({ isActive }) =>
            `px-3 py-2 text-sm font-medium border-b-2 transition-colors ${
              isActive
                ? "border-primary text-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground"
            }`
          }
        >
          {tab.label}
        </NavLink>
      ))}
    </div>
  );
}
