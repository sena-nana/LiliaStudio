<template>
  <div v-if="open" class="command-palette">
    <button v-for="command in commands" :key="command.to" type="button" @click="go(command.to)">
      {{ command.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const open = ref(false)
const commands = [
  { label: '打开项目库', to: '/' },
  { label: '打开审计', to: '/audit' },
  { label: '打开模拟', to: '/simulation' },
  { label: '打开诊断', to: '/diagnostics' },
]

function onKeydown(event: KeyboardEvent) {
  if (event.ctrlKey && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    open.value = !open.value
  }
}

function go(to: string) {
  open.value = false
  void router.push(to)
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>
