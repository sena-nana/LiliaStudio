<template>
  <div class="app-shell">
    <header class="app-header">
      <div class="brand">
        <span class="brand-mark">A</span>
        <div>
          <strong>Ameya</strong>
          <small>{{ projectLabel }}</small>
        </div>
      </div>

      <ShellTopTabs :active-workspace="currentWorkspace" />

      <div class="header-actions">
        <RouterLink class="header-action" to="/search">搜索</RouterLink>
        <details class="shell-menu">
          <summary>更多</summary>
          <div class="shell-menu-panel">
            <RouterLink v-for="item in workspaceMenuEntries" :key="item.to" :to="item.to">
              {{ item.label }}
            </RouterLink>
          </div>
        </details>
      </div>
    </header>

    <aside class="app-sidebar" aria-label="集合栏">
      <ShellCollectionRail
        :workspace="currentWorkspace"
        :active-collection="activeCollection"
        @select="selectCollection"
      />
    </aside>

    <main class="app-main">
      <RouterView />
    </main>

    <aside class="context-panel" aria-label="上下文面板">
      <ShellInspector
        :workspace="currentWorkspace"
        :project-name="projectLabel"
        :collection-name="activeCollectionLabel"
      />
    </aside>

    <footer class="status-bar">
      <span>本地优先</span>
      <span>Windows</span>
      <JobStatusBar />
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import JobStatusBar from '@/components/jobs/JobStatusBar.vue'
import ShellCollectionRail from '@/components/layout/ShellCollectionRail.vue'
import ShellInspector from '@/components/layout/ShellInspector.vue'
import ShellTopTabs from '@/components/layout/ShellTopTabs.vue'
import {
  getWorkspaceCollections,
  resolveWorkspaceKey,
  workspaceMenuEntries,
  type WorkspaceKey,
} from './workspaceModel'
import { useProjectStore } from '@/stores/projectStore'

const route = useRoute()
const projectStore = useProjectStore()
const currentWorkspace = ref<WorkspaceKey>('project')
const activeCollections = reactive<Record<WorkspaceKey, string>>({
  project: 'recent',
  library: 'all',
  character: 'all',
  event: 'all',
  rule: 'all',
  relation: 'all',
  analysis: 'search',
})

watch(
  () => route.name,
  (name) => {
    currentWorkspace.value = resolveWorkspaceKey(name, currentWorkspace.value)
    const collections = getWorkspaceCollections(currentWorkspace.value)
    const selectedCollection = activeCollections[currentWorkspace.value]

    if (!collections.some((collection) => collection.key === selectedCollection)) {
      activeCollections[currentWorkspace.value] = collections[0]?.key ?? ''
    }
  },
  { immediate: true },
)

const projectLabel = computed(() => projectStore.activeProject?.name ?? '未选择项目')
const activeCollection = computed(() => activeCollections[currentWorkspace.value])
const activeCollectionLabel = computed(() => {
  return (
    getWorkspaceCollections(currentWorkspace.value).find(
      (collection) => collection.key === activeCollection.value,
    )?.label ?? '未选择集合'
  )
})

function selectCollection(collectionKey: string) {
  activeCollections[currentWorkspace.value] = collectionKey
}
</script>
