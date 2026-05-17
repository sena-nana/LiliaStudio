<template>
  <nav class="workspace-tabs" aria-label="对象标签">
    <RouterLink
      v-for="tab in workspaceTabs"
      :key="tab.key"
      :to="tab.to"
      class="workspace-tab"
      :class="{ active: tab.key === activeWorkspace }"
      :aria-current="tab.key === activeWorkspace ? 'page' : undefined"
    >
      <strong>{{ tab.label }}</strong>
      <span>{{ tab.hint }}</span>
    </RouterLink>
  </nav>
</template>

<script setup lang="ts">
import { workspaceTabs, type WorkspaceKey } from './workspaceModel'

defineProps<{
  activeWorkspace: WorkspaceKey
}>()
</script>
