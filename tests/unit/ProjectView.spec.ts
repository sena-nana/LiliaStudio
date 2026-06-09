import { fireEvent, render, screen, waitFor } from '@testing-library/vue'
import { invoke } from '@tauri-apps/api/core'
import { createPinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ProjectView from '@/views/ProjectView.vue'

const invokeMock = vi.mocked(invoke)

vi.mock('@/components/RichTextEditor.vue', () => ({
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
    if (command === 'list_projects') {
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
    if (command === 'list_entries') {
      return [
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
      ]
    }
    if (command === 'list_characters') {
      return [{ id: 'character_1', projectId: 'project_1', name: '椎名', faction: '北境' }]
    }
    if (command === 'list_events') {
      return [{ id: 'event_1', projectId: 'project_1', title: '围城战', timeLabel: '冬季' }]
    }
    if (command === 'search_axioms') {
      return [{ id: 'axiom_1', projectId: 'project_1', subject: '月光金属', predicate: '定义', object: '稀有' }]
    }
    if (command === 'list_relations') {
      return []
    }
    if (command === 'update_entry') {
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
    expect(screen.getByText('椎名')).toBeInTheDocument()
    expect(screen.getByText('围城战')).toBeInTheDocument()
    expect(screen.getByText('月光金属')).toBeInTheDocument()

    await fireEvent.click(screen.getByRole('button', { name: /月光阔剑/ }))

    await waitFor(() => {
      expect(view.container.querySelector('.record-row.active')).toHaveTextContent('月光阔剑')
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
      expect(invokeMock).toHaveBeenCalledWith('update_entry', {
        id: 'entry_1',
        draft: expect.objectContaining({
          projectId: 'project_1',
          title: '月光阔剑',
          tags: ['新标签', '规则'],
        }),
      })
    })
  })
})
