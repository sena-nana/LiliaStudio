import { defineStore } from 'pinia'
import {
  cancelAiJob,
  createAiJob,
  currentAiJob,
  listAiJobLogs,
  listAiJobs,
  retryAiJob,
} from '@/api/jobs'
import type { AiJob, AiJobDraft, AiJobLog } from '@/types/ai'

interface JobState {
  jobs: AiJob[]
  logsByJobId: Record<string, AiJobLog[]>
  activeJob: AiJob | null
  loading: boolean
}

export const useJobStore = defineStore('jobs', {
  state: (): JobState => ({
    jobs: [],
    logsByJobId: {},
    activeJob: null,
    loading: false,
  }),
  actions: {
    async loadJobs() {
      this.jobs = await listAiJobs()
    },
    async refreshActiveJob() {
      this.activeJob = await currentAiJob()
      return this.activeJob
    },
    async loadLogs(jobId: string) {
      const logs = await listAiJobLogs(jobId)
      this.logsByJobId = { ...this.logsByJobId, [jobId]: logs }
      return logs
    },
    async createJob(draft: AiJobDraft) {
      const job = await createAiJob(draft)
      this.jobs = [job, ...this.jobs]
      this.activeJob = job.status === 'running' ? job : this.activeJob
      return job
    },
    async cancelJob(jobId: string) {
      const job = await cancelAiJob(jobId)
      this.replaceJob(job)
      if (this.activeJob?.id === job.id) {
        this.activeJob = null
      }
      return job
    },
    async retryJob(jobId: string) {
      const job = await retryAiJob(jobId)
      this.jobs = [job, ...this.jobs]
      return job
    },
    replaceJob(job: AiJob) {
      const index = this.jobs.findIndex((item) => item.id === job.id)
      if (index >= 0) {
        this.jobs.splice(index, 1, job)
      } else {
        this.jobs = [job, ...this.jobs]
      }
    },
  },
})
