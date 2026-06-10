<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">对象工作台</p>
        <h1>{{ projectTitle }}</h1>
      </div>
      <p v-if="libraryStore.error" class="status-note error">{{ libraryStore.error }}</p>
    </header>

    <div v-if="!projectId" class="empty-state">
      <h2>未选择项目</h2>
    </div>

    <div v-else class="object-workspace">
      <aside class="object-rail">
        <header>
          <h2>对象</h2>
        </header>
        <button
          v-for="collection in collections"
          :key="collection.kind"
          type="button"
          class="collection-row"
          :class="{ active: activeCollection === collection.kind }"
          @click="activeCollection = collection.kind"
        >
          <span>{{ collection.label }}</span>
          <strong>{{ collection.count }}</strong>
        </button>
      </aside>

      <main class="object-main">
        <section class="object-list-panel">
          <header>
            <div>
              <h2>{{ activeCollectionLabel }}</h2>
              <p>{{ objectRows.length }} 个对象</p>
            </div>
            <div class="object-actions">
              <EntryTemplatePanel v-if="activeCollection === 'entry'" v-model="entryType" />
              <button
                type="button"
                class="secondary-button"
                :disabled="activeCollection === 'relation' && entityOptions.length < 2"
                @click="createRecord(activeCollection)"
              >
                新增
              </button>
            </div>
          </header>

          <div class="object-table">
            <button
              v-for="row in objectRows"
              :key="`${row.kind}:${row.id}`"
              type="button"
              class="object-table-row"
              :class="{ active: selected.kind === row.kind && selected.id === row.id }"
              @click="selectRecord(row.kind, row.id)"
            >
              <span class="object-title">{{ row.title }}</span>
              <span>{{ row.typeLabel }}</span>
              <span>{{ row.status }}</span>
              <span>{{ row.relationCount }} 关联</span>
              <span>{{ row.updatedAt || "未记录" }}</span>
            </button>
          </div>
        </section>

        <LibraryEditorShell
          :title="editorTitle"
          :has-selection="Boolean(selected.id)"
          @delete="deleteSelected"
          @save="saveSelected"
        >
          <div v-if="selected.kind === 'entry'" class="editor-form">
            <label>标题<input v-model="entryForm.title" /></label>
            <label>类型<input v-model="entryForm.entryType" /></label>
            <label>状态<input v-model="entryForm.status" /></label>
            <label>标签<input v-model="entryTagsText" placeholder="标签" /></label>
            <div class="wide rich-text-field">
              <span class="field-label">摘要</span>
              <Suspense>
                <RichTextEditor v-model="entryForm.summary" />
                <template #fallback><div class="rich-text-loading">编辑器加载中</div></template>
              </Suspense>
            </div>
            <div class="wide rich-text-field">
              <span class="field-label">正文</span>
              <Suspense>
                <RichTextEditor v-model="entryForm.body" />
                <template #fallback><div class="rich-text-loading">编辑器加载中</div></template>
              </Suspense>
            </div>
          </div>

          <div v-else-if="selected.kind === 'character'" class="editor-form">
            <label>姓名<input v-model="characterForm.name" /></label>
            <label>阵营<input v-model="characterForm.faction" /></label>
            <label>别名<input v-model="characterAliasesText" placeholder="别名" /></label>
            <label>标签<input v-model="characterTagsText" placeholder="标签" /></label>
            <label class="wide">摘要<textarea v-model="characterForm.summary" rows="3" /></label>
            <label>外貌<textarea v-model="characterForm.appearance" rows="4" /></label>
            <label>目标<textarea v-model="characterForm.goals" rows="4" /></label>
            <label>动机<textarea v-model="characterForm.motivations" rows="4" /></label>
            <label>恐惧<textarea v-model="characterForm.fears" rows="4" /></label>
          </div>

          <div v-else-if="selected.kind === 'event'" class="editor-form">
            <label>标题<input v-model="eventForm.title" /></label>
            <label>时间<input v-model="eventForm.timeLabel" /></label>
            <label>排序<input v-model.number="eventForm.sortKey" type="number" /></label>
            <label>地点<input v-model="eventForm.location" /></label>
            <label>开始<input v-model="eventForm.startLabel" /></label>
            <label>结束<input v-model="eventForm.endLabel" /></label>
            <label>重要度<input v-model.number="eventForm.importance" min="0" max="10" type="number" /></label>
            <label>标签<input v-model="eventTagsText" placeholder="标签" /></label>
            <label class="wide">描述<textarea v-model="eventForm.description" rows="4" /></label>
            <label class="wide">结果<textarea v-model="eventForm.outcome" rows="4" /></label>
          </div>

          <div v-else-if="selected.kind === 'axiom'" class="editor-form">
            <label>主体<input v-model="axiomForm.subject" /></label>
            <label>谓词<input v-model="axiomForm.predicate" /></label>
            <label>对象<input v-model="axiomForm.object" /></label>
            <label>确定性<input v-model.number="axiomForm.certainty" min="0" max="1" step="0.05" type="number" /></label>
            <label>时间范围<input v-model="axiomForm.scopeTime" /></label>
            <label>地点范围<input v-model="axiomForm.scopeLocation" /></label>
            <label>来源类型<input v-model="sourceEntityTypeText" /></label>
            <label>来源标识<input v-model="sourceEntityIdText" /></label>
            <label>标签<input v-model="axiomTagsText" placeholder="标签" /></label>
            <label class="wide">自然语言<textarea v-model="axiomForm.naturalLanguage" rows="5" /></label>
          </div>

          <div v-else-if="selected.kind === 'relation'" class="editor-form">
            <label>
              来源
              <select v-model="relationSourceKey">
                <option v-for="option in entityOptions" :key="option.key" :value="option.key">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <label>
              目标
              <select v-model="relationTargetKey">
                <option v-for="option in entityOptions" :key="option.key" :value="option.key">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <label>
              关系类型
              <input v-model="relationForm.relationType" list="relation-type-presets" />
            </label>
            <datalist id="relation-type-presets">
              <option v-for="type in relationTypes" :key="type" :value="type" />
            </datalist>
            <label>置信度<input v-model.number="relationForm.confidence" min="0" max="1" step="0.05" type="number" /></label>
            <label class="toggle-row"><input v-model="relationForm.directed" type="checkbox" />有方向</label>
            <label class="wide">描述<textarea v-model="relationForm.description" rows="5" /></label>
          </div>
        </LibraryEditorShell>
      </main>

      <aside class="object-inspector">
        <template v-if="selectedEntityRef && neighborhood">
          <header>
            <div>
              <p class="eyebrow">{{ entityTypeLabel(neighborhood.center.entityType) }}</p>
              <h2>{{ neighborhood.center.title }}</h2>
              <p>{{ neighborhood.center.subtitle || "未分类" }}</p>
            </div>
            <strong>{{ neighborhood.relationCount }}</strong>
          </header>

          <section>
            <h3>摘要</h3>
            <p>{{ neighborhood.center.summary || "暂无摘要" }}</p>
          </section>

          <section>
            <h3>已有关联</h3>
            <ul class="inspector-list">
              <li v-for="edge in neighborhood.edges" :key="edge.id">
                <button type="button" @click="selectEdgeNeighbor(edge)">
                  <strong>{{ edge.relationType }}</strong>
                  <span>{{ edgeLabel(edge) }}</span>
                </button>
              </li>
            </ul>
            <p v-if="neighborhood.edges.length === 0" class="muted">暂无已确认关系</p>
          </section>

          <section>
            <h3>待确认建议</h3>
            <ul class="inspector-list">
              <li v-for="suggestion in neighborhood.suggestions" :key="suggestionKey(suggestion)">
                <div class="suggestion-row">
                  <div>
                    <strong>{{ suggestion.relationType }} · {{ suggestion.strength }}</strong>
                    <span>{{ suggestionLabel(suggestion) }} · {{ suggestion.reason }}</span>
                  </div>
                  <button type="button" class="secondary-button" @click="confirmSuggestion(suggestion)">确认</button>
                </div>
              </li>
            </ul>
            <p v-if="neighborhood.suggestions.length === 0" class="muted">暂无待确认建议</p>
          </section>

          <section>
            <h3>缺失项</h3>
            <div class="chip-list">
              <span v-for="item in neighborhood.missing" :key="item" class="status-chip">{{ item }}</span>
              <span v-if="neighborhood.missing.length === 0" class="status-chip ok">结构完整</span>
            </div>
          </section>

          <section>
            <h3>邻域预览</h3>
            <div class="neighborhood-map">
              <button
                v-for="node in neighborhood.nodes"
                :key="`${node.entityType}:${node.entityId}`"
                type="button"
                class="map-node"
                :class="{ active: node.entityType === neighborhood.center.entityType && node.entityId === neighborhood.center.entityId }"
                @click="selectEntity(node.entityType, node.entityId)"
              >
                <strong>{{ node.title }}</strong>
                <span>{{ entityTypeLabel(node.entityType) }}</span>
              </button>
            </div>
          </section>
        </template>

        <template v-else-if="selected.kind === 'relation' && selectedRelation">
          <header>
            <div>
              <p class="eyebrow">关系</p>
              <h2>{{ selectedRelation.relationType }}</h2>
              <p>{{ selectedRelation.confidence.toFixed(2) }} 置信度</p>
            </div>
          </header>
          <section>
            <h3>连接</h3>
            <p>{{ entityLabel(selectedRelation.source, entityOptions) }}</p>
            <p>{{ entityLabel(selectedRelation.target, entityOptions) }}</p>
          </section>
        </template>

        <div v-else class="empty-state compact">
          <h2>未选择</h2>
        </div>
      </aside>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, reactive, ref, watch } from "vue";
