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
      <article class="library-panel">
        <header>
          <h2>词条</h2>
          <button type="button" @click="createEntry">新增</button>
        </header>
        <EntryTemplatePanel v-model="entryType" />
        <ul>
          <li v-for="entry in libraryStore.entries" :key="entry.id">
            <button
              type="button"
              class="record-row"
              :class="{ active: selected.kind === 'entry' && selected.id === entry.id }"
              @click="selectRecord('entry', entry.id)"
            >
              <strong>{{ entry.title }}</strong>
              <span>{{ entry.entryType }} · {{ entry.status }}</span>
            </button>
          </li>
        </ul>
      </article>

      <article class="library-panel">
        <header>
          <h2>角色</h2>
          <button type="button" @click="createCharacter">新增</button>
        </header>
        <ul>
          <li v-for="character in libraryStore.characters" :key="character.id">
            <button
              type="button"
              class="record-row"
              :class="{ active: selected.kind === 'character' && selected.id === character.id }"
              @click="selectRecord('character', character.id)"
            >
              <strong>{{ character.name }}</strong>
              <span>{{ character.faction || "未分配阵营" }}</span>
            </button>
          </li>
        </ul>
      </article>

      <article class="library-panel">
        <header>
          <h2>事件</h2>
          <button type="button" @click="createEvent">新增</button>
        </header>
        <ul>
          <li v-for="event in libraryStore.events" :key="event.id">
            <button
              type="button"
              class="record-row"
              :class="{ active: selected.kind === 'event' && selected.id === event.id }"
              @click="selectRecord('event', event.id)"
            >
              <strong>{{ event.title }}</strong>
              <span>{{ event.timeLabel || "未定时间" }}</span>
            </button>
          </li>
        </ul>
      </article>

      <article class="library-panel">
        <header>
          <h2>公理</h2>
          <button type="button" @click="createAxiom">新增</button>
        </header>
        <ul>
          <li v-for="axiom in libraryStore.axioms" :key="axiom.id">
            <button
              type="button"
              class="record-row"
              :class="{ active: selected.kind === 'axiom' && selected.id === axiom.id }"
              @click="selectRecord('axiom', axiom.id)"
            >
              <strong>{{ axiom.subject }}</strong>
              <span>{{ axiom.predicate }} = {{ axiom.object }}</span>
            </button>
          </li>
        </ul>
      </article>

      <article class="library-panel relation-panel">
        <header>
          <h2>关系</h2>
          <button type="button" :disabled="entityOptions.length < 2" @click="createRelation">新增</button>
        </header>
        <ul>
          <li v-for="relation in libraryStore.relations" :key="relation.id">
            <button
              type="button"
              class="record-row"
              :class="{ active: selected.kind === 'relation' && selected.id === relation.id }"
              @click="selectRecord('relation', relation.id)"
            >
              <strong>{{ relation.relationType }}</strong>
              <span>{{ entityLabel(relation.source) }} -> {{ entityLabel(relation.target) }}</span>
            </button>
          </li>
        </ul>
      </article>

      <article class="library-editor">
        <header>
          <div>
            <h2>{{ editorTitle }}</h2>
          </div>
          <div class="editor-actions">
            <button type="button" class="secondary-button" :disabled="!selected.id" @click="deleteSelected">
              删除
            </button>
            <button type="button" class="primary-button" :disabled="!selected.id" @click="saveSelected">
              保存
            </button>
          </div>
        </header>

        <div v-if="selected.kind === 'entry'" class="editor-form">
          <label>标题<input v-model="entryForm.title" /></label>
          <label>类型<input v-model="entryForm.entryType" /></label>
          <label>状态<input v-model="entryForm.status" /></label>
          <label>标签<input v-model="entryTagsText" placeholder="标签" /></label>
          <label class="wide">摘要<textarea v-model="entryForm.summary" rows="3" /></label>
          <label class="wide">正文<textarea v-model="entryForm.body" rows="8" /></label>
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
          <label>来源 ID<input v-model="sourceEntityIdText" /></label>
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

        <div v-else class="empty-state compact">
          <h2>未选择</h2>
        </div>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useRoute } from "vue-router";
import EntryTemplatePanel from "@/components/entry/EntryTemplatePanel.vue";
import { getTemplate } from "@/domain/entryTemplates";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import type {
  AxiomDraft,
  CharacterDraft,
  EntityRef,
  EntryDraft,
  EventDraft,
  RelationDraft,
} from "@/types/library";

type RecordKind = "entry" | "character" | "event" | "axiom" | "relation" | null;

