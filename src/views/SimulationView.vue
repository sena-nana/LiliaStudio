<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">模拟</p>
        <h1>平行时空推演</h1>
      </div>
      <button type="button" class="primary-button" @click="run">运行样例模拟</button>
    </header>
    <article v-if="workflowStore.simulation" class="empty-state">
      <h2>{{ workflowStore.simulation.scenario }}</h2>
      <p v-for="phase in workflowStore.simulation.phases" :key="phase.label">
        {{ phase.label }}：{{ phase.summary }}
      </p>
    </article>
  </section>
</template>

<script setup lang="ts">
import { useProjectStore } from '@/stores/projectStore'
import { useWorkflowStore } from '@/stores/workflowStore'

const projectStore = useProjectStore()
const workflowStore = useWorkflowStore()

function run() {
  void workflowStore.runSampleSimulation(projectStore.activeProjectId ?? 'project_sample')
}
</script>