import { useRoute } from "vue-router";
import * as libraryApi from "@/api/library";
import EntryTemplatePanel from "@/components/entry/EntryTemplatePanel.vue";
import LibraryEditorShell from "@/components/library/LibraryEditorShell.vue";
import { entityTypeLabel, entryStatusLabel, entryTypeLabel } from "@/domain/displayLabels";
import {
  RELATION_TYPE_PRESETS,
  axiomToDraft,
  buildEntityOptions,
  characterToDraft,
  emptyAxiomDraft,
  emptyCharacterDraft,
  emptyEntryDraft,
  emptyEventDraft,
  emptyRelationDraft,
  entityLabel,
  entryToDraft,
  eventToDraft,
  makeEntityKey,
  newAxiomDraft,
  newCharacterDraft,
  newEntryDraft,
  newEventDraft,
  newRelationDraft,
  normalizeNullable,
  parseEntityKey,
  parseList,
  relationCountForEntity,
  relationSuggestionToDraft,
  relationToDraft,
  selectionToEntityRef,
  type LibraryRecordKind,
  type LibrarySelection,
} from "@/domain/libraryWorkspace";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import type {
  AxiomDraft,
  CharacterDraft,
  EntityRef,
  EntryDraft,
  EventDraft,
  Relation,
  RelationDraft,
  RelationNeighborhood,
  RelationSuggestion,
} from "@/types/library";

