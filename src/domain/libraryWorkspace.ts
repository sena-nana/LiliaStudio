import { entityTypeLabel } from "@/domain/displayLabels";
import { getTemplate } from "@/domain/entryTemplates";
import type {
  Axiom,
  AxiomDraft,
  Character,
  CharacterDraft,
  EntityRef,
  Entry,
  EntryDraft,
  EventDraft,
  EventRecord,
  Relation,
  RelationDraft,
  RelationSuggestion,
} from "@/types/library";

export type LibraryRecordKind = "entry" | "character" | "event" | "axiom" | "relation";

export const RELATION_TYPE_PRESETS = [
  "参与事件",
  "发生于",
  "属于阵营",
  "导致",
  "支撑规则",
  "约束",
  "来源于",
  "冲突候选",
  "语义相关",
] as const;

export interface LibrarySelection {
  kind: LibraryRecordKind | null;
  id: string | null;
}

export interface LibraryEntityOption {
  key: string;
  label: string;
}

export interface LibraryPanelRow {
  id: string;
  kind: LibraryRecordKind;
  title: string;
  meta: string;
}

export interface LibraryCollections {
  entries: Entry[];
  characters: Character[];
  events: EventRecord[];
  axioms: Axiom[];
  relations: Relation[];
}

export function parseList(value: string): string[] {
  return value
    .split(/[,，]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

export function normalizeNullable(value: string): string | null {
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : null;
}

export function makeEntityKey(entityType: string, entityId: string): string {
  return `${entityType}:${entityId}`;
}

export function parseEntityKey(key: string): EntityRef {
  const separator = key.indexOf(":");
  if (separator < 0) {
    return { entityType: "entry", entityId: key };
  }
  return {
    entityType: key.slice(0, separator),
    entityId: key.slice(separator + 1),
  };
}

export function buildEntityOptions(collections: Pick<LibraryCollections, "entries" | "characters" | "events" | "axioms">): LibraryEntityOption[] {
  return [
    ...collections.entries.map((entry) => ({
      key: makeEntityKey("entry", entry.id),
      label: `词条：${entry.title}`,
    })),
    ...collections.characters.map((character) => ({
      key: makeEntityKey("character", character.id),
      label: `角色：${character.name}`,
    })),
    ...collections.events.map((event) => ({
      key: makeEntityKey("event", event.id),
      label: `事件：${event.title}`,
    })),
    ...collections.axioms.map((axiom) => ({
      key: makeEntityKey("axiom", axiom.id),
      label: `公理：${axiom.subject}`,
    })),
  ];
}

export function entityLabel(entity: EntityRef, options: LibraryEntityOption[]): string {
  return (
    options.find((option) => option.key === makeEntityKey(entity.entityType, entity.entityId))?.label ??
    `${entityTypeLabel(entity.entityType)}：${entity.entityId}`
  );
}

export function selectionToEntityRef(selection: LibrarySelection): EntityRef | null {
  if (!selection.kind || !selection.id || selection.kind === "relation") {
    return null;
  }
  return { entityType: selection.kind, entityId: selection.id };
}

export function relationTouchesEntity(relation: Relation, entity: EntityRef): boolean {
  return (
    (relation.source.entityType === entity.entityType && relation.source.entityId === entity.entityId) ||
    (relation.target.entityType === entity.entityType && relation.target.entityId === entity.entityId)
  );
}

export function relationCountForEntity(relations: Relation[], entity: EntityRef): number {
  return relations.filter((relation) => relationTouchesEntity(relation, entity)).length;
}

export function relationSuggestionToDraft(projectId: string, suggestion: RelationSuggestion): RelationDraft {
  return {
    projectId,
    source: { ...suggestion.source },
    target: { ...suggestion.target },
    relationType: suggestion.relationType,
    description: suggestion.description,
    confidence: suggestion.confidence,
    directed: suggestion.directed,
  };
}

export function emptyEntryDraft(projectId: string): EntryDraft {
  return { projectId, entryType: "", title: "", summary: "", body: "", tags: [], status: "draft" };
}

export function emptyCharacterDraft(projectId: string): CharacterDraft {
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

export function emptyEventDraft(projectId: string): EventDraft {
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

export function emptyAxiomDraft(projectId: string): AxiomDraft {
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

export function emptyRelationDraft(projectId: string): RelationDraft {
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

export function newEntryDraft(projectId: string, entryType: string, count: number): EntryDraft {
  const template = getTemplate(entryType);
  return {
    projectId,
    entryType: template.type,
    title: `新词条 ${count + 1}`,
    summary: template.summary,
    body: template.body,
    tags: [...template.tags],
    status: "draft",
  };
}

export function newCharacterDraft(projectId: string, count: number): CharacterDraft {
  return {
    ...emptyCharacterDraft(projectId),
    name: `新角色 ${count + 1}`,
  };
}

export function newEventDraft(projectId: string, count: number): EventDraft {
  return {
    ...emptyEventDraft(projectId),
    title: `新事件 ${count + 1}`,
    sortKey: Date.now(),
  };
}

export function newAxiomDraft(projectId: string): AxiomDraft {
  return {
    ...emptyAxiomDraft(projectId),
    subject: "新主体",
    predicate: "定义",
    object: "新对象",
  };
}

export function newRelationDraft(projectId: string, options: LibraryEntityOption[]): RelationDraft | null {
  if (options.length < 2) return null;
  return {
    ...emptyRelationDraft(projectId),
    source: parseEntityKey(options[0].key),
    target: parseEntityKey(options[1].key),
    relationType: "关联",
  };
}

export function entryToDraft(entry: Entry): EntryDraft {
  return {
    projectId: entry.projectId,
    entryType: entry.entryType,
    title: entry.title,
    summary: entry.summary,
    body: entry.body,
    tags: [...entry.tags],
    status: entry.status,
  };
}

export function characterToDraft(character: Character): CharacterDraft {
  return {
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
  };
}

export function eventToDraft(event: EventRecord): EventDraft {
  return {
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
  };
}

export function axiomToDraft(axiom: Axiom): AxiomDraft {
  return {
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
  };
}

export function relationToDraft(relation: Relation): RelationDraft {
  return {
    projectId: relation.projectId,
    source: { ...relation.source },
    target: { ...relation.target },
    relationType: relation.relationType,
    description: relation.description,
    confidence: relation.confidence,
    directed: relation.directed,
  };
}
