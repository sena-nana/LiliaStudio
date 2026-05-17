<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">模板</p>
        <h1>Prompt 模板</h1>
      </div>
      <div class="template-editor-actions">
        <button type="button" class="secondary-button" @click="loadTemplates">
          刷新
        </button>
        <button type="button" class="primary-button" @click="resetBuiltins">
          重置内置
        </button>
      </div>
    </header>

    <section class="prompt-template-layout">
      <aside class="prompt-template-list" aria-label="Prompt 模板列表">
        <button
          v-for="template in promptStore.templates"
          :key="template.id"
          type="button"
          :class="{ active: promptStore.selectedTemplate?.id === template.id }"
          @click="selectTemplate(template.id)"
        >
          <strong>{{ template.name }}</strong>
          <span
            >{{ purposeLabel(template.purpose) }} ·
            {{ template.builtIn ? "内置" : "自定义" }}</span
          >
        </button>
      </aside>

      <section class="prompt-template-editor">
        <header>
          <div>
            <h2>{{ form.name || "未选择模板" }}</h2>
            <p>{{ selectedTemplate?.builtIn ? "内置模板" : "自定义模板" }}</p>
          </div>
          <div class="template-editor-actions">
            <button
              type="button"
              class="secondary-button"
              :disabled="!selectedTemplate"
              @click="copySelected"
            >
              复制
            </button>
            <button
              type="button"
              class="secondary-button"
              @click="previewTemplate"
            >
              预览
            </button>
            <button
              type="button"
              class="primary-button"
              :disabled="!canSave"
              @click="saveTemplate"
            >
              保存
            </button>
          </div>
        </header>

        <div class="settings-grid">
          <label>
            名称
            <input v-model="form.name" :disabled="isReadOnly" />
          </label>
          <label>
            用途
            <input v-model="form.purpose" :disabled="isReadOnly" />
          </label>
        </div>

        <label class="stacked-field">
          模板
          <textarea v-model="form.template" :readonly="isReadOnly" rows="14" />
        </label>
      </section>

      <aside class="prompt-preview-panel" aria-label="Prompt 预览">
        <section class="variable-list">
          <h2>变量</h2>
          <label
            v-for="variable in visibleVariables"
            :key="variable.name"
            class="variable-field"
          >
            <strong>{{ variable.name }}</strong>
            <input
              v-model="variableValues[variable.name]"
              :placeholder="variable.example"
            />
          </label>
        </section>

        <section class="prompt-preview-output">
          <header>
            <h2>预览</h2>
            <button
              type="button"
              class="secondary-button"
              @click="fillExamples"
            >
              填充示例
            </button>
          </header>
          <pre>{{ promptStore.previewResult?.prompt || form.template }}</pre>
          <p
            v-if="promptStore.previewResult?.missingVariables.length"
            class="missing-variables"
          >
            缺少变量：{{
              promptStore.previewResult.missingVariables.join("、")
            }}
          </p>
        </section>
      </aside>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, watch } from "vue";
import { usePromptTemplateStore } from "@/stores/promptTemplateStore";
import type { PromptTemplate, PromptTemplateVariable } from "@/types/ai";

const promptStore = usePromptTemplateStore();
const form = reactive({
  id: null as string | null,
  name: "",
  purpose: "",
  template: "",
});
const variableValues = reactive<Record<string, string>>({});

const selectedTemplate = computed(() => promptStore.selectedTemplate);
const isReadOnly = computed(() => selectedTemplate.value?.builtIn ?? true);
const canSave = computed(() => Boolean(form.id && !isReadOnly.value));
const visibleVariables = computed(() =>
  describeVisibleVariables(form.template, selectedTemplate.value),
);

onMounted(() => {
  void loadTemplates();
});

watch(
  selectedTemplate,
  (template) => {
    syncForm(template);
  },
  { immediate: true },
);

watch(
  visibleVariables,
  (variables) => {
    for (const variable of variables) {
      if (variableValues[variable.name] === undefined) {
        variableValues[variable.name] = "";
      }
    }
  },
  { immediate: true },
);

async function loadTemplates() {
  promptStore.loading = true;
  try {
    await promptStore.loadTemplates();
  } finally {
    promptStore.loading = false;
  }
}

function selectTemplate(templateId: string) {
  promptStore.selectTemplate(templateId);
}

async function copySelected() {
  if (!selectedTemplate.value) {
    return;
  }
  await promptStore.copyTemplate(selectedTemplate.value.id);
}

async function saveTemplate() {
  if (!canSave.value) {
    return;
  }
  await promptStore.saveTemplate({
    id: form.id,
    name: form.name,
    purpose: form.purpose,
    template: form.template,
  });
}

async function resetBuiltins() {
  await promptStore.resetBuiltins();
  if (!promptStore.selectedTemplate && promptStore.templates[0]) {
    promptStore.selectTemplate(promptStore.templates[0].id);
  }
}

async function previewTemplate() {
  await promptStore.preview(
    form.template,
    visibleVariables.value.map((variable) => ({
      name: variable.name,
      value: variableValues[variable.name] ?? "",
    })),
  );
}

function fillExamples() {
  for (const variable of visibleVariables.value) {
    if (variable.example) {
      variableValues[variable.name] = variable.example;
    }
  }
}

function syncForm(template: PromptTemplate | null) {
  form.id = template?.id ?? null;
  form.name = template?.name ?? "";
  form.purpose = template?.purpose ?? "";
  form.template = template?.template ?? "";
  promptStore.previewResult = null;
}

function describeVisibleVariables(
  template: string,
  selected: PromptTemplate | null,
): PromptTemplateVariable[] {
  const known = new Map(
    (selected?.variables ?? []).map((variable) => [variable.name, variable]),
  );
  return extractVariableNames(template).map((name) => {
    return (
      known.get(name) ?? {
        name,
        description: "",
        example: "",
      }
    );
  });
}

function extractVariableNames(template: string) {
  const names: string[] = [];
  const pattern = /\{\{\s*([A-Za-z0-9_]+)\s*\}\}/g;
  let match = pattern.exec(template);
  while (match) {
    if (!names.includes(match[1])) {
      names.push(match[1]);
    }
    match = pattern.exec(template);
  }
  return names;
}

function purposeLabel(purpose: string) {
  const labels: Record<string, string> = {
    logic_audit: "逻辑审计",
    completion_questions: "补完问题",
    character_analysis: "角色分析",
    behavior_audit: "行为校验",
    simulation: "叙事模拟",
  };
  return labels[purpose] ?? purpose;
}
</script>
