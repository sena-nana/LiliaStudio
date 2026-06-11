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
      <LibraryCollectionRail
        v-model:active-collection="activeCollection"
        :collections="collections"
      />

      <main class="object-main">
        <LibraryObjectBrowser
          v-model:entry-type="entryType"
          :active-collection="activeCollection"
          :active-collection-label="activeCollectionLabel"
          :create-disabled="activeCollection === 'relation' && entityOptions.length < 2"
          :rows="objectRows"
          :selected="selected"
          @create="createRecord"
          @select="selectRecord"
        />

        <LibraryRecordEditor
          :entity-options="entityOptions"
          :forms="forms"
          :relation-types="relationTypes"
          :selected="selected"
          :title="editorTitle"
          @delete="deleteSelected"
          @save="saveSelected"
        />
      </main>

      <LibraryObjectInspector
        :entity-options="entityOptions"
        :neighborhood="neighborhood"
        :selected="selected"
        :selected-entity-ref="selectedEntityRef"
        :selected-relation="selectedRelation"
        @confirm-suggestion="confirmSuggestion"
        @select-edge-neighbor="selectEdgeNeighbor"
        @select-entity="selectEntity"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import LibraryCollectionRail from "@/components/library/LibraryCollectionRail.vue";
import LibraryObjectBrowser from "@/components/library/LibraryObjectBrowser.vue";
import LibraryObjectInspector from "@/components/library/LibraryObjectInspector.vue";
import LibraryRecordEditor from "@/components/library/LibraryRecordEditor.vue";
import { useLibraryWorkspace } from "@/composables/useLibraryWorkspace";

const {
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
} = useLibraryWorkspace();
</script>
