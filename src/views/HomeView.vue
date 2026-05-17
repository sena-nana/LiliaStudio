<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">项目库</p>
        <h1>项目</h1>
      </div>
      <button type="button" class="primary-button" @click="createDefaultProject">新建项目</button>
    </header>

    <form class="quick-create" @submit.prevent="createProject">
      <input v-model="draft.name" required placeholder="项目名称" />
      <input v-model="draft.description" placeholder="描述" />
      <button type="submit">创建</button>
    </form>

    <div v-if="projectStore.loading" class="empty-state">
      <h2>正在加载</h2>
    </div>
    <div v-else-if="projectStore.projects.length === 0" class="empty-state">
      <h2>还没有项目</h2>
    </div>
    <div v-else class="project-list">
      <article
        v-for="project in projectStore.projects"
        :key="project.id"
        class="project-row"
        :class="{ active: project.id === projectStore.activeProjectId }"
      >
        <button type="button" class="project-open-button" @click="openProject(project.id)">
          <strong>{{ project.name }}</strong>
          <span>{{ project.description || '无描述' }}</span>
        </button>
        <div class="project-edit-row">
          <input v-model="editDrafts[project.id].name" placeholder="项目名称" />
          <input v-model="editDrafts[project.id].description" placeholder="描述" />
          <button type="button" @click="saveProject(project.id)">保存</button>
          <button type="button" @click="archiveProject(project.id)">归档</button>
        </div>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { reactive, watchEffect } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/projectStore'

const router = useRouter()
const projectStore = useProjectStore()
const draft = reactive({
  name: '',
  description: '',
})
const editDrafts = reactive<Record<string, { name: string; description: string }>>({})

watchEffect(() => {
  for (const project of projectStore.projects) {
    if (!editDrafts[project.id]) {
      editDrafts[project.id] = {
        name: project.name,
        description: project.description,
      }
    }
  }
})

void projectStore.loadProjects()

async function createProject() {
  if (!draft.name.trim()) return
  const project = await projectStore.createProject({
    name: draft.name.trim(),
    description: draft.description.trim(),
  })
  draft.name = ''
  draft.description = ''
  await router.push(`/projects/${project.id}`)
}

async function createDefaultProject() {
  const project = await projectStore.createProject({
    name: '未命名宇宙',
    description: '',
  })
  await router.push(`/projects/${project.id}`)
}

async function openProject(id: string) {
  projectStore.selectProject(id)
  await router.push(`/projects/${id}`)
}

async function saveProject(id: string) {
  const draft = editDrafts[id]
  if (!draft?.name.trim()) return
  await projectStore.updateProject(id, {
    name: draft.name.trim(),
    description: draft.description.trim(),
  })
}

async function archiveProject(id: string) {
  await projectStore.archiveProject(id)
  delete editDrafts[id]
}
</script>
