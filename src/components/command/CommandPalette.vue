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
import { SIDEBAR_FOOTER_LINKS, SIDEBAR_GLOBAL_ACTIONS, SIDEBAR_NAV } from '@/config/appShell'

const router = useRouter()
const open = ref(false)
const firstCommandButton = ref<HTMLButtonElement | null>(null)
const commands = computed(() => [
  ...SIDEBAR_GLOBAL_ACTIONS.filter((action) => action.to && !action.disabled).map((action) => ({
    label: action.label,
    to: action.to ?? '/',
  })),
  ...SIDEBAR_NAV.filter((item) => item.to && !item.disabled).map((item) => ({
    label: item.label,
    to: item.to ?? '/',
  })),
  ...SIDEBAR_FOOTER_LINKS.map((item) => ({ label: item.title ?? item.label, to: item.to })),
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
