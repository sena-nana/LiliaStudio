<template>
  <article class="library-panel" :class="panelClass">
    <header>
      <h2>{{ title }}</h2>
      <button type="button" :disabled="createDisabled" @click="$emit('create')">新增</button>
    </header>
    <slot />
    <ul>
      <li v-for="row in rows" :key="row.id">
        <button
          type="button"
          class="record-row"
          :class="{ active: selected.kind === row.kind && selected.id === row.id }"
          @click="$emit('select', row.kind, row.id)"
        >
          <strong>{{ row.title }}</strong>
          <span>{{ row.meta }}</span>
        </button>
      </li>
    </ul>
  </article>
</template>

<script setup lang="ts">
import type { LibraryPanelRow, LibraryRecordKind, LibrarySelection } from "@/domain/libraryWorkspace";

defineProps<{
  title: string;
  rows: LibraryPanelRow[];
  selected: LibrarySelection;
  createDisabled?: boolean;
  panelClass?: string;
}>();

defineEmits<{
  create: [];
  select: [kind: LibraryRecordKind, id: string];
}>();
</script>
