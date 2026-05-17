import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useJobStore } from '@/stores/jobStore'

const invokeMock = vi.mocked(invoke)

describe('jobStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('loads jobs and current running job', async () => {
    invokeMock
      .mockResolvedValueOnce([{ id: 'job_1', status: 'running', jobType: 'logicAudit' }])
      .mockResolvedValueOnce({ id: 'job_1', status: 'running', jobType: 'logicAudit' })

    const store = useJobStore()
    await store.loadJobs()
    await store.refreshActiveJob()

    expect(store.jobs[0].id).toBe('job_1')
    expect(store.activeJob?.status).toBe('running')
    expect(invokeMock).toHaveBeenCalledWith('list_ai_jobs')
    expect(invokeMock).toHaveBeenCalledWith('current_ai_job')
  })

  it('cancels and retries jobs through commands', async () => {
    invokeMock
      .mockResolvedValueOnce({ id: 'job_1', status: 'cancelled' })
      .mockResolvedValueOnce({ id: 'job_2', status: 'queued', retryOfJobId: 'job_1' })

    const store = useJobStore()
    const cancelled = await store.cancelJob('job_1')
    const retry = await store.retryJob('job_1')

    expect(cancelled.status).toBe('cancelled')
    expect(retry.retryOfJobId).toBe('job_1')
    expect(invokeMock).toHaveBeenCalledWith('cancel_ai_job', { jobId: 'job_1' })
    expect(invokeMock).toHaveBeenCalledWith('retry_ai_job', { jobId: 'job_1' })
  })

  it('loads sanitized job logs', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 'log_1',
        jobId: 'job_1',
        level: 'error',
        message: 'Authorization: Bearer [redacted]',
      },
    ])

    const store = useJobStore()
    await store.loadLogs('job_1')

    expect(store.logsByJobId.job_1[0].message).not.toContain('sk-')
    expect(invokeMock).toHaveBeenCalledWith('list_ai_job_logs', { jobId: 'job_1' })
  })
})
