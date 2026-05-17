<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">任务</p>
        <h1>AI 后台队列</h1>
      </div>
      <button type="button" class="primary-button" :disabled="jobStore.loading" @click="refreshJobs">
        刷新
      </button>
    </header>

    <section class="settings-list">
      <article v-for="job in jobStore.jobs" :key="job.id" class="job-card">
        <header class="job-card-header">
          <div>
            <h2>{{ job.jobType }}</h2>
            <p>{{ job.providerKind }} · {{ job.status }}</p>
          </div>
          <div class="job-card-actions">
            <button type="button" @click="loadLogs(job.id)">日志</button>
            <button type="button" @click="cancel(job.id)">取消</button>
            <button type="button" @click="retry(job.id)">重试</button>
          </div>
        </header>
        <p class="job-summary">{{ job.inputSummary }}</p>
        <p v-if="job.errorMessage" class="job-error">{{ job.errorMessage }}</p>
        <ul v-if="jobStore.logsByJobId[job.id]?.length" class="job-log-list">
          <li v-for="log in jobStore.logsByJobId[job.id]" :key="log.id">
            <strong>{{ log.level }}</strong>
            <span>{{ log.message }}</span>
          </li>
        </ul>
      </article>
    </section>
  </section>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useJobStore } from '@/stores/jobStore'

const jobStore = useJobStore()

onMounted(() => {
  void refreshJobs()
})

async function refreshJobs() {
  jobStore.loading = true
  try {
    await jobStore.loadJobs()
    await jobStore.refreshActiveJob()
  } finally {
    jobStore.loading = false
  }
}

async function loadLogs(jobId: string) {
  await jobStore.loadLogs(jobId)
}

async function cancel(jobId: string) {
  await jobStore.cancelJob(jobId)
}

async function retry(jobId: string) {
  await jobStore.retryJob(jobId)
}
</script>