const RichTextEditor = defineAsyncComponent(() => import("@/components/RichTextEditor.vue"));

const editorTitles: Record<LibraryRecordKind, string> = {
  entry: "词条",
  character: "角色",
  event: "事件",
  axiom: "公理",
  relation: "关系",
};

interface ObjectCollection {
  kind: LibraryRecordKind;
  label: string;
  count: number;
}

interface ObjectRow {
  id: string;
  kind: LibraryRecordKind;
  title: string;
  typeLabel: string;
  status: string;
  relationCount: number;
  updatedAt: string;
}

const route = useRoute();
const projectStore = useProjectStore();
const libraryStore = useLibraryStore();
const activeCollection = ref<LibraryRecordKind>("entry");
const entryType = ref("world_rule");
const selected = reactive<LibrarySelection>({
  kind: null,
  id: null,
});
const neighborhood = ref<RelationNeighborhood | null>(null);
const relationTypes = ref<string[]>([...RELATION_TYPE_PRESETS]);

const entryForm = reactive<EntryDraft>(emptyEntryDraft(""));
const characterForm = reactive<CharacterDraft>(emptyCharacterDraft(""));
const eventForm = reactive<EventDraft>(emptyEventDraft(""));
const axiomForm = reactive<AxiomDraft>(emptyAxiomDraft(""));
const relationForm = reactive<RelationDraft>(emptyRelationDraft(""));
const entryTagsText = ref("");
const characterAliasesText = ref("");
const characterTagsText = ref("");
const eventTagsText = ref("");
const axiomTagsText = ref("");
const sourceEntityTypeText = ref("");
const sourceEntityIdText = ref("");
const relationSourceKey = ref("");
const relationTargetKey = ref("");

