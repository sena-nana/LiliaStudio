<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">审计</p>
        <h1>逻辑一致性报告</h1>
      </div>
      <button type="button" class="primary-button" @click="workflowStore.runSampleAudit">运行样例审计</button>
    </header>

    <article v-for="conflict in workflowStore.conflicts" :key="conflict.message" class="empty-state">
      <h2>{{ conflict.conflictType }}</h2>
      <p>{{ conflict.message }}</p>
    </article>
    <article v-for="repair in workflowStore.repairs" :key="repair.title" class="empty-state">
      <h2>{{ repair.title }}</h2>
      <p>{{ repair.description }} · {{ repair.impact }}</p>
    </article>
  </section>
</template>

<script setup lang="ts">
import { useWorkflowStore } from '@/stores/workflowStore'

const workflowStore = useWorkflowStore()
</script>
