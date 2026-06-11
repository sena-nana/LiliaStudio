<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">索引</p>
        <h1>索引</h1>
      </div>
      <button
        type="button"
        class="primary-button"
        :disabled="aiStore.loading || !projectId"
        @click="runIndex"
      >
        重建
      </button>
    </header>

    <div v-if="!projectId" class="empty-state">
      <h2>未选择项目</h2>
      <p>先选择一个项目，再重建切片和 embedding 索引。</p>
    </div>

    <template v-else>
      <div class="indexing-metrics">
        <article class="empty-state compact">
          <h2>切片</h2>
          <p>{{ activeIndexState.chunkCount }}</p>
        </article>
        <article class="empty-state compact">
          <h2>Embedding</h2>
          <p>{{ activeIndexState.embeddingCount }}</p>
        </article>
        <article class="empty-state compact">
          <h2>缺失</h2>
          <p>{{ activeIndexState.missingEmbeddingCount }}</p>
        </article>
        <article class="empty-state compact">
          <h2>过期</h2>
          <p>{{ activeIndexState.staleEmbeddingCount }}</p>
        </article>
        <article class="empty-state compact">
          <h2>模型</h2>
          <p>{{ activeIndexState.model || "未生成" }}</p>
        </article>
      </div>

      <p v-if="activeIndexState.status === 'loading'" class="status-note">
        正在重建索引
      </p>
      <p v-else-if="activeIndexState.status === 'ready'" class="status-note">
        当前 embedding 索引可用
      </p>
      <p v-else-if="activeIndexState.status === 'degraded'" class="status-note">
        {{ activeIndexState.message }}
      </p>
      <p v-else-if="activeIndexState.status === 'failed'" class="status-note error">
        {{ activeIndexState.message }}
      </p>
      <p v-else class="status-note">
        尚未执行索引
      </p>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useAiStore } from '@/stores/aiStore'
import { useProjectStore } from '@/stores/projectStore'
import type { EmbeddingIndexState } from '@/types/ai'

const route = useRoute()
const aiStore = useAiStore()
const projectStore = useProjectStore()

const idleIndexState: EmbeddingIndexState = {
  chunkCount: 0,
  embeddingCount: 0,
  missingEmbeddingCount: 0,
  staleEmbeddingCount: 0,
  model: '',
  status: 'idle',
  message: '',
  lastProjectId: null,
}

const projectId = computed(() => {
  const value = route.params.projectId
  return typeof value === 'string' && value.length > 0 ? value : projectStore.activeProjectId
})

const activeIndexState = computed(() => {
  if (!projectId.value) return idleIndexState
  return aiStore.indexState.lastProjectId === projectId.value ? aiStore.indexState : idleIndexState
})

async function runIndex() {
  if (!projectId.value) return
  await aiStore.rebuildEmbeddingIndex(projectId.value, 600)
}

watch(
  projectId,
  (nextProjectId) => {
    if (!nextProjectId) return
    void aiStore.loadIndexStatus(nextProjectId).catch(() => undefined)
  },
  { immediate: true },
)
</script>

<style scoped>
.indexing-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

@media (max-width: 800px) {
  .indexing-metrics {
    grid-template-columns: 1fr;
  }
}
</style>
