import { callCommand } from './client'
import type { ImportedProject, ProjectArchive } from '@/types/archive'

export function exportProjectArchive(projectId: string): Promise<ProjectArchive> {
  return callCommand<ProjectArchive>('export_project_archive', { projectId })
}

export function importProjectArchive(archive: ProjectArchive): Promise<ImportedProject> {
  return callCommand<ImportedProject>('import_project_archive', { archive })
}
