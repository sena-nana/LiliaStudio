import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { buildEntityOptions, isLibraryRecordKind } from "@/domain/libraryWorkspace";
import {
  buildGraphEdges,
  buildGraphEntityRefs,
  buildGraphNodes,
  edgeLabel as relationGraphEdgeLabel,
  uniqueSorted,
} from "@/domain/relationGraph";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import type { Relation } from "@/types/library";

export function useRelationGraphWorkspace() {
  const route = useRoute();
  const router = useRouter();
  const projectStore = useProjectStore();
  const libraryStore = useLibraryStore();
  const relationTypeFilter = ref("");
  const entityTypeFilter = ref("");
  const focusedNodeKey = ref("");

  const projectId = computed(() => {
    const value = route.params.projectId;
    return typeof value === "string" && value.length > 0 ? value : projectStore.activeProjectId;
  });
  const entityOptions = computed(() => buildEntityOptions(libraryStore));
  const relationTypes = computed(() => uniqueSorted(libraryStore.relations.map((relation) => relation.relationType).filter(Boolean)));
  const graphEntityRefs = computed(() => buildGraphEntityRefs(libraryStore.relations));
  const entityTypes = computed(() => uniqueSorted(graphEntityRefs.value.map((entity) => entity.entityType)));
  const visibleEdges = computed(() =>
    libraryStore.relations.filter((relation) => {
      if (relationTypeFilter.value && relation.relationType !== relationTypeFilter.value) return false;
      if (!entityTypeFilter.value) return true;
      return relation.source.entityType === entityTypeFilter.value || relation.target.entityType === entityTypeFilter.value;
    }),
  );
  const graphNodes = computed(() =>
    buildGraphNodes(visibleEdges.value, libraryStore, entityOptions.value, focusedNodeKey.value),
  );
  const graphEdges = computed(() => buildGraphEdges(visibleEdges.value, graphNodes.value));
  const focusedNode = computed(() => graphNodes.value.find((node) => node.key === focusedNodeKey.value) ?? null);

  onMounted(async () => {
    if (projectStore.projects.length === 0) {
      await projectStore.loadProjects();
    }
    if (projectId.value) {
      await libraryStore.loadProject(projectId.value);
    }
  });

  watch(projectId, (id) => {
    focusedNodeKey.value = "";
    if (id) {
      void libraryStore.loadProject(id);
    }
  });

  watch([relationTypeFilter, entityTypeFilter], () => {
    focusedNodeKey.value = "";
  });

  function focusNode(key: string) {
    focusedNodeKey.value = focusedNodeKey.value === key ? "" : key;
  }

  function openRecord(kind: string, id: string) {
    if (!projectId.value || !isLibraryRecordKind(kind)) return;
    void router.push({
      name: "project",
      params: { projectId: projectId.value },
      query: { kind, id },
    });
  }

  function edgeLabel(edge: Relation): string {
    return relationGraphEdgeLabel(edge, entityOptions.value);
  }

  return {
    edgeLabel,
    entityTypeFilter,
    entityTypes,
    focusNode,
    focusedNode,
    focusedNodeKey,
    graphEdges,
    graphNodes,
    libraryStore,
    openRecord,
    projectId,
    relationTypeFilter,
    relationTypes,
    visibleEdges,
  };
}
