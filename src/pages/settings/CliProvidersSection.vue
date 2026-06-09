<template>
  <div class="card provider-form">
    <header class="settings-section-head">
      <div>
        <h2>命令行提供方</h2>
        <p class="muted">配置 Codex 命令行和 Claude 命令行的命令模板。</p>
      </div>
      <button type="button" class="primary" :disabled="aiStore.loading" @click="saveSettings">
        保存
      </button>
    </header>

    <section v-for="provider in cliForms" :key="provider.kind" class="settings-block">
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

      <label class="stacked-field">
        <span>命令模板</span>
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
          测试
        </button>
        <button
          v-if="provider.kind === 'claudeCli'"
          type="button"
          class="primary"
          :disabled="aiStore.loading"
          @click="testClaudeCli"
        >
          测试
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
import { providerKindLabel } from "@/domain/displayLabels";
import { useAiStore } from "@/stores/aiStore";
import type { AiProviderSettingsDraft } from "@/types/ai";

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

</script>

<style scoped>
@import "./settings-section.css";
</style>
