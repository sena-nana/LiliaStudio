import { reactive, ref, type Ref } from "vue";
import {
  axiomToDraft,
  characterToDraft,
  emptyAxiomDraft,
  emptyCharacterDraft,
  emptyEntryDraft,
  emptyEventDraft,
  emptyRelationDraft,
  entryToDraft,
  eventToDraft,
  makeEntityKey,
  normalizeNullable,
  parseEntityKey,
  parseList,
  relationToDraft,
  type LibraryRecordKind,
} from "@/domain/libraryWorkspace";
import type { useLibraryStore } from "@/stores/libraryStore";
import type {
  AxiomDraft,
  CharacterDraft,
  EntryDraft,
  EventDraft,
  RelationDraft,
} from "@/types/library";

type LibraryStore = ReturnType<typeof useLibraryStore>;

export interface LibraryRecordForms {
  entry: EntryDraft;
  character: CharacterDraft;
  event: EventDraft;
  axiom: AxiomDraft;
  relation: RelationDraft;
  entryTagsText: Ref<string>;
  characterAliasesText: Ref<string>;
  characterTagsText: Ref<string>;
  eventTagsText: Ref<string>;
  axiomTagsText: Ref<string>;
  sourceEntityTypeText: Ref<string>;
  sourceEntityIdText: Ref<string>;
  relationSourceKey: Ref<string>;
  relationTargetKey: Ref<string>;
}

export function useLibraryRecordForms(libraryStore: LibraryStore) {
  const forms: LibraryRecordForms = {
    entry: reactive<EntryDraft>(emptyEntryDraft("")),
    character: reactive<CharacterDraft>(emptyCharacterDraft("")),
    event: reactive<EventDraft>(emptyEventDraft("")),
    axiom: reactive<AxiomDraft>(emptyAxiomDraft("")),
    relation: reactive<RelationDraft>(emptyRelationDraft("")),
    entryTagsText: ref(""),
    characterAliasesText: ref(""),
    characterTagsText: ref(""),
    eventTagsText: ref(""),
    axiomTagsText: ref(""),
    sourceEntityTypeText: ref(""),
    sourceEntityIdText: ref(""),
    relationSourceKey: ref(""),
    relationTargetKey: ref(""),
  };

  function syncSelectedForm(kind: LibraryRecordKind, id: string) {
    switch (kind) {
      case "entry": {
        const entry = libraryStore.entries.find((item) => item.id === id);
        if (!entry) return;
        Object.assign(forms.entry, entryToDraft(entry));
        forms.entryTagsText.value = entry.tags.join(", ");
        break;
      }
      case "character": {
        const character = libraryStore.characters.find((item) => item.id === id);
        if (!character) return;
        Object.assign(forms.character, characterToDraft(character));
        forms.characterAliasesText.value = character.aliases.join(", ");
        forms.characterTagsText.value = character.tags.join(", ");
        break;
      }
      case "event": {
        const event = libraryStore.events.find((item) => item.id === id);
        if (!event) return;
        Object.assign(forms.event, eventToDraft(event));
        forms.eventTagsText.value = event.tags.join(", ");
        break;
      }
      case "axiom": {
        const axiom = libraryStore.axioms.find((item) => item.id === id);
        if (!axiom) return;
        Object.assign(forms.axiom, axiomToDraft(axiom));
        forms.sourceEntityTypeText.value = axiom.sourceEntityType ?? "";
        forms.sourceEntityIdText.value = axiom.sourceEntityId ?? "";
        forms.axiomTagsText.value = axiom.tags.join(", ");
        break;
      }
      case "relation": {
        const relation = libraryStore.relations.find((item) => item.id === id);
        if (!relation) return;
        Object.assign(forms.relation, relationToDraft(relation));
        forms.relationSourceKey.value = makeEntityKey(relation.source.entityType, relation.source.entityId);
        forms.relationTargetKey.value = makeEntityKey(relation.target.entityType, relation.target.entityId);
        break;
      }
    }
  }

  async function saveSelectedForm(kind: LibraryRecordKind, id: string, projectId: string) {
    const saveActions: Record<LibraryRecordKind, () => Promise<unknown>> = {
      entry: () =>
        libraryStore.updateEntry(id, {
          ...forms.entry,
          projectId,
          tags: parseList(forms.entryTagsText.value),
        }),
      character: () =>
        libraryStore.updateCharacter(id, {
          ...forms.character,
          projectId,
          aliases: parseList(forms.characterAliasesText.value),
          tags: parseList(forms.characterTagsText.value),
        }),
      event: () =>
        libraryStore.updateEvent(id, {
          ...forms.event,
          projectId,
          tags: parseList(forms.eventTagsText.value),
        }),
      axiom: () =>
        libraryStore.updateAxiom(id, {
          ...forms.axiom,
          projectId,
          sourceEntityType: normalizeNullable(forms.sourceEntityTypeText.value),
          sourceEntityId: normalizeNullable(forms.sourceEntityIdText.value),
          tags: parseList(forms.axiomTagsText.value),
        }),
      relation: () =>
        libraryStore.updateRelation(id, {
          ...forms.relation,
          projectId,
          source: parseEntityKey(forms.relationSourceKey.value),
          target: parseEntityKey(forms.relationTargetKey.value),
        }),
    };
    await saveActions[kind]();
  }

  return {
    forms,
    saveSelectedForm,
    syncSelectedForm,
  };
}
