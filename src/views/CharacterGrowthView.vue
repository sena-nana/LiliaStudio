<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">角色成长</p>
        <h1>成长工作台</h1>
      </div>
      <p v-if="statusMessage" class="status-note" :class="{ error: statusKind === 'error' }">
        {{ statusMessage }}
      </p>
    </header>

    <div v-if="!projectId" class="empty-state">
      <h2>未选择项目</h2>
      <p>先选择一个项目，再查看角色成长记录。</p>
    </div>

    <div v-else-if="libraryStore.loading || workspaceLoading" class="empty-state">
      <h2>正在加载</h2>
      <p>正在读取角色、事件和成长记录。</p>
    </div>

    <div v-else-if="libraryStore.characters.length === 0" class="empty-state">
      <h2>暂无角色</h2>
      <p>先在资料库里创建角色，再补充成长记录。</p>
    </div>

    <div v-else-if="libraryStore.events.length === 0" class="empty-state">
      <h2>暂无事件</h2>
      <p>先创建项目事件，再记录角色受到的影响。</p>
    </div>

    <div v-else class="object-workspace growth-workspace">
      <aside class="object-rail">
        <header>
          <div>
            <h2>角色</h2>
            <p>{{ libraryStore.characters.length }} 个</p>
          </div>
        </header>
        <div class="object-table">
          <button
            v-for="character in sortedCharacters"
            :key="character.id"
            type="button"
            class="collection-row"
            :class="{ active: character.id === selectedCharacterId }"
            @click="selectedCharacterId = character.id"
          >
            <strong>{{ character.name }}</strong>
            <span>{{ character.faction || "未分配阵营" }}</span>
          </button>
        </div>
      </aside>

      <section class="object-main">
        <article class="empty-state compact">
          <header class="growth-panel-header">
            <div>
              <h2>{{ selectedCharacter?.name }}</h2>
              <p>{{ selectedCharacter?.summary || "暂无角色摘要" }}</p>
            </div>
            <div class="chip-list">
              <span class="status-chip ok">已保存记录 {{ selectedRecords.length }}</span>
              <span class="status-chip">来源事件 {{ uniqueEventCount }}</span>
            </div>
          </header>

          <div v-if="traitEntries.length > 0" class="growth-trait-grid" aria-label="当前 trait 状态">
            <article v-for="[traitName, value] in traitEntries" :key="traitName" class="growth-trait-card">
              <strong>{{ traitName }}</strong>
              <span>{{ formatSigned(value) }}</span>
            </article>
          </div>
          <p v-else class="growth-muted">尚无已保存 trait 状态。</p>
        </article>

        <section class="workspace-grid growth-main-grid">
          <article class="empty-state">
            <header class="growth-panel-header">
              <div>
                <h2>来源事件输入</h2>
                <p>选择项目内事件，填写本次成长影响。</p>
              </div>
            </header>

            <form class="editor-form" @submit.prevent="runPreview">
              <label>
                <span class="field-label">来源事件</span>
                <select v-model="draft.sourceEventId" aria-label="来源事件">
                  <option value="">请选择事件</option>
                  <option v-for="event in sortedEvents" :key="event.id" :value="event.id">
                    {{ event.title }} · {{ event.timeLabel || "未定时间" }}
                  </option>
                </select>
              </label>

              <label>
                <span class="field-label">Trait 名称</span>
                <input v-model.trim="draft.traitName" aria-label="Trait 名称" placeholder="例如 responsibility" />
              </label>

              <label>
                <span class="field-label">Delta</span>
                <input v-model="draft.delta" aria-label="Delta" inputmode="decimal" placeholder="例如 0.35" />
              </label>

              <label class="wide">
                <span class="field-label">原因</span>
                <textarea
                  v-model.trim="draft.reason"
                  aria-label="原因"
                  rows="4"
                  placeholder="描述事件如何影响角色 trait"
                />
              </label>

              <div class="object-actions">
                <button type="submit" class="secondary-button" :disabled="previewDisabled">
                  {{ isPreviewing ? "预览中" : "预览变化" }}
                </button>
                <button type="button" class="primary-button" :disabled="saveDisabled" @click="saveRecord">
                  {{ isSaving ? "保存中" : "保存记录" }}
                </button>
              </div>
            </form>
          </article>

          <article class="empty-state">
            <header class="growth-panel-header">
              <div>
                <h2>预览结果</h2>
                <p>基于当前已保存状态叠加本次 delta。</p>
              </div>
            </header>

            <div v-if="previewState" class="view-stack">
              <div class="growth-preview-summary">
                <strong>{{ eventTitle(draft.sourceEventId) }}</strong>
                <span>{{ draft.traitName || "未填写 trait" }} {{ parsedDelta === null ? "" : formatSigned(parsedDelta) }}</span>
              </div>
              <div v-if="previewEntries.length > 0" class="growth-trait-grid" aria-label="预览 trait 状态">
                <article v-for="[traitName, value] in previewEntries" :key="traitName" class="growth-trait-card preview">
                  <strong>{{ traitName }}</strong>
                  <span>{{ formatSigned(value) }}</span>
                </article>
              </div>
              <p class="growth-muted">{{ draft.reason || "尚未填写原因。" }}</p>
            </div>
            <p v-else class="growth-muted">先填写事件、trait、delta 和原因，再执行预览。</p>
          </article>
        </section>

        <article class="empty-state compact">
          <header class="growth-panel-header">
            <div>
              <h2>来源记录</h2>
              <p>按保存顺序展示当前角色的成长来源。</p>
            </div>
          </header>

          <ul v-if="selectedRecords.length > 0" class="inspector-list">
            <li v-for="record in visibleRecords" :key="record.id">
              <article class="growth-record-row">
                <div class="growth-record-header">
                  <strong>{{ eventTitle(record.sourceEventId) }}</strong>
                  <span>{{ record.traitName }} {{ formatSigned(record.delta) }}</span>
                </div>
                <p>{{ record.reason || "未填写原因" }}</p>
                <span>{{ formatRecordMeta(record) }}</span>
              </article>
            </li>
          </ul>
          <p v-else class="growth-muted">尚无成长记录，先预览并保存第一条影响。</p>
        </article>
      </section>

      <aside class="object-inspector">
        <header>
          <div>
            <h2>结果状态</h2>
            <p>{{ statusLabel }}</p>
          </div>
        </header>

        <section>
          <h3>当前角色</h3>
          <p>{{ selectedCharacter?.name }}</p>
          <p>{{ selectedCharacter?.faction || "未分配阵营" }}</p>
        </section>

        <section>
          <h3>输入检查</h3>
          <div class="chip-list">
            <span class="status-chip" :class="{ ok: Boolean(draft.sourceEventId) }">事件</span>
            <span class="status-chip" :class="{ ok: Boolean(draft.traitName) }">Trait</span>
            <span class="status-chip" :class="{ ok: parsedDelta !== null }">Delta</span>
            <span class="status-chip" :class="{ ok: Boolean(draft.reason) }">原因</span>
          </div>
        </section>

        <section>
          <h3>提示</h3>
          <p>第一版不会校验该事件是否真的包含当前角色参与记录。</p>
        </section>
      </aside>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { createCharacterTraitDeltaRecord, loadCharacterGrowthWorkspace, previewTraitDelta } from "@/api/workflows";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import type {
  CharacterGrowthWorkspaceSnapshot,
  CharacterTraitDeltaRecord,
  CharacterTraitState,
} from "@/types/workflows";

