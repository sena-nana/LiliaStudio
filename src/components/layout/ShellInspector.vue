<template>
  <section class="shell-inspector" aria-label="上下文">
    <h2>{{ title }}</h2>

    <section class="inspector-field">
      <span>项目</span>
      <strong>{{ projectName }}</strong>
    </section>

    <section class="inspector-field">
      <span>集合</span>
      <strong>{{ collectionName }}</strong>
    </section>

    <section class="inspector-actions">
      <RouterLink to="/search">搜索</RouterLink>
      <RouterLink to="/jobs">任务</RouterLink>
      <RouterLink to="/settings">设置</RouterLink>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { workspaceTabs, type WorkspaceKey } from './workspaceModel'

const props = defineProps<{
  workspace: WorkspaceKey
  projectName: string
  collectionName: string
}>()

const title = computed(() => {
  const tab = workspaceTabs.find((item) => item.key === props.workspace)
  return tab ? tab.label : '上下文'
})
</script>
