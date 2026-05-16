import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useWorkflowStore } from '@/stores/workflowStore'

const invokeMock = vi.mocked(invoke)

describe('workflowStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('runs sample audit and loads repair suggestions', async () => {
    invokeMock
      .mockResolvedValueOnce([{ conflictType: 'mutually_exclusive_axioms', factIds: ['fact_1', 'fact_2'], message: '冲突' }])
      .mockResolvedValueOnce([{ title: '添加例外', description: '增加范围', impact: '中等' }])

    const store = useWorkflowStore()
    await store.runSampleAudit()

    expect(store.conflicts).toHaveLength(1)
    expect(store.repairs[0].title).toContain('例外')
  })
})
