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
    selectProject(id: string) {
      this.activeProjectId = id
    },
  },
})
