import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { healthCheck } from '@/api/health'

const invokeMock = vi.mocked(invoke)

describe('healthCheck', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('returns normalized health information from the backend', async () => {
    invokeMock.mockResolvedValue({
      appVersion: '0.1.0',
      platform: 'windows',
      appDataDir: 'C:/Users/demo/AppData/Roaming/ameya',
    })

    await expect(healthCheck()).resolves.toEqual({
      appVersion: '0.1.0',
      platform: 'windows',
      appDataDir: 'C:/Users/demo/AppData/Roaming/ameya',
    })
    expect(invokeMock).toHaveBeenCalledWith('health_check')
  })

  it('wraps backend errors with a stable client error', async () => {
    invokeMock.mockRejectedValue(new Error('backend unavailable'))

    await expect(healthCheck()).rejects.toMatchObject({
      code: 'TAURI_COMMAND_FAILED',
      message: 'backend unavailable',
    })
  })
})
