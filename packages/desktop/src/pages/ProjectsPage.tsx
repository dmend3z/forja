import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { EmptyState } from "@/components/EmptyState";
import { listProjects, addProject, type Project } from "@/lib/tauri";

export function ProjectsPage() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    listProjects()
      .then((result) => setProjects(result.projects))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  async function handleAddProject() {
    const selected = await open({ directory: true, title: "Select project folder" });
    if (!selected) return;

    await addProject(selected as string);
    const result = await listProjects();
    setProjects(result.projects);
  }

  if (loading) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Loading projects...</p>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold">Projects</h2>
        <Button size="sm" variant="outline" onClick={handleAddProject}>
          Add Project
        </Button>
      </div>

      {projects.length === 0 ? (
        <EmptyState
          title="No projects yet"
          description="Add a git repository to start using forja with your project."
          action={
            <Button size="sm" onClick={handleAddProject}>Add your first project</Button>
          }
        />
      ) : (
        <div className="grid gap-3">
          {projects.map((project) => (
            <Card
              key={project.id}
              className="cursor-pointer hover:bg-card/80 transition-colors"
              onClick={() => navigate(`/project/${project.id}/sparks`)}
            >
              <CardHeader className="py-4">
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-base">{project.name}</CardTitle>
                    <CardDescription className="text-xs">{project.path}</CardDescription>
                  </div>
                  {project.forja_initialized && (
                    <span className="text-xs text-primary font-medium px-2 py-1 bg-primary/10 rounded">
                      forja
                    </span>
                  )}
                </div>
              </CardHeader>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
