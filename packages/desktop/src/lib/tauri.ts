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

export async function stopSpark(sparkId: string): Promise<void> {
  return invoke<void>("stop_spark", { id: sparkId });
}

// Specs

export type SpecStatus =
  | "draft"
  | "planning"
  | "ready"
  | "executing"
  | "complete"
  | "failed";

export interface SpecFile {
  id: string;
  title: string;
  description: string;
  priority: string | null;
  tags: string[];
  requirements: string[];
  constraints: string[];
  success_criteria: string[];
  body: string;
  status: SpecStatus;
}

export async function listSpecs(projectPath: string): Promise<SpecFile[]> {
  return invoke<SpecFile[]>("list_specs", { projectPath });
}

export async function getSpec(
  projectPath: string,
  specId: string,
): Promise<SpecFile> {
  return invoke<SpecFile>("get_spec", { projectPath, specId });
}

// Marketplace

export type Phase =
  | "research"
  | "code"
  | "test"
  | "review"
  | "deploy"
  | "teams";

export type ContentType = "skill" | "agent" | "command";

export interface Skill {
  id: string;
  name: string;
  description: string;
  phase: Phase;
  tech: string;
  path: string;
  installed: boolean;
  content_types: ContentType[];
  keywords: string[];
}

export interface AgentFrontmatter {
  name: string;
  description?: string;
  tools?: string;
  model?: string;
}

export interface AgentFile {
  filename: string;
  frontmatter: AgentFrontmatter;
  body: string;
}

export interface SkillDetail {
  skill: Skill;
  agents: AgentFile[];
  skill_files: string[];
  command_files: string[];
}

export interface ForjaPaths {
  registry: string;
  state: string;
}

export interface InstallMeta {
  install_date: string;
  last_used: string | null;
}

export async function getForjaPaths(): Promise<ForjaPaths> {
  return invoke<ForjaPaths>("get_forja_paths");
}

export async function listSkills(registryPath: string): Promise<Skill[]> {
  return invoke<Skill[]>("list_skills", { registryPath });
}

export async function searchSkills(
  registryPath: string,
  query: string,
): Promise<Skill[]> {
  return invoke<Skill[]>("search_skills", { registryPath, query });
}

export async function getSkillDetail(
  registryPath: string,
  skillId: string,
): Promise<SkillDetail> {
  return invoke<SkillDetail>("get_skill_detail", { registryPath, skillId });
}

export async function installSkill(
  registryPath: string,
  skillId: string,
): Promise<void> {
  return invoke<void>("install_skill", { registryPath, skillId });
}

export async function uninstallSkill(skillId: string): Promise<void> {
  return invoke<void>("uninstall_skill", { skillId });
}

export async function createSkill(
  name: string,
  phase: string,
  tech: string,
  description: string,
): Promise<string> {
  return invoke<string>("create_skill", { name, phase, tech, description });
}
