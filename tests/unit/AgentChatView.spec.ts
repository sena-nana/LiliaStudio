import { fireEvent, render, screen } from '@testing-library/vue'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import AgentChatView from '@/views/AgentChatView.vue'
import { askAgent } from '@/api/ai'
import { useJobStore } from '@/stores/jobStore'
import { useLibraryStore } from '@/stores/libraryStore'
import { useProjectStore } from '@/stores/projectStore'
import { useSearchStore } from '@/stores/searchStore'
import type { AgentAskResponse } from '@/types/ai'

const QUESTION = '月光阔剑有什么风险？'
const ANSWER = '月光阔剑的主要风险是补给不稳定。'
const PROJECT = {
  id: 'project_1',
  name: '北境档案',
  description: '',
  createdAt: '',
  updatedAt: '',
  archivedAt: null,
}
const KEYWORD_RESULT = {
  entityType: 'entry',
  entityId: 'entry_1',
  title: '月光阔剑',
  snippet: '潮汐能武器',
  score: 2,
}
const AGENT_RESPONSE: AgentAskResponse = {
  answer: ANSWER,
  providerKind: 'openAiCompatible',
  model: 'story-chat',
  references: [{ ...KEYWORD_RESULT, source: 'keyword' }],
}

vi.mock('@/api/ai', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/ai')>()
  return {
    ...actual,
    askAgent: vi.fn(),
  }
})

function renderAgentChatView(pinia: Pinia) {
  return render(AgentChatView, {
    global: {
      plugins: [pinia],
    },
  })
}

describe('AgentChatView', () => {
  let pinia: Pinia

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    vi.mocked(askAgent).mockReset()
  })

  it('shows empty project state and disables submit without project', async () => {
    renderAgentChatView(pinia)
    const projectStore = useProjectStore()
    vi.spyOn(projectStore, 'loadProjects').mockResolvedValue(undefined)
    await nextTick()

    expect(await screen.findByRole('heading', { name: '未选择项目' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '提交' })).toBeDisabled()
  })

  it('submits a question through search and agent api', async () => {
    setupActiveProject()
    renderAgentChatView(pinia)
    const searchStore = mockSearchResults({
      semanticResults: [
        {
          entityType: 'event',
          entityId: 'event_1',
          title: '围城战',
          snippet: '首次公开使用',
          score: 0.78,
        },
      ],
      semanticStatus: 'degraded',
      semanticMessage: '当前项目还没有可用的 embedding 索引。',
    })
    vi.mocked(askAgent).mockResolvedValue(AGENT_RESPONSE)
    await nextTick()

    await fireEvent.update(screen.getByPlaceholderText('基于当前项目资料提问'), QUESTION)
    await fireEvent.click(screen.getByRole('button', { name: '提交' }))

    expect(searchStore.run).toHaveBeenCalledWith('project_1', QUESTION)
    expect(askAgent).toHaveBeenCalledWith({
      projectId: 'project_1',
      question: QUESTION,
      references: expect.arrayContaining([
        expect.objectContaining({
          entityType: 'entry',
          entityId: 'entry_1',
          title: '月光阔剑',
          source: 'keyword',
        }),
      ]),
    })
    expect(await screen.findByText(ANSWER)).toBeInTheDocument()
    expect(screen.getByText('当前项目还没有可用的 embedding 索引。')).toBeInTheDocument()
  })

  it('saves the latest answer as a local report entry', async () => {
    await submitWithAnswer(pinia)
    const libraryStore = useLibraryStore()
    const createEntrySpy = vi.spyOn(libraryStore, 'createEntry').mockResolvedValue({
      id: 'entry_report',
      projectId: 'project_1',
      entryType: 'report',
      title: `Agent 问答：${QUESTION}`,
      summary: QUESTION,
      body: '',
      tags: [],
      status: 'active',
      createdAt: '',
      updatedAt: '',
      deletedAt: null,
    })

    await fireEvent.click(screen.getByRole('button', { name: '保存为报告' }))

    expect(createEntrySpy).toHaveBeenCalledWith(
      expect.objectContaining({
        projectId: 'project_1',
        entryType: 'report',
        status: 'active',
        summary: QUESTION,
        body: expect.stringContaining(ANSWER),
      }),
    )
    expect(await screen.findByText('已保存为本地报告')).toBeInTheDocument()
  })

  it('saves the latest answer as an agent chat job record', async () => {
    await submitWithAnswer(pinia)
    const jobStore = useJobStore()
    const createJobSpy = vi.spyOn(jobStore, 'createJob').mockResolvedValue({
      id: 'job_1',
      projectId: 'project_1',
      providerKind: 'openAiCompatible',
      jobType: 'agentChat',
      status: 'queued',
      inputSummary: QUESTION,
      outputText: '',
      errorMessage: null,
      startedAt: null,
      finishedAt: null,
      cancelRequestedAt: null,
      retryOfJobId: null,
      createdAt: '',
      updatedAt: '',
    })

    await fireEvent.click(screen.getByRole('button', { name: '保存为任务' }))

    expect(createJobSpy).toHaveBeenCalledWith({
      projectId: 'project_1',
      providerKind: 'openAiCompatible',
      jobType: 'agentChat',
      inputSummary: QUESTION,
    })
    expect(await screen.findByText('已保存为任务')).toBeInTheDocument()
  })
})

async function submitWithAnswer(pinia: Pinia) {
  setupActiveProject()
  renderAgentChatView(pinia)
  mockSearchResults()
  vi.mocked(askAgent).mockResolvedValue(AGENT_RESPONSE)
  await nextTick()

  await fireEvent.update(screen.getByPlaceholderText('基于当前项目资料提问'), QUESTION)
  await fireEvent.click(screen.getByRole('button', { name: '提交' }))
  await screen.findByText(ANSWER)
}

function setupActiveProject() {
  const projectStore = useProjectStore()
  projectStore.projects = [PROJECT]
  projectStore.activeProjectId = 'project_1'
}

function mockSearchResults(
  overrides: Partial<Pick<ReturnType<typeof useSearchStore>, 'semanticResults' | 'semanticStatus' | 'semanticMessage'>> = {},
) {
  const searchStore = useSearchStore()
  vi.spyOn(searchStore, 'run').mockImplementation(async () => {
    searchStore.keywordResults = [KEYWORD_RESULT]
    searchStore.semanticResults = overrides.semanticResults ?? []
    searchStore.semanticStatus = overrides.semanticStatus ?? 'idle'
    searchStore.semanticMessage = overrides.semanticMessage ?? ''
  })
  return searchStore
}
