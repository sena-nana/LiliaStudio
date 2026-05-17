<template>
  <div v-if="open" class="command-palette" role="dialog" aria-label="命令面板">
    <ul>
      <li v-for="(command, index) in commands" :key="command.to">
        <button type="button" :ref="setFirstCommandButton(index)" @click="go(command.to)">
          {{ command.label }}
        </button>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import type { ComponentPublicInstance } from 'vue'
import { useRouter } from 'vue-router'
import { workspaceMenuEntries, workspaceTabs } from '@/components/layout/workspaceModel'

const router = useRouter()
const open = ref(false)
const firstCommandButton = ref<HTMLButtonElement | null>(null)
const commands = computed(() => [
  ...workspaceTabs.map((tab) => ({ label: tab.label, to: tab.to })),
  ...workspaceMenuEntries,
])

function onKeydown(event: KeyboardEvent) {
  if (event.ctrlKey && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    open.value = !open.value
    if (open.value) {
      void focusFirstCommand()
    }
  }
  if (open.value && event.key === 'Escape') {
    event.preventDefault()
    open.value = false
  }
}

function setFirstCommandButton(index: number) {
  return (element: Element | ComponentPublicInstance | null) => {
    if (index === 0) {
      firstCommandButton.value = element instanceof HTMLButtonElement ? element : null
    }
  }
}

async function focusFirstCommand() {
  await nextTick()
  firstCommandButton.value?.focus()
}

function go(to: string) {
  open.value = false
  void router.push(to)
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>
