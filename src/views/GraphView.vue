<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">图谱</p>
        <h1>关系邻域</h1>
      </div>
    </header>

    <div class="graph-board">
      <div v-for="node in nodes" :key="node.id" class="graph-node">
        <strong>{{ node.title }}</strong>
        <span>{{ node.type }}</span>
      </div>
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
const nodes = computed(() => [
  ...libraryStore.entries.slice(0, 20).map((entry) => ({ id: entry.id, title: entry.title, type: entry.entryType })),
  ...libraryStore.characters.slice(0, 20).map((character) => ({ id: character.id, title: character.name, type: 'character' })),
  ...libraryStore.events.slice(0, 20).map((event) => ({ id: event.id, title: event.title, type: 'event' })),
  ...libraryStore.axioms.slice(0, 20).map((axiom) => ({ id: axiom.id, title: axiom.subject, type: 'axiom' })),
])

onMounted(() => {
  if (projectId.value) {
    void libraryStore.loadProject(projectId.value)
  }
})
</script>
