<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">诊断</p>
        <h1>本机状态</h1>
      </div>
      <button type="button" class="primary-button" @click="workflowStore.loadDiagnostics">刷新</button>
    </header>
    <div v-if="workflowStore.diagnostics" class="empty-state">
      <h2>{{ workflowStore.diagnostics.platform }}</h2>
      <p>版本 {{ workflowStore.diagnostics.appVersion }} · {{ workflowStore.diagnostics.database }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useWorkflowStore } from '@/stores/workflowStore'

const workflowStore = useWorkflowStore()
</script>
