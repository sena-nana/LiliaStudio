<template>
  <section class="collection-rail" aria-label="集合栏">
    <header class="collection-rail-header">
      <div>
        <p class="eyebrow">集合</p>
        <h2>{{ title }}</h2>
      </div>
    </header>

    <p class="collection-rail-summary">{{ summary }}</p>

    <nav class="collection-list">
      <button
        v-for="collection in collections"
        :key="collection.key"
        type="button"
        class="collection-item"
        :class="{ active: collection.key === activeCollection }"
        :aria-pressed="collection.key === activeCollection"
        @click="emit('select', collection.key)"
      >
        <strong>{{ collection.label }}</strong>
        <span>{{ collection.description }}</span>
      </button>
    </nav>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  getWorkspaceCollections,
  getWorkspaceSummary,
  workspaceTabs,
  type WorkspaceKey,
} from './workspaceModel'

const props = defineProps<{
  workspace: WorkspaceKey
  activeCollection: string
}>()

const emit = defineEmits<{
  select: [collectionKey: string]
}>()

const collections = computed(() => getWorkspaceCollections(props.workspace))
const summary = computed(() => getWorkspaceSummary(props.workspace))
const title = computed(() => {
  const tab = workspaceTabs.find((item) => item.key === props.workspace)
  return tab ? `${tab.label}集合` : '集合'
})
</script>
