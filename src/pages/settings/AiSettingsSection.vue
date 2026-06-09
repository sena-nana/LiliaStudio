<template>
  <div class="card provider-form">
    <header class="settings-section-head">
      <div>
        <h2>AI Settings</h2>
        <p class="muted">Configure OpenAI-compatible provider, models, and secrets.</p>
      </div>
      <button type="button" class="primary" :disabled="aiStore.loading" @click="saveSettings">
        Save
      </button>
    </header>

    <section v-for="provider in openAiForms" :key="provider.kind" class="settings-block">
      <header class="settings-row settings-row--top">
        <div class="settings-row__label">
          <div>{{ providerLabel(provider.kind) }}</div>
          <div class="settings-row__hint">{{ provider.enabled ? "Enabled" : "Disabled" }}</div>
        </div>
        <label class="toggle-row">
          <input v-model="provider.enabled" type="checkbox" />
          Enabled
        </label>
      </header>

      <div class="settings-grid">
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
          Clear saved key
        </label>
      </div>

      <div class="settings-actions">
        <button type="button" class="primary" :disabled="aiStore.loading" @click="testOpenAi">
          Test
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
    statusMessage.value = "Settings saved";
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

function providerLabel(kind: AiProviderKind) {
  return {
    openAiCompatible: "OpenAI-compatible",
    codexCli: "Codex CLI",
    claudeCli: "Claude CLI",
  }[kind];
}

function apiKeyPlaceholder(kind: AiProviderKind) {
  const provider = aiStore.providerSettings.find((item) => item.kind === kind);
  return provider?.apiKeyPreview ? `Saved: ${provider.apiKeyPreview}` : "Not saved";
}
</script>

<style scoped>
@import "./settings-section.css";
</style>
