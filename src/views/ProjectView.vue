<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">工作台</p>
        <h1>{{ projectTitle }}</h1>
      </div>
      <p v-if="libraryStore.error" class="status-note error">{{ libraryStore.error }}</p>
    </header>

    <div v-if="!projectId" class="empty-state">
      <h2>未选择项目</h2>
    </div>

    <div v-else class="workspace-grid">
      <LibraryRecordPanel
        title="词条"
        :rows="entryRows"
        :selected="selected"
        @create="createRecord('entry')"
        @select="selectRecord"
      >
        <EntryTemplatePanel v-model="entryType" />
      </LibraryRecordPanel>

      <LibraryRecordPanel
        v-for="panel in secondaryPanels"
        :key="panel.kind"
        :title="panel.title"
        :rows="panel.rows"
        :selected="selected"
        :panel-class="panel.panelClass"
        :create-disabled="panel.createDisabled"
        @create="createRecord(panel.kind)"
        @select="selectRecord"
      />

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
          <label>关系类型<input v-model="relationForm.relationType" /></label>
          <label>置信度<input v-model.number="relationForm.confidence" min="0" max="1" step="0.05" type="number" /></label>
          <label class="toggle-row"><input v-model="relationForm.directed" type="checkbox" />有方向</label>
          <label class="wide">描述<textarea v-model="relationForm.description" rows="5" /></label>
        </div>
      </LibraryEditorShell>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, reactive, ref, watch } from "vue";
import { useRoute } from "vue-router";
import EntryTemplatePanel from "@/components/entry/EntryTemplatePanel.vue";
import LibraryEditorShell from "@/components/library/LibraryEditorShell.vue";
import LibraryRecordPanel from "@/components/library/LibraryRecordPanel.vue";
import { entryStatusLabel, entryTypeLabel } from "@/domain/displayLabels";
import {
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
  relationToDraft,
  type LibraryPanelRow,
  type LibraryRecordKind,
  type LibrarySelection,
} from "@/domain/libraryWorkspace";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import type { AxiomDraft, CharacterDraft, EntryDraft, EventDraft, RelationDraft } from "@/types/library";

const RichTextEditor = defineAsyncComponent(() => import("@/components/RichTextEditor.vue"));

const editorTitles: Record<LibraryRecordKind, string> = {
  entry: "词条",
  character: "角色",
  event: "事件",
  axiom: "公理",
  relation: "关系",
};

interface LibraryPanelConfig {
  kind: LibraryRecordKind;
  title: string;
  rows: LibraryPanelRow[];
  panelClass?: string;
  createDisabled?: boolean;
}

const route = useRoute();
const projectStore = useProjectStore();
const libraryStore = useLibraryStore();
const entryType = ref("world_rule");
const selected = reactive<LibrarySelection>({
  kind: null,
  id: null,
});

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

const entryRows = computed<LibraryPanelRow[]>(() =>
  libraryStore.entries.map((entry) => ({
    id: entry.id,
    kind: "entry",
    title: entry.title,
    meta: `${entryTypeLabel(entry.entryType)} · ${entryStatusLabel(entry.status)}`,
  })),
);
const characterRows = computed<LibraryPanelRow[]>(() =>
  libraryStore.characters.map((character) => ({
    id: character.id,
    kind: "character",
    title: character.name,
    meta: character.faction || "未分配阵营",
  })),
);
const eventRows = computed<LibraryPanelRow[]>(() =>
  libraryStore.events.map((event) => ({
    id: event.id,
    kind: "event",
    title: event.title,
    meta: event.timeLabel || "未定时间",
  })),
);
const axiomRows = computed<LibraryPanelRow[]>(() =>
  libraryStore.axioms.map((axiom) => ({
    id: axiom.id,
    kind: "axiom",
    title: axiom.subject,
    meta: `${axiom.predicate} = ${axiom.object}`,
  })),
);
const relationRows = computed<LibraryPanelRow[]>(() =>
  libraryStore.relations.map((relation) => ({
    id: relation.id,
    kind: "relation",
    title: relation.relationType,
    meta: `${entityLabel(relation.source, entityOptions.value)} → ${entityLabel(relation.target, entityOptions.value)}`,
  })),
);
const secondaryPanels = computed<LibraryPanelConfig[]>(() => [
  { kind: "character", title: "角色", rows: characterRows.value },
  { kind: "event", title: "事件", rows: eventRows.value },
  { kind: "axiom", title: "公理", rows: axiomRows.value },
  {
    kind: "relation",
    title: "关系",
    rows: relationRows.value,
    panelClass: "relation-panel",
    createDisabled: entityOptions.value.length < 2,
  },
]);

onMounted(async () => {
  if (projectStore.projects.length === 0) {
    await projectStore.loadProjects();
  }
  if (projectId.value) {
    await libraryStore.loadProject(projectId.value);
  }
});

watch(projectId, (id) => {
  clearSelection();
  if (id) {
    void libraryStore.loadProject(id);
  }
});

async function createRecord(kind: LibraryRecordKind) {
  if (!projectId.value) return;
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
  selected.kind = kind;
  selected.id = id;
  syncSelectedForm();
}

function clearSelection() {
  selected.kind = null;
  selected.id = null;
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
</script>
