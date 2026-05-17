import { callCommand } from './client'
import type { AiJob, AiJobDraft, AiJobLog } from '@/types/ai'

export function listAiJobs(): Promise<AiJob[]> {
  return callCommand<AiJob[]>('list_ai_jobs').then((items) => items ?? [])
}

export function createAiJob(draft: AiJobDraft): Promise<AiJob> {
  return callCommand<AiJob>('create_ai_job', { draft })
}

export function currentAiJob(): Promise<AiJob | null> {
  return callCommand<AiJob | null>('current_ai_job')
}

export function listAiJobLogs(jobId: string): Promise<AiJobLog[]> {
  return callCommand<AiJobLog[]>('list_ai_job_logs', { jobId }).then((items) => items ?? [])
}

export function cancelAiJob(jobId: string): Promise<AiJob> {
  return callCommand<AiJob>('cancel_ai_job', { jobId })
}

export function retryAiJob(jobId: string): Promise<AiJob> {
  return callCommand<AiJob>('retry_ai_job', { jobId })
}
