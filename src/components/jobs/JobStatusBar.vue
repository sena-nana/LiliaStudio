<template>
  <RouterLink class="job-status-link" to="/jobs">
    <span v-if="jobStore.activeJob">
      运行中：{{ jobStore.activeJob.jobType }} · {{ jobStore.activeJob.providerKind }}
    </span>
    <span v-else>任务空闲</span>
  </RouterLink>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import { useJobStore } from '@/stores/jobStore'

const jobStore = useJobStore()
let timer: number | undefined

onMounted(() => {
  void refreshActiveJob()
  timer = window.setInterval(() => {
    void refreshActiveJob()
  }, 5000)
})

onBeforeUnmount(() => {
  if (timer !== undefined) {
    window.clearInterval(timer)
  }
})

async function refreshActiveJob() {
  try {
    await jobStore.refreshActiveJob()
  } catch {
    jobStore.activeJob = null
  }
}
</script>
