<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">时间线</p>
        <h1>事件演化</h1>
      </div>
    </header>

    <div class="timeline">
      <article v-for="event in sortedEvents" :key="event.id">
        <time>{{ event.timeLabel || '未定时间' }}</time>
        <div>
          <h2>{{ event.title }}</h2>
          <p>{{ event.description || event.outcome || '暂无描述' }}</p>
        </div>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useLibraryStore } from '@/stores/libraryStore'
import { useProjectStore } from '@/stores/projectStore'

const route = useRoute()
const projectStore = useProjectStore()
const libraryStore = useLibraryStore()
const projectId = computed(() => {
  const value = route.params.projectId
  return typeof value === 'string' && value.length > 0 ? value : projectStore.activeProjectId
})
const sortedEvents = computed(() =>
  [...libraryStore.events].sort((left, right) => left.sortKey - right.sortKey),
)

onMounted(() => {
  if (projectId.value) {
    void libraryStore.loadProject(projectId.value)
  }
})
</script>