const projectId = computed(() => {
  const value = route.params.projectId;
  return typeof value === "string" && value.length > 0 ? value : projectStore.activeProjectId;
});
const projectTitle = computed(() => projectStore.activeProject?.name ?? "资料");
const editorTitle = computed(() => (selected.kind ? editorTitles[selected.kind] : "资料"));
const entityOptions = computed(() => buildEntityOptions(libraryStore));
const selectedEntityRef = computed(() => selectionToEntityRef(selected));
const selectedRelation = computed(() =>
  selected.kind === "relation" ? libraryStore.relations.find((relation) => relation.id === selected.id) ?? null : null,
);
const collections = computed<ObjectCollection[]>(() => [
  { kind: "entry", label: "资料", count: libraryStore.entries.length },
  { kind: "character", label: "角色", count: libraryStore.characters.length },
  { kind: "event", label: "事件", count: libraryStore.events.length },
  { kind: "axiom", label: "规则", count: libraryStore.axioms.length },
  { kind: "relation", label: "关系", count: libraryStore.relations.length },
]);
const activeCollectionLabel = computed(
  () => collections.value.find((collection) => collection.kind === activeCollection.value)?.label ?? "对象",
);
const objectRows = computed<ObjectRow[]>(() => {
  switch (activeCollection.value) {
    case "entry":
      return libraryStore.entries.map((entry) => entityRow("entry", entry.id, entry.title, entryTypeLabel(entry.entryType), entryStatusLabel(entry.status), entry.updatedAt));
    case "character":
      return libraryStore.characters.map((character) => entityRow("character", character.id, character.name, "角色", character.faction || "未分配阵营", character.updatedAt));
    case "event":
      return libraryStore.events.map((event) => entityRow("event", event.id, event.title, "事件", event.timeLabel || "未定时间", event.updatedAt));
    case "axiom":
      return libraryStore.axioms.map((axiom) => entityRow("axiom", axiom.id, axiom.subject, "规则", `${axiom.predicate} = ${axiom.object}`, axiom.updatedAt));
    case "relation":
      return libraryStore.relations.map((relation) => ({
        id: relation.id,
        kind: "relation",
        title: relation.relationType,
        typeLabel: "关系",
        status: relation.confidence < 0.6 ? "低置信度" : "已确认",
        relationCount: 1,
        updatedAt: relation.updatedAt,
      }));
  }
});

onMounted(async () => {
  if (projectStore.projects.length === 0) {
    await projectStore.loadProjects();
  }
  if (projectId.value) {
    await libraryStore.loadProject(projectId.value);
  }
  void loadRelationTypes();
});

