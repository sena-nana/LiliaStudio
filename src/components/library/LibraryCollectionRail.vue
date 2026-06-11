<template>
  <aside class="object-rail">
    <header>
      <h2>对象</h2>
    </header>
    <button
      v-for="collection in collections"
      :key="collection.kind"
      type="button"
      class="collection-row"
      :class="{ active: activeCollection === collection.kind }"
      @click="$emit('update:activeCollection', collection.kind)"
    >
      <span>{{ collection.label }}</span>
      <strong>{{ collection.count }}</strong>
    </button>
  </aside>
</template>

<script setup lang="ts">
import type { ObjectCollection } from "@/composables/useLibraryWorkspace";
import type { LibraryRecordKind } from "@/domain/libraryWorkspace";

defineProps<{
  activeCollection: LibraryRecordKind;
  collections: ObjectCollection[];
}>();

defineEmits<{
  "update:activeCollection": [kind: LibraryRecordKind];
}>();
</script>
