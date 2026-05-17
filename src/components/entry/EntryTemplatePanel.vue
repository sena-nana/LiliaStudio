<template>
  <div class="template-panel">
    <button
      v-for="template in entryTemplates"
      :key="template.type"
      type="button"
      :class="{ active: template.type === modelValue }"
      @click="$emit('update:modelValue', template.type)"
    >
      <strong>{{ template.label }}</strong>
    </button>
  </div>
</template>

<script setup lang="ts">
import { entryTemplates } from '@/domain/entryTemplates'

defineProps<{
  modelValue: string
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>
