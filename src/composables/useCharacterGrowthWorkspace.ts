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

const emptyState: CharacterTraitState = {
  values: {},
  sources: [],
};

export function useCharacterGrowthWorkspace() {
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

  return {
    draft,
    eventTitle,
    formatRecordMeta,
    formatSigned,
    isPreviewing,
    isSaving,
    libraryStore,
    parsedDelta,
    previewDisabled,
    previewEntries,
    previewState,
    projectId,
    runPreview,
    saveDisabled,
    saveRecord,
    selectedCharacter,
    selectedCharacterId,
    selectedRecords,
    sortedCharacters,
    sortedEvents,
    statusKind,
    statusLabel,
    statusMessage,
    traitEntries,
    uniqueEventCount,
    visibleRecords,
    workspaceLoading,
  };
}
