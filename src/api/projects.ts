import { executeCommand, listResult } from './client'
import { commands } from './commands'
import type { Project, ProjectDraft } from '@/types/project'

export function listProjects(): Promise<Project[]> {
  return executeCommand(commands.project.list).then(listResult)
}

export function createProject(draft: ProjectDraft): Promise<Project> {
  return executeCommand(commands.project.create, { draft })
}

export function updateProject(id: string, draft: ProjectDraft): Promise<Project> {
  return executeCommand(commands.project.update, { id, draft })
}

export function archiveProject(id: string): Promise<void> {
  return executeCommand(commands.project.archive, { id })
}
