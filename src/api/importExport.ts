import { executeCommand } from './client'
import { commands } from './commands'
import type { ImportedProject, ProjectArchive } from '@/types/archive'

export function exportProjectArchive(projectId: string): Promise<ProjectArchive> {
  return executeCommand(commands.archive.exportProject, { projectId })
}

export function importProjectArchive(archive: ProjectArchive): Promise<ImportedProject> {
  return executeCommand(commands.archive.importProject, { archive })
}
