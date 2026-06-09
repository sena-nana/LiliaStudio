<template>
  <div class="card provider-form">
    <header class="settings-section-head">
      <div>
        <h2>人工智能设置</h2>
        <p class="muted">配置 OpenAI 兼容接口、模型和密钥。</p>
      </div>
      <button type="button" class="primary" :disabled="aiStore.loading" @click="saveSettings">
        保存
      </button>
    </header>

    <section v-for="provider in openAiForms" :key="provider.kind" class="settings-block">
      <header class="settings-row settings-row--top">
        <div class="settings-row__label">
          <div>{{ providerKindLabel(provider.kind) }}</div>
          <div class="settings-row__hint">{{ provider.enabled ? "已启用" : "已停用" }}</div>
        </div>
        <label class="toggle-row">
          <input v-model="provider.enabled" type="checkbox" />
          启用
        </label>
      </header>

      <div class="settings-grid">
        <label>
          <span>基础地址</span>
          <input v-model="provider.baseUrl" placeholder="https://api.example.com/v1" />
        </label>
        <label>
          <span>对话模型</span>
          <input v-model="provider.chatModel" placeholder="gpt-4.1" />
        </label>
        <label>
          <span>嵌入模型</span>
          <input v-model="provider.embeddingModel" placeholder="text-embedding-3-small" />
        </label>
        <label>
          <span>接口密钥</span>
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
      </div>

      <div class="settings-actions">
        <button type="button" class="primary" :disabled="aiStore.loading" @click="testOpenAi">
          测试
        </button>
        <span
          v-if="aiStore.openAiProviderTest"
          :class="['test-result', aiStore.openAiProviderTest.ok ? 'ok' : 'error']"
        >
          {{ aiStore.openAiProviderTest.message }}
        </span>
      </div>
    </section>

    <p v-if="statusMessage" class="status-note">{{ statusMessage }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { providerKindLabel } from "@/domain/displayLabels";
import { useAiStore } from "@/stores/aiStore";
import type { AiProviderKind, AiProviderSettingsDraft } from "@/types/ai";

const aiStore = useAiStore();
const providerForms = reactive<AiProviderSettingsDraft[]>([]);
const statusMessage = ref("");
const openAiForms = computed(() => providerForms.filter((provider) => provider.kind === "openAiCompatible"));

onMounted(() => {
  void loadSettings();
});

async function loadSettings() {
  aiStore.loading = true;
  try {
    await aiStore.loadProviderSettings();
    providerForms.splice(0, providerForms.length, ...toDrafts());
  } finally {
    aiStore.loading = false;
  }
}

function toDrafts() {
  return aiStore.providerSettings.map((provider) => ({
    kind: provider.kind,
    baseUrl: provider.baseUrl,
    apiKey: null,
    clearApiKey: false,
    chatModel: provider.chatModel,
    embeddingModel: provider.embeddingModel,
    commandTemplate: provider.commandTemplate,
    enabled: provider.enabled,
  }));
}

async function saveSettings() {
  aiStore.loading = true;
  statusMessage.value = "";
  try {
    await aiStore.saveProviderSettings(normalizeDrafts());
    await loadSettings();
    statusMessage.value = "设置已保存";
  } catch (error) {
    statusMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    aiStore.loading = false;
  }
}

function normalizeDrafts() {
  return providerForms.map((provider) => ({
    ...provider,
    apiKey: provider.apiKey?.trim() ? provider.apiKey.trim() : null,
  }));
}

async function testOpenAi() {
  statusMessage.value = "";
  aiStore.loading = true;
  try {
    await aiStore.testOpenAiProvider();
  } catch (error) {
    statusMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    aiStore.loading = false;
  }
}

function apiKeyPlaceholder(kind: AiProviderKind) {
  const provider = aiStore.providerSettings.find((item) => item.kind === kind);
  return provider?.apiKeyPreview ? `已保存：${provider.apiKeyPreview}` : "未保存";
}
</script>

<style scoped>
@import "./settings-section.css";
</style>
