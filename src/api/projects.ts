import { callCommand } from './client'
import type { Project, ProjectDraft } from '@/types/project'

export function listProjects(): Promise<Project[]> {
  return callCommand<Project[]>('list_projects').then((projects) => projects ?? [])
}

export function createProject(draft: ProjectDraft): Promise<Project> {
  return callCommand<Project>('create_project', { draft })
}

export function updateProject(id: string, draft: ProjectDraft): Promise<Project> {
  return callCommand<Project>('update_project', { id, draft })
}

export function archiveProject(id: string): Promise<void> {
  return callCommand<void>('archive_project', { id })
}
