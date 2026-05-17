<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">设置</p>
        <h1>本机配置</h1>
      </div>
      <button type="button" class="primary-button" :disabled="aiStore.loading" @click="saveSettings">
        保存
      </button>
    </header>

    <div class="settings-list">
      <section class="provider-form" v-for="provider in providerForms" :key="provider.kind">
        <header>
          <div>
            <h2>{{ providerLabel(provider.kind) }}</h2>
            <p>{{ provider.enabled ? '已启用' : '未启用' }}</p>
          </div>
          <label class="toggle-row">
            <input v-model="provider.enabled" type="checkbox" />
            启用
          </label>
        </header>

        <div v-if="provider.kind === 'openAiCompatible'" class="settings-grid">
          <label>
            <span>Base URL</span>
            <input v-model="provider.baseUrl" placeholder="https://api.example.com/v1" />
          </label>
          <label>
            <span>Chat model</span>
            <input v-model="provider.chatModel" placeholder="gpt-4.1" />
          </label>
          <label>
            <span>Embedding model</span>
            <input v-model="provider.embeddingModel" placeholder="text-embedding-3-small" />
          </label>
          <label>
            <span>API Key</span>
            <input
              v-model="provider.apiKey"
              type="password"
              :placeholder="apiKeyPlaceholder(provider.kind)"
              autocomplete="new-password"
            />
          </label>
          <label class="toggle-row">
            <input v-model="provider.clearApiKey" type="checkbox" />
            清除已保存密钥
          </label>
          <div class="settings-actions">
            <button type="button" class="primary-button" :disabled="aiStore.loading" @click="testOpenAi">
              测试 Provider
            </button>
            <span v-if="aiStore.openAiProviderTest" :class="['test-result', aiStore.openAiProviderTest.ok ? 'ok' : 'error']">
              {{ aiStore.openAiProviderTest.message }}
            </span>
          </div>
        </div>

        <label v-else class="stacked-field">
          <span>命令模板</span>
          <textarea v-model="provider.commandTemplate" rows="3" />
        </label>
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

    <p v-if="statusMessage" class="status-note">{{ statusMessage }}</p>
  </section>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useAiStore } from '@/stores/aiStore'
import type { AiProviderKind, AiProviderSettingsDraft } from '@/types/ai'

const aiStore = useAiStore()
const providerForms = reactive<AiProviderSettingsDraft[]>([])
const statusMessage = ref('')

onMounted(() => {
  void loadSettings()
})

async function loadSettings() {
  aiStore.loading = true
  try {
    await aiStore.loadProviderSettings()
    providerForms.splice(
      0,
      providerForms.length,
      ...aiStore.providerSettings.map((provider) => ({
        kind: provider.kind,
        baseUrl: provider.baseUrl,
        apiKey: null,
        clearApiKey: false,
        chatModel: provider.chatModel,
        embeddingModel: provider.embeddingModel,
        commandTemplate: provider.commandTemplate,
        enabled: provider.enabled,
      })),
    )
  } finally {
    aiStore.loading = false
  }
}

async function saveSettings() {
  aiStore.loading = true
  statusMessage.value = ''
  try {
    await aiStore.saveProviderSettings(
      providerForms.map((provider) => ({
        ...provider,
        apiKey: provider.apiKey?.trim() ? provider.apiKey.trim() : null,
      })),
    )
    await loadSettings()
    statusMessage.value = '设置已保存'
  } catch (error) {
    statusMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    aiStore.loading = false
  }
}

function loadPrompts() {
  void aiStore.loadPromptsAndJobs()
}

async function testOpenAi() {
  statusMessage.value = ''
  aiStore.loading = true
  try {
    await aiStore.testOpenAiProvider()
  } catch (error) {
    statusMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    aiStore.loading = false
  }
}

function providerLabel(kind: AiProviderKind) {
  return {
    openAiCompatible: 'OpenAI-compatible',
    codexCli: 'Codex CLI',
    claudeCli: 'Claude CLI',
  }[kind]
}

function apiKeyPlaceholder(kind: AiProviderKind) {
  const provider = aiStore.providerSettings.find((item) => item.kind === kind)
  return provider?.apiKeyPreview ? `已保存：${provider.apiKeyPreview}` : '未保存'
}
</script>
