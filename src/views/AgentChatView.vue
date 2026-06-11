<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">智能体</p>
        <h1>项目问答</h1>
      </div>
      <label class="project-picker">
        <span>项目</span>
        <select v-model="selectedProjectId" :disabled="projectStore.loading || asking">
          <option value="">未选择</option>
          <option v-for="project in projectStore.projects" :key="project.id" :value="project.id">
            {{ project.name }}
          </option>
        </select>
      </label>
    </header>

    <form class="agent-panel" @submit.prevent="submitQuestion">
      <label class="question-field">
        <span>问题</span>
        <textarea
          v-model="question"
          rows="4"
          placeholder="基于当前项目资料提问"
          :disabled="asking || !selectedProjectId"
        />
      </label>
      <div class="agent-actions">
        <button type="submit" class="primary-button" :disabled="!canSubmit">
          {{ asking ? "生成中" : "提交" }}
        </button>
        <button type="button" :disabled="!canSave || savingReport" @click="saveReport">
          保存为报告
        </button>
        <button type="button" :disabled="!canSave || savingTask" @click="saveTask">
          保存为任务
        </button>
      </div>
    </form>

    <div v-if="!selectedProjectId" class="empty-state">
      <h2>未选择项目</h2>
      <p>先选择一个项目，再提交问题。</p>
    </div>

    <p v-if="statusMessage" class="status-note">{{ statusMessage }}</p>
    <p v-if="errorMessage" class="status-note error">{{ errorMessage }}</p>
    <p v-if="semanticNotice" class="status-note">{{ semanticNotice }}</p>

    <article v-if="answer" class="empty-state agent-answer">
      <div class="answer-head">
        <div>
          <h2>回答</h2>
          <p v-if="lastResponse" class="answer-meta">
            {{ providerKindLabel(lastResponse.providerKind) }} · {{ lastResponse.model }}
          </p>
        </div>
      </div>
      <p class="answer-text">{{ answer }}</p>
    </article>

    <section v-if="references.length > 0" class="references-list">
      <header>
        <h2>引用来源</h2>
        <span>{{ references.length }} 条</span>
      </header>
      <article
        v-for="reference in references"
        :key="`${reference.source}:${reference.entityType}:${reference.entityId}`"
        class="project-row reference-row"
      >
        <div class="reference-title">
          <strong>{{ reference.title }}</strong>
          <span class="status-chip" :class="reference.source">
            {{ reference.source === "semantic" ? "语义" : "关键词" }}
          </span>
        </div>
        <span>
          {{ entityTypeLabel(reference.entityType) }}
          <template v-if="reference.source === 'semantic'">
            · 相似度 {{ formatScore(reference.score) }}
          </template>
          · {{ reference.snippet || "无摘要" }}
        </span>
      </article>
    </section>
  </section>
</template>

<script setup lang="ts">
import { entityTypeLabel, providerKindLabel } from "@/domain/displayLabels";
import { useAgentChatWorkspace } from "@/composables/useAgentChatWorkspace";

const {
  answer,
  asking,
  canSave,
  canSubmit,
  errorMessage,
  formatScore,
  lastResponse,
  projectStore,
  question,
  references,
  saveReport,
  saveTask,
  savingReport,
  savingTask,
  selectedProjectId,
  semanticNotice,
  statusMessage,
  submitQuestion,
} = useAgentChatWorkspace();
</script>

<style scoped>
.project-picker {
  display: grid;
  gap: 6px;
  min-width: min(280px, 100%);
  color: var(--text-muted);
  font-size: 12px;
}

.project-picker select {
  min-height: 36px;
}

.agent-panel {
  display: grid;
  gap: 12px;
}

.question-field {
  display: grid;
  gap: 8px;
  color: var(--text-muted);
  font-size: 12px;
}

.question-field textarea {
  width: 100%;
  resize: vertical;
}

.agent-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.agent-answer {
  align-items: stretch;
}

.answer-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.answer-meta {
  margin-top: 4px;
  color: var(--text-muted);
  font-size: 12px;
}

.answer-text {
  white-space: pre-wrap;
}

.references-list {
  display: grid;
  gap: 10px;
}

.references-list > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--text-muted);
}

.references-list h2 {
  margin: 0;
  color: var(--text);
  font-size: 16px;
}

.reference-row {
  display: grid;
  gap: 6px;
}

.reference-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.status-chip.keyword {
  color: var(--text-muted);
}

.status-chip.semantic {
  color: var(--accent);
}

@media (max-width: 800px) {
  .view-header {
    align-items: stretch;
  }

  .project-picker {
    width: 100%;
  }
}
</style>
