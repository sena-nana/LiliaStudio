<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">工作台</p>
        <h1>{{ projectTitle }}</h1>
      </div>
    </header>

    <div v-if="!projectId" class="empty-state">
      <h2>未选择项目</h2>
      <p>请先在项目库中创建或打开一个项目。</p>
    </div>

    <div v-else class="tool-grid">
      <article class="library-panel">
        <header>
          <h2>词条</h2>
          <button type="button" @click="createEntry">新增</button>
        </header>
        <p>世界观、物品、地点、阵营和资源。</p>
        <EntryTemplatePanel v-model="entryType" />
        <ul>
          <li v-for="entry in libraryStore.entries" :key="entry.id">
            <strong>{{ entry.title }}</strong>
            <span>{{ entry.entryType }}</span>
          </li>
        </ul>
      </article>

      <article class="library-panel">
        <header>
          <h2>角色</h2>
          <button type="button" @click="createCharacter">新增</button>
        </header>
        <p>角色档案、目标、动机和阵营。</p>
        <ul>
          <li v-for="character in libraryStore.characters" :key="character.id">
            <strong>{{ character.name }}</strong>
            <span>{{ character.faction || '未分配阵营' }}</span>
          </li>
        </ul>
      </article>

      <article class="library-panel">
        <header>
          <h2>事件</h2>
          <button type="button" @click="createEvent">新增</button>
        </header>
        <p>历史节点、参与者和结果。</p>
        <ul>
          <li v-for="event in libraryStore.events" :key="event.id">
            <strong>{{ event.title }}</strong>
            <span>{{ event.timeLabel || '未定时间' }}</span>
          </li>
        </ul>
      </article>

      <article class="library-panel">
        <header>
          <h2>公理</h2>
          <button type="button" @click="createAxiom">新增</button>
        </header>
        <p>物理法则、社会契约和逻辑约束。</p>
        <ul>
          <li v-for="axiom in libraryStore.axioms" :key="axiom.id">
            <strong>{{ axiom.subject }}</strong>
            <span>{{ axiom.predicate }} = {{ axiom.object }}</span>
          </li>
        </ul>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import EntryTemplatePanel from '@/components/entry/EntryTemplatePanel.vue'
import { getTemplate } from '@/domain/entryTemplates'
import { useLibraryStore } from '@/stores/libraryStore'
import { useProjectStore } from '@/stores/projectStore'

const route = useRoute()
const projectStore = useProjectStore()
const libraryStore = useLibraryStore()
const entryType = ref('world_rule')

const projectId = computed(() => {
  const value = route.params.projectId
  return typeof value === 'string' && value.length > 0 ? value : projectStore.activeProjectId
})
const projectTitle = computed(() => projectStore.activeProject?.name ?? '世界观资料编辑')

onMounted(async () => {
  if (projectStore.projects.length === 0) {
    await projectStore.loadProjects()
  }
  if (projectId.value) {
    await libraryStore.loadProject(projectId.value)
  }
})

watch(projectId, (id) => {
  if (id) {
    void libraryStore.loadProject(id)
  }
})

async function createEntry() {
  if (!projectId.value) return
  const template = getTemplate(entryType.value)
  await libraryStore.createEntry({
    projectId: projectId.value,
    entryType: template.type,
    title: `新词条 ${libraryStore.entries.length + 1}`,
    summary: template.summary,
    body: template.body,
    tags: template.tags,
    status: 'draft',
  })
}

async function createCharacter() {
  if (!projectId.value) return
  await libraryStore.createCharacter({
    projectId: projectId.value,
    name: `新角色 ${libraryStore.characters.length + 1}`,
    aliases: [],
    summary: '',
    appearance: '',
    goals: '',
    motivations: '',
    fears: '',
    faction: '',
    tags: [],
  })
}

async function createEvent() {
  if (!projectId.value) return
  await libraryStore.createEvent({
    projectId: projectId.value,
    title: `新事件 ${libraryStore.events.length + 1}`,
    description: '',
    timeLabel: '',
    sortKey: Date.now(),
    startLabel: '',
    endLabel: '',
    location: '',
    importance: 1,
    outcome: '',
    tags: [],
  })
}

async function createAxiom() {
  if (!projectId.value) return
  await libraryStore.createAxiom({
    projectId: projectId.value,
    subject: '新主体',
    predicate: 'defines',
    object: '新对象',
    scopeTime: '',
    scopeLocation: '',
    certainty: 1,
    sourceEntityType: null,
    sourceEntityId: null,
    naturalLanguage: '',
    tags: [],
  })
}
</script>
