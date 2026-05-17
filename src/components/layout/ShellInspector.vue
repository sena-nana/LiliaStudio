<template>
  <section class="shell-inspector" aria-label="Inspector">
    <p class="eyebrow">Inspector</p>
    <h2>{{ title }}</h2>
    <p class="shell-inspector-summary">{{ summary }}</p>

    <section class="inspector-section">
      <h3>当前项目</h3>
      <p>{{ projectName }}</p>
    </section>

    <section class="inspector-section">
      <h3>当前集合</h3>
      <p>{{ collectionName }}</p>
    </section>

    <section class="inspector-section">
      <h3>快捷动作</h3>
      <RouterLink to="/search">全局搜索</RouterLink>
      <RouterLink to="/jobs">查看任务</RouterLink>
      <RouterLink to="/settings">打开设置</RouterLink>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { getWorkspaceSummary, workspaceTabs, type WorkspaceKey } from './workspaceModel'

const props = defineProps<{
  workspace: WorkspaceKey
  projectName: string
  collectionName: string
}>()

const summary = computed(() => getWorkspaceSummary(props.workspace))
const title = computed(() => {
  const tab = workspaceTabs.find((item) => item.key === props.workspace)
  return tab ? `${tab.label}上下文` : '上下文'
})
</script>
