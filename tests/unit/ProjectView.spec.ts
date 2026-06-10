import { fireEvent, render, screen, waitFor } from '@testing-library/vue'
import { invoke } from '@tauri-apps/api/core'
import { createPinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ProjectView from '@/views/ProjectView.vue'

const invokeMock = vi.mocked(invoke)

vi.mock('@/components/RichTextEditor.vue', () => ({
  __isTeleport: false,
  default: {
    props: ['modelValue'],
    emits: ['update:modelValue'],
    template: '<textarea aria-label="rich text" :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
  },
}))

async function renderProjectView(path = '/projects/project_1') {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/projects/:projectId?', component: ProjectView }],
  })
  await router.push(path)
  await router.isReady()
  return render(ProjectView, {
    global: {
      plugins: [createPinia(), router],
    },
  })
}

function mockProjectWorkspace() {
  invokeMock.mockImplementation(async (command) => {
    if (command === 'project_list') {
      return [
        {
          id: 'project_1',
          name: '雨夜都市',
          description: '',
          createdAt: '',
          updatedAt: '',
          archivedAt: null,
        },
      ]
    }
    if (command === 'library_project_snapshot') {
      return {
        projectId: 'project_1',
        entries: [
        {
          id: 'entry_1',
          projectId: 'project_1',
          entryType: 'item',
          title: '月光阔剑',
          summary: '',
          body: '',
          tags: ['旧标签'],
          status: 'draft',
          createdAt: '',
          updatedAt: '',
          deletedAt: null,
        },
        ],
        characters: [
          {
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
          },
        ],
        events: [
          {
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
          },
        ],
        axioms: [
          {
            id: 'axiom_1',
            projectId: 'project_1',
            subject: '月光金属',
            predicate: '定义',
            object: '稀有',
            scopeTime: '',
            scopeLocation: '',
            certainty: 1,
            sourceEntityType: null,
            sourceEntityId: null,
            naturalLanguage: '',
            tags: [],
            createdAt: '',
            updatedAt: '',
            deletedAt: null,
          },
        ],
        relations: [],
      }
    }
    if (command === 'library_relation_type_presets') {
      return ['参与事件', '语义相关']
    }
    if (command === 'library_relation_neighborhood') {
      return {
        center: {
          entityType: 'character',
          entityId: 'character_1',
          title: '椎名',
          subtitle: '北境',
          summary: '冷静的调查者',
        },
        nodes: [
          {
            entityType: 'character',
            entityId: 'character_1',
            title: '椎名',
            subtitle: '北境',
            summary: '冷静的调查者',
          },
          {
            entityType: 'event',
            entityId: 'event_1',
            title: '围城战',
            subtitle: '冬季',
            summary: '',
          },
        ],
        edges: [],
        suggestions: [
          {
            source: { entityType: 'character', entityId: 'character_1' },
            target: { entityType: 'event', entityId: 'event_1' },
            relationType: '参与事件',
            description: '同一事件参与者',
            confidence: 0.95,
            directed: true,
            reason: '同一事件参与者',
            strength: '高',
          },
        ],
        missing: ['存在待确认关联建议'],
        relationCount: 0,
      }
    }
    if (command === 'library_create_relation') {
      return {
        id: 'relation_1',
        projectId: 'project_1',
        source: { entityType: 'character', entityId: 'character_1' },
        target: { entityType: 'event', entityId: 'event_1' },
        relationType: '参与事件',
        description: '同一事件参与者',
        confidence: 0.95,
        directed: true,
        createdAt: '',
        updatedAt: '',
        deletedAt: null,
      }
    }
    if (command === 'library_update_entry') {
      return {
        id: 'entry_1',
        projectId: 'project_1',
        entryType: 'item',
        title: '月光阔剑',
        summary: '',
        body: '',
        tags: ['新标签', '规则'],
        status: 'draft',
        createdAt: '',
        updatedAt: '',
        deletedAt: null,
      }
    }
    return []
  })
}

describe('ProjectView', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('shows the empty state without a selected project', async () => {
    invokeMock.mockResolvedValue([])

    await renderProjectView('/projects')

    expect(await screen.findByRole('heading', { name: '未选择项目' })).toBeInTheDocument()
  })

  it('renders library panels and selects a record', async () => {
    mockProjectWorkspace()

    const view = await renderProjectView()

    expect(await screen.findByText('月光阔剑')).toBeInTheDocument()
    await fireEvent.click(screen.getByRole('button', { name: /角色/ }))
    expect(screen.getByText('椎名')).toBeInTheDocument()
    await fireEvent.click(screen.getByRole('button', { name: /事件/ }))
    expect(screen.getByText('围城战')).toBeInTheDocument()
    await fireEvent.click(screen.getByRole('button', { name: /规则/ }))
    expect(screen.getByText('月光金属')).toBeInTheDocument()

    await fireEvent.click(screen.getByRole('button', { name: /资料/ }))
    await fireEvent.click(screen.getByRole('button', { name: /月光阔剑/ }))

    await waitFor(() => {
      expect(view.container.querySelector('.object-table-row.active')).toHaveTextContent('月光阔剑')
    })
    expect(screen.getByDisplayValue('月光阔剑')).toBeInTheDocument()
  })

  it('saves an entry with parsed tags through the existing command', async () => {
    mockProjectWorkspace()

    await renderProjectView()
    await fireEvent.click(await screen.findByRole('button', { name: /月光阔剑/ }))
    await fireEvent.update(screen.getByDisplayValue('旧标签'), '新标签，规则')
    await fireEvent.click(screen.getByRole('button', { name: '保存' }))

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('library_update_entry', {
        id: 'entry_1',
        draft: expect.objectContaining({
          projectId: 'project_1',
          title: '月光阔剑',
          tags: ['新标签', '规则'],
        }),
      })
    })
  })

  it('shows relation suggestions in the inspector and confirms one as a relation', async () => {
    mockProjectWorkspace()

    await renderProjectView()
    await fireEvent.click(await screen.findByRole('button', { name: /角色/ }))
    await fireEvent.click(await screen.findByRole('button', { name: /椎名/ }))

    expect(await screen.findByText(/参与事件 · 高/)).toBeInTheDocument()
    await fireEvent.click(screen.getByRole('button', { name: '确认' }))

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('library_create_relation', {
        draft: expect.objectContaining({
          projectId: 'project_1',
          relationType: '参与事件',
          source: { entityType: 'character', entityId: 'character_1' },
          target: { entityType: 'event', entityId: 'event_1' },
        }),
      })
    })
  })
})