const route = useRoute();
const projectStore = useProjectStore();
const libraryStore = useLibraryStore();
const entryType = ref("world_rule");
const selected = reactive<{ kind: RecordKind; id: string | null }>({
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
const editorTitle = computed(() => {
  if (!selected.kind) return "资料";
  return {
    entry: "词条",
    character: "角色",
    event: "事件",
    axiom: "公理",
    relation: "关系",
  }[selected.kind];
});
const entityOptions = computed(() => [
  ...libraryStore.entries.map((entry) => ({
    key: makeEntityKey("entry", entry.id),
    label: `词条：${entry.title}`,
  })),
  ...libraryStore.characters.map((character) => ({
    key: makeEntityKey("character", character.id),
    label: `角色：${character.name}`,
  })),
  ...libraryStore.events.map((event) => ({
    key: makeEntityKey("event", event.id),
    label: `事件：${event.title}`,
  })),
  ...libraryStore.axioms.map((axiom) => ({
    key: makeEntityKey("axiom", axiom.id),
    label: `公理：${axiom.subject}`,
  })),
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

async function createEntry() {
  if (!projectId.value) return;
  const template = getTemplate(entryType.value);
  const entry = await libraryStore.createEntry({
    projectId: projectId.value,
    entryType: template.type,
    title: `新词条 ${libraryStore.entries.length + 1}`,
    summary: template.summary,
    body: template.body,
    tags: template.tags,
    status: "draft",
  });
  selectRecord("entry", entry.id);
}

async function createCharacter() {
  if (!projectId.value) return;
  const character = await libraryStore.createCharacter({
    projectId: projectId.value,
    name: `新角色 ${libraryStore.characters.length + 1}`,
    aliases: [],
    summary: "",
    appearance: "",
    goals: "",
    motivations: "",
    fears: "",
    faction: "",
    tags: [],
  });
  selectRecord("character", character.id);
}

async function createEvent() {
  if (!projectId.value) return;
  const event = await libraryStore.createEvent({
    projectId: projectId.value,
    title: `新事件 ${libraryStore.events.length + 1}`,
    description: "",
    timeLabel: "",
    sortKey: Date.now(),
    startLabel: "",
    endLabel: "",
    location: "",
    importance: 1,
    outcome: "",
    tags: [],
  });
  selectRecord("event", event.id);
}

async function createAxiom() {
  if (!projectId.value) return;
  const axiom = await libraryStore.createAxiom({
    projectId: projectId.value,
    subject: "新主体",
    predicate: "defines",
    object: "新对象",
    scopeTime: "",
    scopeLocation: "",
    certainty: 1,
    sourceEntityType: null,
    sourceEntityId: null,
    naturalLanguage: "",
    tags: [],
  });
  selectRecord("axiom", axiom.id);
}

async function createRelation() {
  if (!projectId.value || entityOptions.value.length < 2) return;
  const source = parseEntityKey(entityOptions.value[0].key);
  const target = parseEntityKey(entityOptions.value[1].key);
  const relation = await libraryStore.createRelation({
    projectId: projectId.value,
    source,
    target,
    relationType: "relates_to",
    description: "",
    confidence: 1,
    directed: true,
  });
  selectRecord("relation", relation.id);
}

function selectRecord(kind: Exclude<RecordKind, null>, id: string) {
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
  if (selected.kind === "entry") {
    await libraryStore.updateEntry(selected.id, {
      ...entryForm,
      projectId: projectId.value,
      tags: parseList(entryTagsText.value),
    });
  } else if (selected.kind === "character") {
    await libraryStore.updateCharacter(selected.id, {
      ...characterForm,
      projectId: projectId.value,
      aliases: parseList(characterAliasesText.value),
      tags: parseList(characterTagsText.value),
    });
  } else if (selected.kind === "event") {
    await libraryStore.updateEvent(selected.id, {
      ...eventForm,
      projectId: projectId.value,
      tags: parseList(eventTagsText.value),
    });
  } else if (selected.kind === "axiom") {
    await libraryStore.updateAxiom(selected.id, {
      ...axiomForm,
      projectId: projectId.value,
      sourceEntityType: normalizeNullable(sourceEntityTypeText.value),
      sourceEntityId: normalizeNullable(sourceEntityIdText.value),
      tags: parseList(axiomTagsText.value),
    });
  } else if (selected.kind === "relation") {
    await libraryStore.updateRelation(selected.id, {
      ...relationForm,
      projectId: projectId.value,
      source: parseEntityKey(relationSourceKey.value),
      target: parseEntityKey(relationTargetKey.value),
    });
  }
  syncSelectedForm();
}

async function deleteSelected() {
  if (!selected.kind || !selected.id) return;
  const { kind, id } = selected;
  if (kind === "entry") await libraryStore.deleteEntry(id);
  if (kind === "character") await libraryStore.deleteCharacter(id);
  if (kind === "event") await libraryStore.deleteEvent(id);
  if (kind === "axiom") await libraryStore.deleteAxiom(id);
  if (kind === "relation") await libraryStore.deleteRelation(id);
  clearSelection();
}

function syncSelectedForm() {
  if (selected.kind === "entry") {
    const entry = libraryStore.entries.find((item) => item.id === selected.id);
    if (!entry) return;
    Object.assign(entryForm, {
      projectId: entry.projectId,
      entryType: entry.entryType,
      title: entry.title,
      summary: entry.summary,
      body: entry.body,
      tags: [...entry.tags],
      status: entry.status,
    });
    entryTagsText.value = entry.tags.join(", ");
  } else if (selected.kind === "character") {
    const character = libraryStore.characters.find((item) => item.id === selected.id);
    if (!character) return;
    Object.assign(characterForm, {
      projectId: character.projectId,
      name: character.name,
      aliases: [...character.aliases],
      summary: character.summary,
      appearance: character.appearance,
      goals: character.goals,
      motivations: character.motivations,
      fears: character.fears,
      faction: character.faction,
      tags: [...character.tags],
    });
    characterAliasesText.value = character.aliases.join(", ");
    characterTagsText.value = character.tags.join(", ");
  } else if (selected.kind === "event") {
    const event = libraryStore.events.find((item) => item.id === selected.id);
    if (!event) return;
    Object.assign(eventForm, {
      projectId: event.projectId,
      title: event.title,
      description: event.description,
      timeLabel: event.timeLabel,
      sortKey: event.sortKey,
      startLabel: event.startLabel,
      endLabel: event.endLabel,
      location: event.location,
      importance: event.importance,
      outcome: event.outcome,
      tags: [...event.tags],
    });
    eventTagsText.value = event.tags.join(", ");
  } else if (selected.kind === "axiom") {
    const axiom = libraryStore.axioms.find((item) => item.id === selected.id);
    if (!axiom) return;
    Object.assign(axiomForm, {
      projectId: axiom.projectId,
      subject: axiom.subject,
      predicate: axiom.predicate,
      object: axiom.object,
      scopeTime: axiom.scopeTime,
      scopeLocation: axiom.scopeLocation,
      certainty: axiom.certainty,
      sourceEntityType: axiom.sourceEntityType,
      sourceEntityId: axiom.sourceEntityId,
      naturalLanguage: axiom.naturalLanguage,
      tags: [...axiom.tags],
    });
    sourceEntityTypeText.value = axiom.sourceEntityType ?? "";
    sourceEntityIdText.value = axiom.sourceEntityId ?? "";
    axiomTagsText.value = axiom.tags.join(", ");
  } else if (selected.kind === "relation") {
    const relation = libraryStore.relations.find((item) => item.id === selected.id);
    if (!relation) return;
    Object.assign(relationForm, {
      projectId: relation.projectId,
      source: { ...relation.source },
      target: { ...relation.target },
      relationType: relation.relationType,
      description: relation.description,
      confidence: relation.confidence,
      directed: relation.directed,
    });
    relationSourceKey.value = makeEntityKey(relation.source.entityType, relation.source.entityId);
    relationTargetKey.value = makeEntityKey(relation.target.entityType, relation.target.entityId);
  }
}

function emptyEntryDraft(projectId: string): EntryDraft {
  return { projectId, entryType: "", title: "", summary: "", body: "", tags: [], status: "draft" };
}

function emptyCharacterDraft(projectId: string): CharacterDraft {
  return {
    projectId,
    name: "",
    aliases: [],
    summary: "",
    appearance: "",
    goals: "",
    motivations: "",
    fears: "",
    faction: "",
    tags: [],
  };
}

function emptyEventDraft(projectId: string): EventDraft {
  return {
    projectId,
    title: "",
    description: "",
    timeLabel: "",
    sortKey: 0,
    startLabel: "",
    endLabel: "",
    location: "",
    importance: 1,
    outcome: "",
    tags: [],
  };
}

function emptyAxiomDraft(projectId: string): AxiomDraft {
  return {
    projectId,
    subject: "",
    predicate: "",
    object: "",
    scopeTime: "",
    scopeLocation: "",
    certainty: 1,
    sourceEntityType: null,
    sourceEntityId: null,
    naturalLanguage: "",
    tags: [],
  };
}

function emptyRelationDraft(projectId: string): RelationDraft {
  return {
    projectId,
    source: { entityType: "entry", entityId: "" },
    target: { entityType: "entry", entityId: "" },
    relationType: "",
    description: "",
    confidence: 1,
    directed: true,
  };
}

function parseList(value: string) {
  return value
    .split(/[,，]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function normalizeNullable(value: string) {
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : null;
}

function makeEntityKey(entityType: string, entityId: string) {
  return `${entityType}:${entityId}`;
}

function parseEntityKey(key: string): EntityRef {
  const separator = key.indexOf(":");
  if (separator < 0) {
    return { entityType: "entry", entityId: key };
  }
  return {
    entityType: key.slice(0, separator),
    entityId: key.slice(separator + 1),
  };
}

function entityLabel(entity: EntityRef) {
  return (
    entityOptions.value.find((option) => option.key === makeEntityKey(entity.entityType, entity.entityId))
      ?.label ?? `${entity.entityType}:${entity.entityId}`
  );
}
</script>
