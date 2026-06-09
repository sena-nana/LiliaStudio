import { executeCommand, listResult } from './client'
import { commands } from './commands'
import type { AiJob, AiJobDraft, AiJobLog } from '@/types/ai'

export function listAiJobs(): Promise<AiJob[]> {
  return executeCommand(commands.job.list).then(listResult)
}

export function createAiJob(draft: AiJobDraft): Promise<AiJob> {
  return executeCommand(commands.job.create, { draft })
}

export function currentAiJob(): Promise<AiJob | null> {
  return executeCommand(commands.job.current)
}

export function listAiJobLogs(jobId: string): Promise<AiJobLog[]> {
  return executeCommand(commands.job.logs, { jobId }).then(listResult)
}

export function cancelAiJob(jobId: string): Promise<AiJob> {
  return executeCommand(commands.job.cancel, { jobId })
}

export function retryAiJob(jobId: string): Promise<AiJob> {
  return executeCommand(commands.job.retry, { jobId })
}
