import { fireEvent, render, screen, waitFor } from '@testing-library/vue'
import { invoke } from '@tauri-apps/api/core'
import type { InvokeArgs } from '@tauri-apps/api/core'
import { createPinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import CharacterGrowthView from '@/views/CharacterGrowthView.vue'

const invokeMock = vi.mocked(invoke)

async function renderGrowthView(path = '/growth/project_1') {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/growth/:projectId?', component: CharacterGrowthView }],
  })
  await router.push(path)
  await router.isReady()
  return render(CharacterGrowthView, {
    global: {
      plugins: [createPinia(), router],
    },
  })
}

const project = {
  id: 'project_1',
  name: '雨夜都市',
  description: '',
  createdAt: '',
  updatedAt: '',
  archivedAt: null,
}

const character = {
  id: 'character_1',
  projectId: 'project_1',
  name: '椎名',
  aliases: [],
  summary: '冷静的调查者',
  appearance: '',
  goals: '',
  motivations: '',
  fears: '',
  faction: '北境',
  tags: [],
  createdAt: '',
  updatedAt: '',
  deletedAt: null,
}

const event = {
  id: 'event_1',
  projectId: 'project_1',
  title: '围城战',
  description: '',
  timeLabel: '冬季',
  sortKey: 1,
  startLabel: '',
  endLabel: '',
  location: '',
  importance: 5,
  outcome: '',
  tags: [],
  createdAt: '',
  updatedAt: '',
  deletedAt: null,
}

function mockGrowthWorkspace(options: { characters?: unknown[]; events?: unknown[]; failPreview?: boolean } = {}) {
  const characters = options.characters ?? [character]
  const events = options.events ?? [event]
  const records = [
    {
      id: 'record_1',
      projectId: 'project_1',
      characterId: 'character_1',
      sourceEventId: 'event_1',
      traitName: 'responsibility',
      delta: 0.25,
      reason: '守住城墙',
      createdAt: '2026-06-10T00:00:00Z',
      updatedAt: '2026-06-10T00:00:00Z',
      deletedAt: null,
    },
  ]
  let saved = false

  invokeMock.mockImplementation(async (command, args) => {
    const invokeArgs = asRecord(args)
    if (command === 'project_list') return [project]
    if (command === 'library_project_snapshot') {
      return {
        projectId: 'project_1',
        entries: [],
        characters,
        events,
        axioms: [],
        relations: [],
      }
    }
    if (command === 'character_growth_workspace') {
      return {
        projectId: 'project_1',
        records: saved
          ? [
              ...records,
              {
                id: 'record_2',
                projectId: 'project_1',
                characterId: 'character_1',
                sourceEventId: 'event_1',
                traitName: 'responsibility',
                delta: 0.5,
                reason: '保护平民',
                createdAt: '2026-06-11T00:00:00Z',
                updatedAt: '2026-06-11T00:00:00Z',
                deletedAt: null,
              },
            ]
          : records,
        states: {
          character_1: {
            values: { responsibility: saved ? 0.75 : 0.25 },
            sources: [],
          },
        },
      }
    }
    if (command === 'character_growth_preview_trait_delta') {
      if (options.failPreview) throw new Error('preview failed')
      return {
        values: { responsibility: 0.75 },
        sources: [invokeArgs.delta],
      }
    }
    if (command === 'character_growth_create_record') {
      saved = true
      return {
        id: 'record_2',
        ...(invokeArgs.draft as object),
        createdAt: '2026-06-11T00:00:00Z',
        updatedAt: '2026-06-11T00:00:00Z',
        deletedAt: null,
      }
    }
    return []
  })
}

function asRecord(args: InvokeArgs | undefined): Record<string, unknown> {
  return args && !Array.isArray(args) ? (args as Record<string, unknown>) : {}
}

describe('CharacterGrowthView', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('shows the empty state without a selected project', async () => {
    invokeMock.mockResolvedValue([])

    await renderGrowthView('/growth')

    expect(await screen.findByRole('heading', { name: '未选择项目' })).toBeInTheDocument()
  })

  it('shows empty states for missing characters and events', async () => {
    mockGrowthWorkspace({ characters: [] })
    await renderGrowthView()
    expect(await screen.findByRole('heading', { name: '暂无角色' })).toBeInTheDocument()

    invokeMock.mockReset()
    mockGrowthWorkspace({ events: [] })
    await renderGrowthView()
    expect(await screen.findByRole('heading', { name: '暂无事件' })).toBeInTheDocument()
  })

  it('renders character state, previews trait delta, and saves a record', async () => {
    mockGrowthWorkspace()

    await renderGrowthView()

    expect(await screen.findByRole('button', { name: /椎名/ })).toBeInTheDocument()
    expect(screen.getByText('+0.25')).toBeInTheDocument()

    await fireEvent.update(screen.getByLabelText('来源事件'), 'event_1')
    await fireEvent.update(screen.getByLabelText('Trait 名称'), 'responsibility')
    await fireEvent.update(screen.getByLabelText('Delta'), '0.5')
    await fireEvent.update(screen.getByLabelText('原因'), '保护平民')
    await fireEvent.click(screen.getByRole('button', { name: '预览变化' }))

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('character_growth_preview_trait_delta', {
        state: expect.objectContaining({ values: { responsibility: 0.25 } }),
        delta: {
          sourceEventId: 'event_1',
          traitName: 'responsibility',
          delta: 0.5,
          reason: '保护平民',
        },
      })
    })
    expect(await screen.findByText('已更新预览结果')).toBeInTheDocument()

    await fireEvent.click(screen.getByRole('button', { name: '保存记录' }))
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('character_growth_create_record', {
        draft: {
          projectId: 'project_1',
          characterId: 'character_1',
          sourceEventId: 'event_1',
          traitName: 'responsibility',
          delta: 0.5,
          reason: '保护平民',
        },
      })
    })
    expect(await screen.findByText('成长记录已保存')).toBeInTheDocument()
    expect(screen.getByText('+0.75')).toBeInTheDocument()
    expect(screen.getByText('保护平民')).toBeInTheDocument()
  })

  it('shows preview errors in result status', async () => {
    mockGrowthWorkspace({ failPreview: true })

    await renderGrowthView()
    await fireEvent.update(await screen.findByLabelText('来源事件'), 'event_1')
    await fireEvent.update(screen.getByLabelText('Trait 名称'), 'responsibility')
    await fireEvent.update(screen.getByLabelText('Delta'), '0.5')
    await fireEvent.update(screen.getByLabelText('原因'), '保护平民')
    await fireEvent.click(screen.getByRole('button', { name: '预览变化' }))

    expect(await screen.findByText('preview failed')).toBeInTheDocument()
    expect(screen.getByText('失败')).toBeInTheDocument()
  })
})
