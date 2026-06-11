import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { entryStatusLabel, entryTypeLabel } from "@/domain/displayLabels";
import { buildEntityOptions, entityLabel, isLibraryRecordKind, makeEntityKey } from "@/domain/libraryWorkspace";
import { useLibraryStore } from "@/stores/libraryStore";
import { useProjectStore } from "@/stores/projectStore";
import type { EntityRef, Relation } from "@/types/library";

interface GraphNode {
  key: string;
  entityType: string;
  entityId: string;
  title: string;
  subtitle: string;
  x: number;
  y: number;
  relationCount: number;
  relatedToFocus: boolean;
}

interface GraphEdge {
  id: string;
  source: GraphNode;
  target: GraphNode;
}

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
  const graphEntityRefs = computed(() => {
    const refs = new Map<string, EntityRef>();
    for (const relation of libraryStore.relations) {
      refs.set(makeEntityKey(relation.source.entityType, relation.source.entityId), relation.source);
      refs.set(makeEntityKey(relation.target.entityType, relation.target.entityId), relation.target);
    }
    return [...refs.values()];
  });
  const entityTypes = computed(() => uniqueSorted(graphEntityRefs.value.map((entity) => entity.entityType)));
  const visibleEdges = computed(() =>
    libraryStore.relations.filter((relation) => {
      if (relationTypeFilter.value && relation.relationType !== relationTypeFilter.value) return false;
      if (!entityTypeFilter.value) return true;
      return relation.source.entityType === entityTypeFilter.value || relation.target.entityType === entityTypeFilter.value;
    }),
  );
  const graphNodes = computed<GraphNode[]>(() => {
    const refs = new Map<string, EntityRef>();
    const relationCounts = new Map<string, number>();
    const relatedKeys = new Set<string>();

    for (const relation of visibleEdges.value) {
      const sourceKey = makeEntityKey(relation.source.entityType, relation.source.entityId);
      const targetKey = makeEntityKey(relation.target.entityType, relation.target.entityId);
      refs.set(sourceKey, relation.source);
      refs.set(targetKey, relation.target);
      relationCounts.set(sourceKey, (relationCounts.get(sourceKey) ?? 0) + 1);
      relationCounts.set(targetKey, (relationCounts.get(targetKey) ?? 0) + 1);
      if (sourceKey === focusedNodeKey.value) relatedKeys.add(targetKey);
      if (targetKey === focusedNodeKey.value) relatedKeys.add(sourceKey);
    }

    const refsList = [...refs.values()];
    return refsList.map((refValue, index) => {
      const key = makeEntityKey(refValue.entityType, refValue.entityId);
      const point = layoutPoint(index, refsList.length);
      const summary = entitySummary(refValue);
      return {
        key,
        entityType: refValue.entityType,
        entityId: refValue.entityId,
        title: summary.title,
        subtitle: summary.subtitle,
        x: point.x,
        y: point.y,
        relationCount: relationCounts.get(key) ?? 0,
        relatedToFocus: !focusedNodeKey.value || key === focusedNodeKey.value || relatedKeys.has(key),
      };
    });
  });
  const graphEdges = computed<GraphEdge[]>(() => {
    const nodesByKey = new Map(graphNodes.value.map((node) => [node.key, node]));
    return visibleEdges.value.flatMap((relation) => {
      const source = nodesByKey.get(makeEntityKey(relation.source.entityType, relation.source.entityId));
      const target = nodesByKey.get(makeEntityKey(relation.target.entityType, relation.target.entityId));
      return source && target ? [{ id: relation.id, source, target }] : [];
    });
  });
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

  function layoutPoint(index: number, total: number): { x: number; y: number } {
    if (total === 1) return { x: 50, y: 50 };
    const ring = index === 0 ? 0 : Math.ceil(index / 12);
    const ringStart = ring === 0 ? 0 : 1 + (ring - 1) * 12;
    const ringSize = ring === 0 ? 1 : Math.min(12, total - ringStart);
    const angle = ((index - ringStart) / ringSize) * Math.PI * 2 - Math.PI / 2;
    const radius = Math.min(40, 18 + ring * 12);
    return {
      x: Math.round((50 + Math.cos(angle) * radius) * 10) / 10,
      y: Math.round((50 + Math.sin(angle) * radius) * 10) / 10,
    };
  }

  function entitySummary(entity: EntityRef): { title: string; subtitle: string } {
    switch (entity.entityType) {
      case "entry": {
        const entry = libraryStore.entries.find((item) => item.id === entity.entityId);
        return entry
          ? { title: entry.title, subtitle: `${entryTypeLabel(entry.entryType)} · ${entryStatusLabel(entry.status)}` }
          : fallbackEntitySummary(entity);
      }
      case "character": {
        const character = libraryStore.characters.find((item) => item.id === entity.entityId);
        return character ? { title: character.name, subtitle: character.faction || "未分配阵营" } : fallbackEntitySummary(entity);
      }
      case "event": {
        const event = libraryStore.events.find((item) => item.id === entity.entityId);
        return event ? { title: event.title, subtitle: event.timeLabel || "未定时间" } : fallbackEntitySummary(entity);
      }
      case "axiom": {
        const axiom = libraryStore.axioms.find((item) => item.id === entity.entityId);
        return axiom ? { title: axiom.subject, subtitle: `${axiom.predicate} = ${axiom.object}` } : fallbackEntitySummary(entity);
      }
      default:
        return fallbackEntitySummary(entity);
    }
  }

  function fallbackEntitySummary(entity: EntityRef): { title: string; subtitle: string } {
    return {
      title: entityLabel(entity, entityOptions.value),
      subtitle: entity.entityId,
    };
  }

  function edgeLabel(edge: Relation): string {
    const arrow = edge.directed ? " -> " : " - ";
    return `${entityLabel(edge.source, entityOptions.value)}${arrow}${entityLabel(edge.target, entityOptions.value)}`;
  }

  function uniqueSorted(values: string[]): string[] {
    return [...new Set(values)].sort((left, right) => left.localeCompare(right, "zh-Hans-CN"));
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
