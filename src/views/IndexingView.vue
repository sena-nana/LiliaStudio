<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">索引</p>
        <h1>向量与上下文准备</h1>
      </div>
      <button type="button" class="primary-button" @click="runIndex">重建文本切片</button>
    </header>

    <div class="empty-state">
      <h2>DocumentChunk</h2>
      <p>当前项目已生成 {{ aiStore.chunks.length }} 个文本切片。</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useAiStore } from '@/stores/aiStore'
import { useProjectStore } from '@/stores/projectStore'

const route = useRoute()
const aiStore = useAiStore()
const projectStore = useProjectStore()
const projectId = computed(() => {
  const value = route.params.projectId
  return typeof value === 'string' && value.length > 0 ? value : projectStore.activeProjectId
})

async function runIndex() {
  if (!projectId.value) return
  await aiStore.indexProject(projectId.value, 600)
}
</script>
