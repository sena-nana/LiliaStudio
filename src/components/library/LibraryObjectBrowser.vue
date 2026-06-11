<template>
  <section class="object-list-panel">
    <header>
      <div>
        <h2>{{ activeCollectionLabel }}</h2>
        <p>{{ rows.length }} 个对象</p>
      </div>
      <div class="object-actions">
        <EntryTemplatePanel v-if="activeCollection === 'entry'" :model-value="entryType" @update:model-value="$emit('update:entryType', $event)" />
        <button type="button" class="secondary-button" :disabled="createDisabled" @click="$emit('create', activeCollection)">
          新增
        </button>
      </div>
    </header>

    <div class="object-table">
      <button
        v-for="row in rows"
        :key="`${row.kind}:${row.id}`"
        type="button"
        class="object-table-row"
        :class="{ active: selected.kind === row.kind && selected.id === row.id }"
        @click="$emit('select', row.kind, row.id)"
      >
        <span class="object-title">{{ row.title }}</span>
        <span>{{ row.typeLabel }}</span>
        <span>{{ row.status }}</span>
        <span>{{ row.relationCount }} 关联</span>
        <span>{{ row.updatedAt || "未记录" }}</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import EntryTemplatePanel from "@/components/entry/EntryTemplatePanel.vue";
import type { ObjectRow } from "@/composables/useLibraryWorkspace";
import type { LibraryRecordKind, LibrarySelection } from "@/domain/libraryWorkspace";

defineProps<{
  activeCollection: LibraryRecordKind;
  activeCollectionLabel: string;
  createDisabled: boolean;
  entryType: string;
  rows: ObjectRow[];
  selected: LibrarySelection;
}>();

defineEmits<{
  create: [kind: LibraryRecordKind];
  select: [kind: LibraryRecordKind, id: string];
  "update:entryType": [entryType: string];
}>();
</script>
