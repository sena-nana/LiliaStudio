import { defineStore } from 'pinia'
import * as projectsApi from '@/api/projects'
import type { Project, ProjectDraft } from '@/types/project'

interface ProjectState {
  projects: Project[]
  activeProjectId: string | null
  loading: boolean
  error: string | null
}

export const useProjectStore = defineStore('projects', {
  state: (): ProjectState => ({
    projects: [],
    activeProjectId: null,
    loading: false,
    error: null,
  }),
  getters: {
    activeProject(state): Project | null {
      return state.projects.find((project) => project.id === state.activeProjectId) ?? null
    },
  },
  actions: {
    async loadProjects() {
      this.loading = true
      this.error = null
      try {
        this.projects = await projectsApi.listProjects()
        if (!this.activeProjectId && this.projects.length > 0) {
          this.activeProjectId = this.projects[0].id
        }
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error)
      } finally {
        this.loading = false
      }
    },
    async createProject(draft: ProjectDraft) {
      const project = await projectsApi.createProject(draft)
      this.projects = [project, ...this.projects]
      this.activeProjectId = project.id
      return project
    },
    async updateProject(id: string, draft: ProjectDraft) {
      const project = await projectsApi.updateProject(id, draft)
      this.replaceProject(project)
      return project
    },
    async archiveProject(id: string) {
      await projectsApi.archiveProject(id)
      this.projects = this.projects.filter((project) => project.id !== id)
      if (this.activeProjectId === id) {
        this.activeProjectId = this.projects[0]?.id ?? null
      }
    },
    selectProject(id: string) {
      this.activeProjectId = id
    },
    replaceProject(project: Project) {
      const index = this.projects.findIndex((item) => item.id === project.id)
      if (index >= 0) {
        this.projects.splice(index, 1, project)
      } else {
        this.projects = [project, ...this.projects]
      }
    },
  },
})
