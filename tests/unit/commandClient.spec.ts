import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { commands } from '@/api/commands'
import { executeCommand, listResult } from '@/api/client'

const invokeMock = vi.mocked(invoke)

describe('typed command client', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('invokes commands through their typed definition', async () => {
    invokeMock.mockResolvedValueOnce([{ id: 'project_1', name: '雨夜都市' }])

    await expect(executeCommand(commands.project.list)).resolves.toEqual([
      { id: 'project_1', name: '雨夜都市' },
    ])
    expect(invokeMock).toHaveBeenCalledWith('project_list')
  })

  it('passes typed command args', async () => {
    invokeMock.mockResolvedValueOnce({ id: 'project_1', name: '雨夜都市' })

    await executeCommand(commands.project.create, {
      draft: { name: '雨夜都市', description: '' },
    })

    expect(invokeMock).toHaveBeenCalledWith('project_create', {
      draft: { name: '雨夜都市', description: '' },
    })
  })

  it('normalizes nullable list results', () => {
    expect(listResult(null)).toEqual([])
    expect(listResult(undefined)).toEqual([])
    expect(listResult([{ id: 'item_1' }])).toEqual([{ id: 'item_1' }])
  })

  it('wraps command failures with API errors', async () => {
    invokeMock.mockRejectedValueOnce('boom')

    await expect(executeCommand(commands.project.list)).rejects.toMatchObject({
      code: 'TAURI_COMMAND_FAILED',
      message: 'boom',
    })
  })
})
