<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">备份</p>
        <h1>项目导入导出</h1>
      </div>
      <button type="button" class="primary-button" @click="exportArchive">导出 JSON</button>
    </header>

    <textarea v-model="archiveText" class="archive-text" placeholder="导出的 JSON 会显示在这里，也可以粘贴 JSON 后导入。" />
    <button type="button" class="primary-button" @click="importArchive">导入为新项目</button>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { exportProjectArchive, importProjectArchive } from '@/api/importExport'
import { useProjectStore } from '@/stores/projectStore'
import type { ProjectArchive } from '@/types/archive'

const route = useRoute()
const projectStore = useProjectStore()
const archiveText = ref('')
const projectId = computed(() => {
  const value = route.params.projectId
  return typeof value === 'string' && value.length > 0 ? value : projectStore.activeProjectId
})

async function exportArchive() {
  if (!projectId.value) return
  archiveText.value = JSON.stringify(await exportProjectArchive(projectId.value), null, 2)
}

async function importArchive() {
  if (!archiveText.value.trim()) return
  const archive = JSON.parse(archiveText.value) as ProjectArchive
  const imported = await importProjectArchive(archive)
  projectStore.projects = [imported.project, ...projectStore.projects]
  projectStore.activeProjectId = imported.project.id
}
</script>
