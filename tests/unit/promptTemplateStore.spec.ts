import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { usePromptTemplateStore } from '@/stores/promptTemplateStore'

const invokeMock = vi.mocked(invoke)

describe('promptTemplateStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('loads templates and selects the first template', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 'prompt_1',
        name: '逻辑审计',
        purpose: 'logic_audit',
        template: '{{project_context}}',
        builtIn: true,
        variables: [],
      },
    ])

    const store = usePromptTemplateStore()
    await store.loadTemplates()

    expect(store.templates[0].id).toBe('prompt_1')
    expect(store.selectedTemplate?.id).toBe('prompt_1')
    expect(invokeMock).toHaveBeenCalledWith('list_prompt_templates')
  })

  it('copies, saves, resets, and previews through commands', async () => {
    invokeMock
      .mockResolvedValueOnce({ id: 'prompt_copy', builtIn: false })
      .mockResolvedValueOnce({ id: 'prompt_copy', name: '自定义审计', builtIn: false })
      .mockResolvedValueOnce([{ id: 'prompt_builtin', builtIn: true }])
      .mockResolvedValueOnce({ prompt: '最终 prompt', missingVariables: [] })

    const store = usePromptTemplateStore()
    await store.copyTemplate('prompt_1')
    await store.saveTemplate({
      id: 'prompt_copy',
      name: '自定义审计',
      purpose: 'logic_audit',
      template: '{{project_context}}',
    })
    await store.resetBuiltins()
    await store.preview('{{question}}', [{ name: 'question', value: '是否冲突' }])

    expect(invokeMock).toHaveBeenCalledWith('copy_prompt_template', { templateId: 'prompt_1' })
    expect(invokeMock).toHaveBeenCalledWith('save_prompt_template', {
      draft: {
        id: 'prompt_copy',
        name: '自定义审计',
        purpose: 'logic_audit',
        template: '{{project_context}}',
      },
    })
    expect(invokeMock).toHaveBeenCalledWith('reset_builtin_prompt_templates')
    expect(invokeMock).toHaveBeenCalledWith('preview_prompt_template', {
      request: {
        template: '{{question}}',
        values: [{ name: 'question', value: '是否冲突' }],
      },
    })
    expect(store.previewResult?.prompt).toBe('最终 prompt')
  })
})