watch(projectId, (id) => {
  clearSelection();
  if (id) {
    void libraryStore.loadProject(id);
  }
});

watch(
  () => [selected.kind, selected.id, projectId.value, libraryStore.relations.length] as const,
  () => {
    void loadNeighborhood();
  },
);

async function loadRelationTypes() {
  try {
    relationTypes.value = await libraryApi.relationTypePresets();
  } catch {
    relationTypes.value = [...RELATION_TYPE_PRESETS];
  }
}

async function loadNeighborhood() {
  neighborhood.value = null;
  if (!projectId.value || !selectedEntityRef.value) return;
  neighborhood.value = await libraryApi.relationNeighborhood(projectId.value, selectedEntityRef.value, 1);
}

function entityRow(
  kind: Exclude<LibraryRecordKind, "relation">,
  id: string,
  title: string,
  typeLabel: string,
  status: string,
  updatedAt: string,
): ObjectRow {
  return {
    id,
    kind,
    title,
    typeLabel,
    status,
    relationCount: relationCountForEntity(libraryStore.relations, { entityType: kind, entityId: id }),
    updatedAt,
  };
}

async function createRecord(kind: LibraryRecordKind) {
  if (!projectId.value) return;
  activeCollection.value = kind;
  switch (kind) {
    case "entry": {
      const entry = await libraryStore.createEntry(newEntryDraft(projectId.value, entryType.value, libraryStore.entries.length));
      selectRecord("entry", entry.id);
      break;
    }
    case "character": {
      const character = await libraryStore.createCharacter(newCharacterDraft(projectId.value, libraryStore.characters.length));
      selectRecord("character", character.id);
      break;
    }
    case "event": {
      const event = await libraryStore.createEvent(newEventDraft(projectId.value, libraryStore.events.length));
      selectRecord("event", event.id);
      break;
    }
    case "axiom": {
      const axiom = await libraryStore.createAxiom(newAxiomDraft(projectId.value));
      selectRecord("axiom", axiom.id);
      break;
    }
    case "relation": {
      const draft = newRelationDraft(projectId.value, entityOptions.value);
      if (!draft) return;
      const relation = await libraryStore.createRelation(draft);
      selectRecord("relation", relation.id);
      break;
    }
  }
}

function selectRecord(kind: LibraryRecordKind, id: string) {
  activeCollection.value = kind;
  selected.kind = kind;
  selected.id = id;
  syncSelectedForm();
}

function selectEntity(entityType: string, entityId: string) {
  if (entityType === "relation") return;
  selectRecord(entityType as LibraryRecordKind, entityId);
}

function clearSelection() {
  selected.kind = null;
  selected.id = null;
  neighborhood.value = null;
}

async function saveSelected() {
  if (!projectId.value || !selected.kind || !selected.id) return;
  const id = selected.id;
  const currentProjectId = projectId.value;
  const saveActions: Record<LibraryRecordKind, () => Promise<unknown>> = {
    entry: () =>
      libraryStore.updateEntry(id, {
        ...entryForm,
        projectId: currentProjectId,
        tags: parseList(entryTagsText.value),
      }),
    character: () =>
      libraryStore.updateCharacter(id, {
        ...characterForm,
        projectId: currentProjectId,
        aliases: parseList(characterAliasesText.value),
        tags: parseList(characterTagsText.value),
      }),
    event: () =>
      libraryStore.updateEvent(id, {
        ...eventForm,
        projectId: currentProjectId,
        tags: parseList(eventTagsText.value),
      }),
    axiom: () =>
      libraryStore.updateAxiom(id, {
        ...axiomForm,
        projectId: currentProjectId,
        sourceEntityType: normalizeNullable(sourceEntityTypeText.value),
        sourceEntityId: normalizeNullable(sourceEntityIdText.value),
        tags: parseList(axiomTagsText.value),
      }),
    relation: () =>
      libraryStore.updateRelation(id, {
        ...relationForm,
        projectId: currentProjectId,
        source: parseEntityKey(relationSourceKey.value),
        target: parseEntityKey(relationTargetKey.value),
      }),
  };
  await saveActions[selected.kind]();
  syncSelectedForm();
  await loadNeighborhood();
}

