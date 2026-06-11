import { fireEvent, render, screen } from '@testing-library/vue'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import IndexingView from '@/views/IndexingView.vue'
import { useAiStore } from '@/stores/aiStore'
import { useProjectStore } from '@/stores/projectStore'
import type { EmbeddingIndexState } from '@/types/ai'

async function renderIndexingView(pinia: Pinia, path = '/indexing/project_1') {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/indexing/:projectId?', component: IndexingView }],
  })
  await router.push(path)
  await router.isReady()
  return render(IndexingView, {
    global: {
      plugins: [pinia, router],
    },
  })
}

function makeIndexState(overrides: Partial<EmbeddingIndexState> = {}): EmbeddingIndexState {
  return {
    chunkCount: 0,
    embeddingCount: 0,
    missingEmbeddingCount: 0,
    staleEmbeddingCount: 0,
    model: '',
    status: 'idle',
    message: '',
    lastProjectId: null,
    ...overrides,
  }
}

describe('IndexingView', () => {
  let pinia: Pinia

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('renders index metrics and degraded status for the active project', async () => {
    const aiStore = useAiStore()
    vi.spyOn(aiStore, 'loadIndexStatus').mockResolvedValue(makeIndexState({ lastProjectId: 'project_1' }))
    await renderIndexingView(pinia)
    const projectStore = useProjectStore()
    projectStore.activeProjectId = 'project_1'
    aiStore.indexState = makeIndexState({
      chunkCount: 14,
      embeddingCount: 0,
      missingEmbeddingCount: 14,
      staleEmbeddingCount: 0,
      model: '',
      status: 'degraded',
      message: '请先在 AI 设置中启用并配置 OpenAI 兼容接口的嵌入模型和密钥。',
      lastProjectId: 'project_1',
    })
    await nextTick()

    expect((await screen.findAllByText('14')).length).toBeGreaterThan(0)
    expect(screen.getAllByText('0').length).toBeGreaterThan(0)
    expect(screen.getByText('未生成')).toBeInTheDocument()
    expect(screen.getByText('缺失')).toBeInTheDocument()
    expect(screen.getByText('过期')).toBeInTheDocument()
    expect(screen.getByText('请先在 AI 设置中启用并配置 OpenAI 兼容接口的嵌入模型和密钥。')).toBeInTheDocument()
  })

  it('renders failed status and triggers rebuild on click', async () => {
    const aiStore = useAiStore()
    vi.spyOn(aiStore, 'loadIndexStatus').mockResolvedValue(makeIndexState({ lastProjectId: 'project_1' }))
    await renderIndexingView(pinia)
    const rebuildSpy = vi.spyOn(aiStore, 'rebuildEmbeddingIndex').mockResolvedValue(
      makeIndexState({
        chunkCount: 6,
        embeddingCount: 6,
        model: 'story-embed',
        status: 'ready',
        lastProjectId: 'project_1',
      }),
    )
    aiStore.indexState = makeIndexState({
      chunkCount: 6,
      embeddingCount: 6,
      model: 'story-embed',
      status: 'failed',
      message: 'provider returned HTTP 401',
      lastProjectId: 'project_1',
    })
    await nextTick()

    expect(await screen.findByText('provider returned HTTP 401')).toBeInTheDocument()
    await fireEvent.click(screen.getByRole('button', { name: '重建' }))

    expect(rebuildSpy).toHaveBeenCalledWith('project_1', 600)
  })

  it('shows empty state without a selected project', async () => {
    await renderIndexingView(pinia, '/indexing')

    expect(await screen.findByRole('heading', { name: '未选择项目' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '重建' })).toBeDisabled()
  })
})