type StatusKind = "idle" | "previewing" | "previewed" | "saving" | "saved" | "error";

const route = useRoute();
const projectStore = useProjectStore();
const libraryStore = useLibraryStore();

const workspaceLoading = ref(false);
const previewState = ref<CharacterTraitState | null>(null);
const workspace = ref<CharacterGrowthWorkspaceSnapshot | null>(null);
const selectedCharacterId = ref("");
const statusKind = ref<StatusKind>("idle");
const statusMessage = ref("");
const draft = reactive({
  sourceEventId: "",
  traitName: "",
  delta: "",
  reason: "",
});

const emptyState: CharacterTraitState = {
  values: {},
  sources: [],
};

const projectId = computed(() => {
  const value = route.params.projectId;
  return typeof value === "string" && value.length > 0 ? value : projectStore.activeProjectId;
});

const sortedCharacters = computed(() =>
  [...libraryStore.characters].sort((left, right) => left.name.localeCompare(right.name, "zh-Hans-CN")),
);
const sortedEvents = computed(() => [...libraryStore.events].sort((left, right) => left.sortKey - right.sortKey));
const selectedCharacter = computed(
  () => sortedCharacters.value.find((character) => character.id === selectedCharacterId.value) ?? null,
);
const selectedState = computed<CharacterTraitState>(() => {
  if (!selectedCharacterId.value) return emptyState;
  return workspace.value?.states[selectedCharacterId.value] ?? emptyState;
});
const selectedRecords = computed(() =>
  (workspace.value?.records ?? []).filter((record) => record.characterId === selectedCharacterId.value),
);
const visibleRecords = computed(() => [...selectedRecords.value].reverse());
const traitEntries = computed(() =>
  Object.entries(selectedState.value.values).sort((left, right) => left[0].localeCompare(right[0], "en")),
);
const previewEntries = computed(() =>
  Object.entries(previewState.value?.values ?? {}).sort((left, right) => left[0].localeCompare(right[0], "en")),
);
const parsedDelta = computed<number | null>(() => {
  if (draft.delta.trim().length === 0) return null;
  const value = Number(draft.delta);
  return Number.isFinite(value) ? value : null;
});
const isPreviewing = computed(() => statusKind.value === "previewing");
const isSaving = computed(() => statusKind.value === "saving");
const previewDisabled = computed(
  () =>
    isPreviewing.value ||
    isSaving.value ||
    !selectedCharacterId.value ||
    !draft.sourceEventId ||
    !draft.traitName ||
    parsedDelta.value === null ||
    !draft.reason,
);
const saveDisabled = computed(() => isSaving.value || isPreviewing.value || previewState.value === null);
const uniqueEventCount = computed(() => new Set(selectedRecords.value.map((record) => record.sourceEventId)).size);
const statusLabel = computed(() => {
  switch (statusKind.value) {
    case "previewing":
      return "预览中";
    case "previewed":
      return "已生成预览";
    case "saving":
      return "保存中";
    case "saved":
      return "已保存";
    case "error":
      return "失败";
    default:
      return "空闲";
  }
});

