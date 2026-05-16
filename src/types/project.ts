export interface Project {
  id: string
  name: string
  description: string
  createdAt: string
  updatedAt: string
  archivedAt: string | null
}

export interface ProjectDraft {
  name: string
  description: string
}
