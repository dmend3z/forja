import { invoke } from "@tauri-apps/api/core";

export interface Project {
  id: string;
  name: string;
  path: string;
  last_opened: string;
  forja_initialized: boolean;
}

export interface ProjectList {
  projects: Project[];
  active_project_id: string | null;
}

export async function listProjects(): Promise<ProjectList> {
  return invoke<ProjectList>("list_projects");
}

export async function addProject(path: string): Promise<Project> {
  return invoke<Project>("add_project", { path });
}

// Sparks

export type SparkType = "task" | "quick_fix" | "plan";
export type SparkStatus = "starting" | "running" | "idle" | "stopped" | "failed";

export interface SparkInfo {
  id: string;
  project_id: string;
  spark_type: SparkType;
  description: string;
  status: SparkStatus;
  created_at: string;
  finished_at: string | null;
  output: string | null;
  error: string | null;
}

export async function startSpark(
  projectId: string,
  sparkType: SparkType,
  description: string,
  projectPath: string,
): Promise<SparkInfo> {
  return invoke<SparkInfo>("start_spark", {
    projectId,
    sparkType,
    description,
    projectPath,
  });
}

export async function listSparks(projectId: string): Promise<SparkInfo[]> {
  return invoke<SparkInfo[]>("list_sparks", { projectId });
}
