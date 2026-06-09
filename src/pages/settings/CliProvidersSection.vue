<template>
  <div class="card provider-form">
    <header class="settings-section-head">
      <div>
        <h2>CLI Provider</h2>
        <p class="muted">Configure Codex CLI and Claude CLI command templates.</p>
      </div>
      <button type="button" class="primary" :disabled="aiStore.loading" @click="saveSettings">
        Save
      </button>
    </header>

    <section v-for="provider in cliForms" :key="provider.kind" class="settings-block">
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

      <label class="stacked-field">
        <span>Command template</span>
        <textarea v-model="provider.commandTemplate" rows="3" />
      </label>

      <div class="settings-actions">
        <button
          v-if="provider.kind === 'codexCli'"
          type="button"
          class="primary"
          :disabled="aiStore.loading"
          @click="testCodexCli"
        >
          Test
        </button>
        <button
          v-if="provider.kind === 'claudeCli'"
          type="button"
          class="primary"
          :disabled="aiStore.loading"
          @click="testClaudeCli"
        >
          Test
        </button>
        <span v-if="provider.kind === 'codexCli' && aiStore.codexCliProviderTest" :class="['test-result', aiStore.codexCliProviderTest.ok ? 'ok' : 'error']">
          {{ aiStore.codexCliProviderTest.message }}
        </span>
        <span v-if="provider.kind === 'claudeCli' && aiStore.claudeCliProviderTest" :class="['test-result', aiStore.claudeCliProviderTest.ok ? 'ok' : 'error']">
          {{ aiStore.claudeCliProviderTest.message }}
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
const cliForms = computed(() => providerForms.filter((provider) => provider.kind !== "openAiCompatible"));

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

async function testCodexCli() {
  statusMessage.value = "";
  aiStore.loading = true;
  try {
    await aiStore.testCodexCliProvider();
  } catch (error) {
    statusMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    aiStore.loading = false;
  }
}

async function testClaudeCli() {
  statusMessage.value = "";
  aiStore.loading = true;
  try {
    await aiStore.testClaudeCliProvider();
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
</script>

<style scoped>
@import "./settings-section.css";
</style>
