<template>
  <article class="library-editor">
    <header>
      <div>
        <h2>{{ title }}</h2>
      </div>
      <div class="editor-actions">
        <button type="button" class="secondary-button" :disabled="!hasSelection" @click="$emit('delete')">
          删除
        </button>
        <button type="button" class="primary-button" :disabled="!hasSelection" @click="$emit('save')">
          保存
        </button>
      </div>
    </header>

    <slot v-if="hasSelection" />
    <div v-else class="empty-state compact">
      <h2>未选择</h2>
    </div>
  </article>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  hasSelection: boolean;
}>();

defineEmits<{
  delete: [];
  save: [];
}>();
</script>
