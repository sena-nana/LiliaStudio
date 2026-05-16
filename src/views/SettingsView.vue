<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">设置</p>
        <h1>本机配置</h1>
      </div>
    </header>
    <div class="settings-list">
      <section>
        <h2>AI Provider</h2>
        <p>OpenAI-compatible API、Claude CLI 和 Codex CLI 都作为可选能力；未配置时本地资料库照常可用。</p>
        <button type="button" class="primary-button" @click="loadDefaults">加载默认 CLI 模板</button>
        <ul class="provider-list">
          <li v-for="provider in aiStore.providers" :key="provider.kind">
            <strong>{{ provider.kind }}</strong>
            <span>{{ provider.commandTemplate || provider.baseUrl || '未配置' }}</span>
          </li>
        </ul>
      </section>
      <section>
        <h2>Prompt 模板</h2>
        <button type="button" class="primary-button" @click="loadPrompts">加载模板与任务</button>
        <ul class="provider-list">
          <li v-for="prompt in aiStore.prompts" :key="prompt.id">
            <strong>{{ prompt.name }}</strong>
            <span>{{ prompt.purpose }}</span>
          </li>
        </ul>
      </section>
      <section>
        <h2>AI 任务</h2>
        <ul class="provider-list">
          <li v-for="job in aiStore.jobs" :key="job.id">
            <strong>{{ job.jobType }}</strong>
            <span>{{ job.status }} · {{ job.providerKind }}</span>
          </li>
        </ul>
      </section>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useAiStore } from '@/stores/aiStore'

const aiStore = useAiStore()

function loadDefaults() {
  void aiStore.loadDefaults()
}

function loadPrompts() {
  void aiStore.loadPromptsAndJobs()
}
</script>