onMounted(async () => {
  if (projectStore.projects.length === 0) {
    await projectStore.loadProjects();
  }
  await loadView();
});

watch(projectId, async () => {
  resetForm();
  await loadView();
});

watch(sortedCharacters, (characters) => {
  if (!selectedCharacterId.value || !characters.some((character) => character.id === selectedCharacterId.value)) {
    selectedCharacterId.value = characters[0]?.id ?? "";
  }
});

watch(selectedCharacterId, () => {
  previewState.value = null;
  if (statusKind.value !== "error") {
    statusKind.value = "idle";
    statusMessage.value = "";
  }
});

async function loadView() {
  if (!projectId.value) {
    workspace.value = null;
    return;
  }

  workspaceLoading.value = true;
  statusKind.value = "idle";
  statusMessage.value = "";
  try {
    await libraryStore.loadProject(projectId.value);
    workspace.value = await loadCharacterGrowthWorkspace(projectId.value);
    selectedCharacterId.value = sortedCharacters.value[0]?.id ?? "";
  } catch (error) {
    statusKind.value = "error";
    statusMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    workspaceLoading.value = false;
  }
}

async function runPreview() {
  if (previewDisabled.value || parsedDelta.value === null) return;
  statusKind.value = "previewing";
  statusMessage.value = "正在计算预览结果";
  try {
    previewState.value = await previewTraitDelta(selectedState.value, {
      sourceEventId: draft.sourceEventId,
      traitName: draft.traitName,
      delta: parsedDelta.value,
      reason: draft.reason,
    });
    statusKind.value = "previewed";
    statusMessage.value = "已更新预览结果";
  } catch (error) {
    previewState.value = null;
    statusKind.value = "error";
    statusMessage.value = error instanceof Error ? error.message : String(error);
  }
}

async function saveRecord() {
  if (!projectId.value || !selectedCharacterId.value || parsedDelta.value === null || !previewState.value) return;
  statusKind.value = "saving";
  statusMessage.value = "正在保存成长记录";
  try {
    await createCharacterTraitDeltaRecord({
      projectId: projectId.value,
      characterId: selectedCharacterId.value,
      sourceEventId: draft.sourceEventId,
      traitName: draft.traitName,
      delta: parsedDelta.value,
      reason: draft.reason,
    });
    workspace.value = await loadCharacterGrowthWorkspace(projectId.value);
    previewState.value = null;
    statusKind.value = "saved";
    statusMessage.value = "成长记录已保存";
    resetForm(false);
  } catch (error) {
    statusKind.value = "error";
    statusMessage.value = error instanceof Error ? error.message : String(error);
  }
}

function resetForm(clearStatus = true) {
  draft.sourceEventId = "";
  draft.traitName = "";
  draft.delta = "";
  draft.reason = "";
  previewState.value = null;
  if (clearStatus) {
    statusKind.value = "idle";
    statusMessage.value = "";
  }
}

function formatSigned(value: number): string {
  return `${value >= 0 ? "+" : ""}${value.toFixed(2)}`;
}

function eventTitle(eventId: string): string {
  return libraryStore.events.find((event) => event.id === eventId)?.title ?? eventId;
}

function formatRecordMeta(record: CharacterTraitDeltaRecord): string {
  const event = libraryStore.events.find((item) => item.id === record.sourceEventId);
  return `${event?.timeLabel || "未定时间"} · ${record.createdAt}`;
}
</script>

<style scoped>
.growth-workspace {
  grid-template-columns: 180px minmax(0, 1fr) minmax(240px, 280px);
}

.growth-main-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.growth-panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.growth-trait-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 8px;
}

.growth-trait-card {
  display: grid;
  gap: 4px;
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  background: var(--bg-subtle);
  padding: 10px;
}

.growth-trait-card.preview {
  border-color: var(--accent);
  background: var(--accent-soft);
}

.growth-trait-card span,
.growth-preview-summary span,
.growth-record-row span,
.growth-muted {
  color: var(--text-muted);
}

.growth-preview-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.growth-record-row {
  display: grid;
  gap: 4px;
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  background: var(--bg-subtle);
  padding: 10px;
}

.growth-record-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

@media (max-width: 1100px) {
  .growth-workspace,
  .growth-main-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
