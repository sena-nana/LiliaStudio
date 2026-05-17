<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">索引</p>
        <h1>索引</h1>
      </div>
      <button type="button" class="primary-button" @click="runIndex">重建</button>
    </header>

    <div class="empty-state">
      <h2>切片</h2>
      <p>{{ aiStore.chunks.length }}</p>
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
