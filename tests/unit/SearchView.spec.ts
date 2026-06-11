import { fireEvent, render, screen } from '@testing-library/vue'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SearchView from '@/views/SearchView.vue'
import { useProjectStore } from '@/stores/projectStore'
import { useSearchStore } from '@/stores/searchStore'

async function renderSearchView(pinia: Pinia, path = '/search/project_1') {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/search/:projectId?', component: SearchView }],
  })
  await router.push(path)
  await router.isReady()
  return render(SearchView, {
    global: {
      plugins: [pinia, router],
    },
  })
}

describe('SearchView', () => {
  let pinia: Pinia

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('renders mixed results and semantic degraded message', async () => {
    await renderSearchView(pinia)
    const projectStore = useProjectStore()
    projectStore.activeProjectId = 'project_1'
    const searchStore = useSearchStore()
    vi.spyOn(searchStore, 'run').mockImplementation(async () => {
      searchStore.keywordResults = [
        { entityType: 'entry', entityId: 'entry_1', title: '月光阔剑', snippet: '潮汐能武器', score: 2 },
      ]
      searchStore.semanticResults = [
        { entityType: 'event', entityId: 'event_1', title: '围城战', snippet: '首次公开使用', score: 0.78 },
      ]
      searchStore.semanticStatus = 'degraded'
      searchStore.semanticMessage = '当前项目还没有可用的 embedding 索引。'
    })

    await fireEvent.update(screen.getByPlaceholderText('搜索'), '月光')
    await fireEvent.click(screen.getByRole('button', { name: '搜索' }))

    expect(await screen.findByText('当前项目还没有可用的 embedding 索引。')).toBeInTheDocument()
    expect(screen.getByText('关键词')).toBeInTheDocument()
    expect(screen.getByText('语义')).toBeInTheDocument()
    expect(screen.getByText((content) => content.includes('相似度 0.78'))).toBeInTheDocument()
  })

  it('shows empty project state without project id', async () => {
    await renderSearchView(pinia, '/search')

    expect(await screen.findByRole('heading', { name: '未选择项目' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '搜索' })).toBeDisabled()
  })
})