async function deleteSelected() {
  if (!selected.kind || !selected.id) return;
  const { kind, id } = selected;
  const deleteActions: Record<LibraryRecordKind, (recordId: string) => Promise<void>> = {
    entry: (recordId) => libraryStore.deleteEntry(recordId),
    character: (recordId) => libraryStore.deleteCharacter(recordId),
    event: (recordId) => libraryStore.deleteEvent(recordId),
    axiom: (recordId) => libraryStore.deleteAxiom(recordId),
    relation: (recordId) => libraryStore.deleteRelation(recordId),
  };
  await deleteActions[kind](id);
  clearSelection();
}

async function confirmSuggestion(suggestion: RelationSuggestion) {
  if (!projectId.value) return;
  await libraryStore.createRelation(relationSuggestionToDraft(projectId.value, suggestion));
  await loadNeighborhood();
}

function syncSelectedForm() {
  if (!selected.kind || !selected.id) return;
  switch (selected.kind) {
    case "entry": {
      const entry = libraryStore.entries.find((item) => item.id === selected.id);
      if (!entry) return;
      Object.assign(entryForm, entryToDraft(entry));
      entryTagsText.value = entry.tags.join(", ");
      break;
    }
    case "character": {
      const character = libraryStore.characters.find((item) => item.id === selected.id);
      if (!character) return;
      Object.assign(characterForm, characterToDraft(character));
      characterAliasesText.value = character.aliases.join(", ");
      characterTagsText.value = character.tags.join(", ");
      break;
    }
    case "event": {
      const event = libraryStore.events.find((item) => item.id === selected.id);
      if (!event) return;
      Object.assign(eventForm, eventToDraft(event));
      eventTagsText.value = event.tags.join(", ");
      break;
    }
    case "axiom": {
      const axiom = libraryStore.axioms.find((item) => item.id === selected.id);
      if (!axiom) return;
      Object.assign(axiomForm, axiomToDraft(axiom));
      sourceEntityTypeText.value = axiom.sourceEntityType ?? "";
      sourceEntityIdText.value = axiom.sourceEntityId ?? "";
      axiomTagsText.value = axiom.tags.join(", ");
      break;
    }
    case "relation": {
      const relation = libraryStore.relations.find((item) => item.id === selected.id);
      if (!relation) return;
      Object.assign(relationForm, relationToDraft(relation));
      relationSourceKey.value = makeEntityKey(relation.source.entityType, relation.source.entityId);
      relationTargetKey.value = makeEntityKey(relation.target.entityType, relation.target.entityId);
      break;
    }
  }
}

function edgeLabel(edge: Relation): string {
  return `${entityLabel(edge.source, entityOptions.value)} -> ${entityLabel(edge.target, entityOptions.value)}`;
}

function suggestionLabel(suggestion: RelationSuggestion): string {
  return `${entityLabel(suggestion.source, entityOptions.value)} -> ${entityLabel(suggestion.target, entityOptions.value)}`;
}

function suggestionKey(suggestion: RelationSuggestion): string {
  return `${suggestion.source.entityType}:${suggestion.source.entityId}:${suggestion.target.entityType}:${suggestion.target.entityId}:${suggestion.relationType}`;
}

function selectEdgeNeighbor(edge: Relation) {
  const current = selectedEntityRef.value;
  if (!current) return;
  const next: EntityRef =
    edge.source.entityType === current.entityType && edge.source.entityId === current.entityId ? edge.target : edge.source;
  selectEntity(next.entityType, next.entityId);
}
</script>
