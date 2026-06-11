import { entryStatusLabel, entryTypeLabel } from "@/domain/displayLabels";
import { entityLabel, makeEntityKey, type LibraryEntityOption } from "@/domain/libraryWorkspace";
import type { EntityRef, LibraryProjectSnapshot, Relation } from "@/types/library";

export interface GraphNode {
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

export interface GraphEdge {
  id: string;
  source: GraphNode;
  target: GraphNode;
}

export function buildGraphEntityRefs(relations: Relation[]): EntityRef[] {
  const refs = new Map<string, EntityRef>();
  for (const relation of relations) {
    refs.set(makeEntityKey(relation.source.entityType, relation.source.entityId), relation.source);
    refs.set(makeEntityKey(relation.target.entityType, relation.target.entityId), relation.target);
  }
  return [...refs.values()];
}

export function buildGraphNodes(
  edges: Relation[],
  collections: Pick<LibraryProjectSnapshot, "entries" | "characters" | "events" | "axioms">,
  entityOptions: LibraryEntityOption[],
  focusedNodeKey: string,
): GraphNode[] {
  const refs = new Map<string, EntityRef>();
  const relationCounts = new Map<string, number>();
  const relatedKeys = new Set<string>();

  for (const relation of edges) {
    const sourceKey = makeEntityKey(relation.source.entityType, relation.source.entityId);
    const targetKey = makeEntityKey(relation.target.entityType, relation.target.entityId);
    refs.set(sourceKey, relation.source);
    refs.set(targetKey, relation.target);
    relationCounts.set(sourceKey, (relationCounts.get(sourceKey) ?? 0) + 1);
    relationCounts.set(targetKey, (relationCounts.get(targetKey) ?? 0) + 1);
    if (sourceKey === focusedNodeKey) relatedKeys.add(targetKey);
    if (targetKey === focusedNodeKey) relatedKeys.add(sourceKey);
  }

  const refsList = [...refs.values()];
  return refsList.map((refValue, index) => {
    const key = makeEntityKey(refValue.entityType, refValue.entityId);
    const point = layoutPoint(index, refsList.length);
    const summary = entitySummary(refValue, collections, entityOptions);
    return {
      key,
      entityType: refValue.entityType,
      entityId: refValue.entityId,
      title: summary.title,
      subtitle: summary.subtitle,
      x: point.x,
      y: point.y,
      relationCount: relationCounts.get(key) ?? 0,
      relatedToFocus: !focusedNodeKey || key === focusedNodeKey || relatedKeys.has(key),
    };
  });
}

export function buildGraphEdges(edges: Relation[], nodes: GraphNode[]): GraphEdge[] {
  const nodesByKey = new Map(nodes.map((node) => [node.key, node]));
  return edges.flatMap((relation) => {
    const source = nodesByKey.get(makeEntityKey(relation.source.entityType, relation.source.entityId));
    const target = nodesByKey.get(makeEntityKey(relation.target.entityType, relation.target.entityId));
    return source && target ? [{ id: relation.id, source, target }] : [];
  });
}

export function layoutPoint(index: number, total: number): { x: number; y: number } {
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

export function entitySummary(
  entity: EntityRef,
  collections: Pick<LibraryProjectSnapshot, "entries" | "characters" | "events" | "axioms">,
  options: LibraryEntityOption[],
): { title: string; subtitle: string } {
  switch (entity.entityType) {
    case "entry": {
      const entry = collections.entries.find((item) => item.id === entity.entityId);
      return entry
        ? { title: entry.title, subtitle: `${entryTypeLabel(entry.entryType)} · ${entryStatusLabel(entry.status)}` }
        : fallbackEntitySummary(entity, options);
    }
    case "character": {
      const character = collections.characters.find((item) => item.id === entity.entityId);
      return character ? { title: character.name, subtitle: character.faction || "未分配阵营" } : fallbackEntitySummary(entity, options);
    }
    case "event": {
      const event = collections.events.find((item) => item.id === entity.entityId);
      return event ? { title: event.title, subtitle: event.timeLabel || "未定时间" } : fallbackEntitySummary(entity, options);
    }
    case "axiom": {
      const axiom = collections.axioms.find((item) => item.id === entity.entityId);
      return axiom ? { title: axiom.subject, subtitle: `${axiom.predicate} = ${axiom.object}` } : fallbackEntitySummary(entity, options);
    }
    default:
      return fallbackEntitySummary(entity, options);
  }
}

export function edgeLabel(edge: Relation, options: LibraryEntityOption[]): string {
  const arrow = edge.directed ? " -> " : " - ";
  return `${entityLabel(edge.source, options)}${arrow}${entityLabel(edge.target, options)}`;
}

export function uniqueSorted(values: string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right, "zh-Hans-CN"));
}

function fallbackEntitySummary(entity: EntityRef, options: LibraryEntityOption[]): { title: string; subtitle: string } {
  return {
    title: entityLabel(entity, options),
    subtitle: entity.entityId,
  };
}
