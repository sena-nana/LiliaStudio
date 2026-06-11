<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">搜索</p>
        <h1>搜索</h1>
      </div>
    </header>

    <form class="quick-create" @submit.prevent="runSearch">
      <input v-model="query" placeholder="搜索" />
      <button type="submit" class="primary-button" :disabled="searchStore.loading || !projectId">
        搜索
      </button>
    </form>

    <div v-if="!projectId" class="empty-state">
      <h2>未选择项目</h2>
      <p>先选择一个项目，再执行关键词和语义搜索。</p>
    </div>

    <template v-else>
      <p v-if="searchStore.loading" class="status-note">正在搜索</p>
      <p v-else-if="searchStore.semanticStatus === 'degraded'" class="status-note">
        {{ searchStore.semanticMessage }}
      </p>
      <p v-else-if="searchStore.semanticStatus === 'failed'" class="status-note error">
        {{ searchStore.semanticMessage }}
      </p>

      <div v-if="searchStore.results.length === 0" class="empty-state">
        <h2>暂无结果</h2>
        <p>输入查询后会先显示关键词结果，再补充语义命中。</p>
      </div>

      <div v-else class="project-list">
        <article
          v-for="result in searchStore.results"
          :key="`${result.source}:${result.entityType}:${result.entityId}`"
          class="project-row search-result-row"
        >
          <div class="search-result-title">
            <strong>{{ result.title }}</strong>
            <span class="status-chip" :class="result.source">{{ result.source === 'keyword' ? '关键词' : '语义' }}</span>
          </div>
          <span>
            {{ entityTypeLabel(result.entityType) }}
            <template v-if="result.source === 'semantic'"> · 相似度 {{ formatScore(result.score) }}</template>
            · {{ result.snippet || '无摘要' }}
          </span>
        </article>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { entityTypeLabel } from '@/domain/displayLabels'
import { useProjectStore } from '@/stores/projectStore'
import { useSearchStore } from '@/stores/searchStore'

const route = useRoute()
const projectStore = useProjectStore()
const searchStore = useSearchStore()
const query = ref('')
const projectId = computed(() => {
  const value = route.params.projectId
  return typeof value === 'string' && value.length > 0 ? value : projectStore.activeProjectId
})

async function runSearch() {
  if (!projectId.value || !query.value.trim()) return
  await searchStore.run(projectId.value, query.value.trim())
}

function formatScore(score: number): string {
  return score.toFixed(2)
}
</script>

<style scoped>
.search-result-row {
  display: grid;
  gap: 6px;
}

.search-result-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.status-chip.keyword {
  color: var(--text-muted);
}

.status-chip.semantic {
  color: var(--accent);
}
</style>
