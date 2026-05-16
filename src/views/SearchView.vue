<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">搜索</p>
        <h1>项目内检索</h1>
      </div>
    </header>

    <form class="quick-create" @submit.prevent="runSearch">
      <input v-model="query" placeholder="搜索词条、角色、事件和公理" />
      <button type="submit">搜索</button>
    </form>

    <div class="project-list">
      <article v-for="result in searchStore.results" :key="`${result.entityType}:${result.entityId}`" class="project-row">
        <strong>{{ result.title }}</strong>
        <span>{{ result.entityType }} · {{ result.snippet || '无摘要' }}</span>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
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
</script>
