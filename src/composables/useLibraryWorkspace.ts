import { computed, nextTick, onMounted, reactive, ref, watch } from "vue";
import { useRoute } from "vue-router";
import * as libraryApi from "@/api/library";
import { useLibraryRecordForms } from "@/composables/useLibraryRecordForms";
import { entryStatusLabel, entryTypeLabel } from "@/domain/displayLabels";
import {
  RELATION_TYPE_PRESETS,
  buildEntityOptions,
  isLibraryRecordKind,
  newAxiomDraft,
  newCharacterDraft,
  newEntryDraft,
  newEventDraft,
  newRelationDraft,
  relationCountForEntity,
  relationSuggestionToDraft,
  selectionToEntityRef,
  type LibraryRecordKind,
  type LibrarySelection,
} from "@/domain/libraryWorkspace";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import type {
  EntityRef,
  Relation,
  RelationNeighborhood,
  RelationSuggestion,
} from "@/types/library";

const editorTitles: Record<LibraryRecordKind, string> = {
  entry: "词条",
  character: "角色",
  event: "事件",
  axiom: "公理",
  relation: "关系",
};

export interface ObjectCollection {
  kind: LibraryRecordKind;
  label: string;
  count: number;
}

export interface ObjectRow {
  id: string;
  kind: LibraryRecordKind;
  title: string;
  typeLabel: string;
  status: string;
  relationCount: number;
  updatedAt: string;
}

export function useLibraryWorkspace() {
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
  const { forms, saveSelectedForm, syncSelectedForm } = useLibraryRecordForms(libraryStore);

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
        return libraryStore.entries.map((entry) =>
          entityRow("entry", entry.id, entry.title, entryTypeLabel(entry.entryType), entryStatusLabel(entry.status), entry.updatedAt),
        );
      case "character":
        return libraryStore.characters.map((character) =>
          entityRow("character", character.id, character.name, "角色", character.faction || "未分配阵营", character.updatedAt),
        );
      case "event":
        return libraryStore.events.map((event) =>
          entityRow("event", event.id, event.title, "事件", event.timeLabel || "未定时间", event.updatedAt),
        );
      case "axiom":
        return libraryStore.axioms.map((axiom) =>
          entityRow("axiom", axiom.id, axiom.subject, "规则", `${axiom.predicate} = ${axiom.object}`, axiom.updatedAt),
        );
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
      applyRouteSelection();
    }
    void loadRelationTypes();
  });

  watch(projectId, (id) => {
    clearSelection();
    if (id) {
      void libraryStore.loadProject(id).then(applyRouteSelection);
    }
  });

  watch(
    () => [route.query.kind, route.query.id] as const,
    () => {
      void nextTick(applyRouteSelection);
    },
  );

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
    syncSelectedForm(kind, id);
  }

  function applyRouteSelection() {
    const kind = typeof route.query.kind === "string" ? route.query.kind : "";
    const id = typeof route.query.id === "string" ? route.query.id : "";
    if (!id || !isLibraryRecordKind(kind) || !recordExists(kind, id)) return;
    if (selected.kind === kind && selected.id === id) return;
    selectRecord(kind, id);
  }

  function recordExists(kind: LibraryRecordKind, id: string): boolean {
    const records = {
      entry: libraryStore.entries,
      character: libraryStore.characters,
      event: libraryStore.events,
      axiom: libraryStore.axioms,
      relation: libraryStore.relations,
    };
    return records[kind].some((record) => record.id === id);
  }

  function selectEntity(entityType: string, entityId: string) {
    if (!isLibraryRecordKind(entityType) || entityType === "relation") return;
    selectRecord(entityType, entityId);
  }

  function clearSelection() {
    selected.kind = null;
    selected.id = null;
    neighborhood.value = null;
  }

  async function saveSelected() {
    if (!projectId.value || !selected.kind || !selected.id) return;
    await saveSelectedForm(selected.kind, selected.id, projectId.value);
    syncSelectedForm(selected.kind, selected.id);
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

  function selectEdgeNeighbor(edge: Relation) {
    const current = selectedEntityRef.value;
    if (!current) return;
    const next: EntityRef =
      edge.source.entityType === current.entityType && edge.source.entityId === current.entityId ? edge.target : edge.source;
    selectEntity(next.entityType, next.entityId);
  }

  return {
    activeCollection,
    activeCollectionLabel,
    collections,
    confirmSuggestion,
    createRecord,
    deleteSelected,
    editorTitle,
    entityOptions,
    entryType,
    forms,
    libraryStore,
    neighborhood,
    objectRows,
    projectId,
    projectTitle,
    relationTypes,
    saveSelected,
    selectEdgeNeighbor,
    selectEntity,
    selectRecord,
    selected,
    selectedEntityRef,
    selectedRelation,
  };
}
