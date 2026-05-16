<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">项目库</p>
        <h1>选择或创建一个创作宇宙</h1>
      </div>
      <button type="button" class="primary-button" @click="createDefaultProject">新建项目</button>
    </header>

    <form class="quick-create" @submit.prevent="createProject">
      <input v-model="draft.name" required placeholder="项目名称" />
      <input v-model="draft.description" placeholder="一句话描述" />
      <button type="submit">创建</button>
    </form>

    <div v-if="projectStore.loading" class="empty-state">
      <h2>正在加载</h2>
      <p>读取本地项目库。</p>
    </div>
    <div v-else-if="projectStore.projects.length === 0" class="empty-state">
      <h2>还没有项目</h2>
      <p>创建第一个项目后，词条、角色、事件和公理会围绕它组织。</p>
    </div>
    <div v-else class="project-list">
      <button
        v-for="project in projectStore.projects"
        :key="project.id"
        type="button"
        class="project-row"
        :class="{ active: project.id === projectStore.activeProjectId }"
        @click="openProject(project.id)"
      >
        <strong>{{ project.name }}</strong>
        <span>{{ project.description || '无描述' }}</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/projectStore'

const router = useRouter()
const projectStore = useProjectStore()
const draft = reactive({
  name: '',
  description: '',
})

onMounted(() => {
  void projectStore.loadProjects()
})

async function createProject() {
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
</script>
