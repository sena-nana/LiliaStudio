# Linear Object Workspace UX-M1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the first usable slice of the new object-workspace shell: top object tabs, compact collection rail, shared Inspector scaffold, and route-aware command/menu entry points, while keeping existing page content and data models intact.

**Architecture:** Add a small pure workspace-model module that maps current routes to object-workspace state and collection presets. Refactor the shell into focused layout components so the top bar, left rail, right Inspector, and footer status line share one predictable layout. This plan deliberately stops before per-object page splitting and deep editors; those belong in UX-M2.

**Tech Stack:** Vue 3, TypeScript, Vue Router, Pinia, Vitest, Playwright, Tauri 2.

---

## Scope Note

This plan is only for **UX-M1**:

- build the shell chrome
- make the top object tabs visible and route-aware
- make the left rail a real collection rail
- make the right rail a stable Inspector scaffold
- keep the existing center views rendering as they are today

This plan does **not** split `src/views/*` into the new object workspaces yet. That is the next plan.

## File Structure

- Create: `src/components/layout/workspaceModel.ts`
- Create: `src/components/layout/ShellTopTabs.vue`
- Create: `src/components/layout/ShellCollectionRail.vue`
- Create: `src/components/layout/ShellInspector.vue`
- Modify: `src/components/layout/AppShell.vue`
- Modify: `src/components/command/CommandPalette.vue`
- Modify: `src/styles/theme.css`
- Create: `tests/unit/workspaceModel.spec.ts`
- Create: `tests/unit/CommandPalette.spec.ts`
- Modify: `tests/unit/AppShell.spec.ts`
- Modify: `tests/e2e/smoke.spec.ts`

---

## Task 1: Workspace Model

**Files:**
- Create: `src/components/layout/workspaceModel.ts`
- Create: `tests/unit/workspaceModel.spec.ts`

### Step 1: Write the failing test

Create `tests/unit/workspaceModel.spec.ts`:

```ts
import { describe, expect, it } from 'vitest'
import {
  getWorkspaceCollections,
  resolveWorkspaceKey,
  workspaceMenuEntries,
  workspaceTabs,
} from '@/components/layout/workspaceModel'

describe('workspaceModel', () => {
  it('keeps app-level routes on the previous workspace', () => {
    expect(resolveWorkspaceKey('settings', 'event')).toBe('event')
    expect(resolveWorkspaceKey('help', 'relation')).toBe('relation')
    expect(resolveWorkspaceKey('diagnostics', 'project')).toBe('project')
  })

  it('exposes the Linear-style top tab order', () => {
    expect(workspaceTabs.map((tab) => tab.label)).toEqual([
      '项目',
      '资料',
      '角色',
      '事件',
      '规则',
      '关系',
      '分析',
    ])
    expect(workspaceMenuEntries.map((entry) => entry.label)).toEqual([
      '设置',
      '帮助',
      '诊断',
      '任务',
      'Prompt 模板',
    ])
  })

  it('returns the right collection rail presets for each workspace', () => {
    expect(getWorkspaceCollections('character').map((item) => item.label)).toEqual([
      '全部角色',
      '主角',
      '配角',
      '按阵营',
      '缺少动机',
      '缺少目标',
      '有成长记录',
      '最近编辑',
    ])
  })
})
```

### Step 2: Run it to confirm the failure

Run:

```powershell
pnpm test:unit -- tests/unit/workspaceModel.spec.ts
```

Expected: fail because `src/components/layout/workspaceModel.ts` does not exist yet.

### Step 3: Implement the model

Create `src/components/layout/workspaceModel.ts`:

```ts
export type WorkspaceKey =
  | 'project'
  | 'library'
  | 'character'
  | 'event'
  | 'rule'
  | 'relation'
  | 'analysis'

export interface WorkspaceTab {
  key: WorkspaceKey
  label: string
  to: string
  hint: string
}

export interface WorkspaceCollection {
  key: string
  label: string
  description: string
}

export interface WorkspaceMenuEntry {
  label: string
  to: string
}

const routeWorkspaceMap = new Map<string, WorkspaceKey>([
  ['home', 'project'],
  ['project', 'library'],
  ['search', 'analysis'],
  ['graph', 'relation'],
  ['timeline', 'event'],
  ['backup', 'project'],
  ['indexing', 'analysis'],
  ['audit', 'rule'],
  ['growth', 'character'],
  ['simulation', 'analysis'],
  ['agent', 'analysis'],
  ['jobs', 'analysis'],
  ['promptTemplates', 'analysis'],
])

const appLevelRouteNames = new Set(['settings', 'help', 'diagnostics'])

export const workspaceTabs: WorkspaceTab[] = [
  { key: 'project', label: '项目', to: '/', hint: '项目列表、导入导出、备份' },
  { key: 'library', label: '资料', to: '/projects', hint: '词条、地点、物品、阵营、资源' },
  { key: 'character', label: '角色', to: '/growth', hint: '角色档案、成长、事件经历' },
  { key: 'event', label: '事件', to: '/timeline', hint: '时间线、参与者、因果链' },
  { key: 'rule', label: '规则', to: '/audit', hint: '公理、法则、约束、例外' },
  { key: 'relation', label: '关系', to: '/graph', hint: '反链、图谱、连接、置信度' },
  { key: 'analysis', label: '分析', to: '/search', hint: '搜索、审计、模拟、任务' },
]

export const workspaceMenuEntries: WorkspaceMenuEntry[] = [
  { label: '设置', to: '/settings' },
  { label: '帮助', to: '/help' },
  { label: '诊断', to: '/diagnostics' },
  { label: '任务', to: '/jobs' },
  { label: 'Prompt 模板', to: '/prompt-templates' },
]

const collectionMap: Record<WorkspaceKey, WorkspaceCollection[]> = {
  project: [
    { key: 'recent', label: '最近项目', description: '最近打开或修改的项目' },
    { key: 'all', label: '全部项目', description: '完整项目列表' },
    { key: 'archived', label: '已归档', description: '归档项目' },
    { key: 'io', label: '导入导出', description: '导入、导出、迁移' },
    { key: 'backup', label: '备份', description: '本地备份与恢复' },
  ],
  library: [
    { key: 'all', label: '全部资料', description: '全部条目' },
    { key: 'entry', label: '词条', description: '世界观、地点、物品、阵营、资源' },
    { key: 'draft', label: '草稿', description: '尚未整理完成的资料' },
    { key: 'recent', label: '最近编辑', description: '最近修改过的资料' },
    { key: 'orphan', label: '缺少关系', description: '缺少关联的对象' },
  ],
  character: [
    { key: 'all', label: '全部角色', description: '全部角色档案' },
    { key: 'main', label: '主角', description: '主要叙事角色' },
    { key: 'support', label: '配角', description: '辅助叙事角色' },
    { key: 'faction', label: '按阵营', description: '按阵营聚合' },
    { key: 'motivation', label: '缺少动机', description: '动机信息不完整' },
    { key: 'goal', label: '缺少目标', description: '目标信息不完整' },
    { key: 'growth', label: '有成长记录', description: '已经关联成长记录的角色' },
    { key: 'recent', label: '最近编辑', description: '最近修改过的角色' },
  ],
  event: [
    { key: 'all', label: '全部事件', description: '全部事件记录' },
    { key: 'undated', label: '未定时间', description: '时间信息缺失' },
    { key: 'chapter', label: '按章节', description: '按章节聚合' },
    { key: 'location', label: '按地点', description: '按地点聚合' },
    { key: 'impact', label: '高影响事件', description: '重要度较高的事件' },
    { key: 'participant', label: '缺少参与者', description: '参与者信息不完整' },
    { key: 'recent', label: '最近编辑', description: '最近修改过的事件' },
  ],
  rule: [
    { key: 'all', label: '全部规则', description: '全部公理与约束' },
    { key: 'physics', label: '物理法则', description: '世界物理规则' },
    { key: 'social', label: '社会规则', description: '社会契约和制度' },
    { key: 'magic', label: '魔法规则', description: '超自然规则' },
    { key: 'exception', label: '例外', description: '局部豁免和例外条款' },
    { key: 'low-confidence', label: '低置信度', description: '需要进一步审视的规则' },
    { key: 'conflict', label: '冲突候选', description: '可能与其他规则冲突' },
  ],
  relation: [
    { key: 'all', label: '全部关系', description: '全部对象连接' },
    { key: 'person', label: '人物关系', description: '角色与角色之间的关系' },
    { key: 'causal', label: '因果关系', description: '事件与事件之间的因果连接' },
    { key: 'source', label: '来源关系', description: '对象来源和引用关系' },
    { key: 'constraint', label: '约束关系', description: '限制、依赖和约束' },
    { key: 'low-confidence', label: '低置信度', description: '不够稳定的连接' },
    { key: 'orphan', label: '孤立对象', description: '还没有形成有效连接的对象' },
  ],
  analysis: [
    { key: 'search', label: '搜索', description: '跨对象检索' },
    { key: 'audit', label: '审计', description: '逻辑一致性检查' },
    { key: 'simulation', label: '模拟', description: '结构化模拟结果' },
    { key: 'agent', label: 'Agent', description: '对话和分析上下文' },
    { key: 'indexing', label: '索引', description: '切片和向量索引状态' },
    { key: 'jobs', label: '任务', description: '后台队列与执行日志' },
    { key: 'report', label: '报告', description: '审计和模拟报告' },
  ],
}

export function resolveWorkspaceKey(
  routeName: string | symbol | undefined,
  fallback: WorkspaceKey = 'project',
): WorkspaceKey {
  if (typeof routeName !== 'string') {
    return fallback
  }
  if (appLevelRouteNames.has(routeName)) {
    return fallback
  }
  return routeWorkspaceMap.get(routeName) ?? fallback
}

export function getWorkspaceCollections(workspace: WorkspaceKey) {
  return collectionMap[workspace]
}

export function getWorkspaceSummary(workspace: WorkspaceKey) {
  switch (workspace) {
    case 'project':
      return '项目是工作入口。这里先决定要编辑哪一个宇宙，再进入对应资料。'
    case 'library':
      return '资料工作区聚焦词条与泛设定条目。右栏负责轻编辑，深内容进入二级界面。'
    case 'character':
      return '角色工作区聚焦档案、成长和经历。右栏用于摘要、标签和快速动作。'
    case 'event':
      return '事件工作区聚焦时间线与因果链。长叙事进入详情页。'
    case 'rule':
      return '规则工作区聚焦公理、约束和例外。审计和修复建议从这里进入。'
    case 'relation':
      return '关系工作区聚焦连接、反链和图谱。'
    case 'analysis':
      return '分析工作区聚焦搜索、审计、模拟、Agent 和任务。'
  }
}
```

### Step 4: Run the test again

Run:

```powershell
pnpm test:unit -- tests/unit/workspaceModel.spec.ts
```

Expected: pass.

### Step 5: Commit

```powershell
git add src/components/layout/workspaceModel.ts tests/unit/workspaceModel.spec.ts
git commit -m "feat: add workspace model"
```

---

## Task 2: Shell Components and AppShell

**Files:**
- Create: `src/components/layout/ShellTopTabs.vue`
- Create: `src/components/layout/ShellCollectionRail.vue`
- Create: `src/components/layout/ShellInspector.vue`
- Modify: `src/components/layout/AppShell.vue`
- Modify: `tests/unit/AppShell.spec.ts`

### Step 1: Write the failing test

Update `tests/unit/AppShell.spec.ts`:

```ts
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { createPinia } from 'pinia'
import App from '@/App.vue'
import { router } from '@/router'

describe('AppShell', () => {
  it('renders the workspace tabs, collection rail, and inspector scaffold', async () => {
    await router.push('/')
    await router.isReady()

    const wrapper = mount(App, {
      global: {
        plugins: [createPinia(), router],
      },
    })

    expect(wrapper.text()).toContain('Ameya')
    expect(wrapper.text()).toContain('项目')
    expect(wrapper.text()).toContain('资料')
    expect(wrapper.text()).toContain('Inspector')
    expect(wrapper.text()).toContain('任务空闲')
  })
})
```

### Step 2: Run it to confirm the failure

Run:

```powershell
pnpm test:unit -- tests/unit/AppShell.spec.ts
```

Expected: fail because the new shell components do not exist yet and the current layout still renders the old sidebar/context placeholder pattern.

### Step 3: Implement the shell components

Create `src/components/layout/ShellTopTabs.vue`:

```vue
<template>
  <nav class="workspace-tabs" aria-label="对象标签">
    <RouterLink
      v-for="tab in tabs"
      :key="tab.key"
      :to="tab.to"
      class="workspace-tab"
      :class="{ active: tab.key === activeWorkspace }"
    >
      <strong>{{ tab.label }}</strong>
      <span>{{ tab.hint }}</span>
    </RouterLink>
  </nav>
</template>

<script setup lang="ts">
import { workspaceTabs, type WorkspaceKey } from './workspaceModel'

defineProps<{
  activeWorkspace: WorkspaceKey
}>()

const tabs = workspaceTabs
</script>
```

Create `src/components/layout/ShellCollectionRail.vue`:

```vue
<template>
  <section class="collection-rail" aria-label="集合栏">
    <header class="collection-rail-header">
      <div>
        <p class="eyebrow">集合</p>
        <h2>{{ title }}</h2>
      </div>
      <button type="button" class="secondary-button">筛选</button>
    </header>

    <p class="collection-rail-summary">{{ summary }}</p>

    <nav class="collection-list">
      <button
        v-for="item in collections"
        :key="item.key"
        type="button"
        class="collection-item"
        :class="{ active: item.key === activeCollection }"
        @click="$emit('select', item.key)"
      >
        <strong>{{ item.label }}</strong>
        <span>{{ item.description }}</span>
      </button>
    </nav>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  getWorkspaceCollections,
  getWorkspaceSummary,
  type WorkspaceKey,
} from './workspaceModel'

const props = defineProps<{
  workspace: WorkspaceKey
  activeCollection: string
}>()

defineEmits<{
  (event: 'select', collectionKey: string): void
}>()

const collections = computed(() => getWorkspaceCollections(props.workspace))
const summary = computed(() => getWorkspaceSummary(props.workspace))
const title = computed(() => {
  switch (props.workspace) {
    case 'project':
      return '项目集合'
    case 'library':
      return '资料集合'
    case 'character':
      return '角色集合'
    case 'event':
      return '事件集合'
    case 'rule':
      return '规则集合'
    case 'relation':
      return '关系集合'
    case 'analysis':
      return '分析集合'
  }
})
</script>
```

Create `src/components/layout/ShellInspector.vue`:

```vue
<template>
  <section class="shell-inspector" aria-label="Inspector">
    <p class="eyebrow">Inspector</p>
    <h2>{{ title }}</h2>
    <p class="shell-inspector-summary">{{ summary }}</p>

    <section class="inspector-section">
      <h3>当前项目</h3>
      <p>{{ projectName }}</p>
    </section>

    <section class="inspector-section">
      <h3>当前集合</h3>
      <p>{{ collectionName }}</p>
    </section>

    <section class="inspector-section">
      <h3>快捷动作</h3>
      <RouterLink to="/search">全局搜索</RouterLink>
      <RouterLink to="/jobs">查看任务</RouterLink>
      <RouterLink to="/settings">打开设置</RouterLink>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { getWorkspaceSummary, type WorkspaceKey } from './workspaceModel'

const props = defineProps<{
  workspace: WorkspaceKey
  projectName: string
  collectionName: string
}>()

const title = computed(() => {
  switch (props.workspace) {
    case 'project':
      return '项目上下文'
    case 'library':
      return '资料上下文'
    case 'character':
      return '角色上下文'
    case 'event':
      return '事件上下文'
    case 'rule':
      return '规则上下文'
    case 'relation':
      return '关系上下文'
    case 'analysis':
      return '分析上下文'
  }
})

const summary = computed(() => getWorkspaceSummary(props.workspace))
</script>
```

Modify `src/components/layout/AppShell.vue`:

```vue
<template>
  <div class="app-shell">
    <header class="app-header">
      <div class="brand">
        <span class="brand-mark">A</span>
        <div>
          <strong>Ameya</strong>
          <small>{{ projectLabel }}</small>
        </div>
      </div>

      <ShellTopTabs :active-workspace="currentWorkspace" />

      <div class="header-actions">
        <RouterLink class="header-action" to="/search">搜索</RouterLink>
        <details class="shell-menu">
          <summary>更多</summary>
          <div class="shell-menu-panel">
            <RouterLink
              v-for="item in workspaceMenuEntries"
              :key="item.to"
              :to="item.to"
            >
              {{ item.label }}
            </RouterLink>
          </div>
        </details>
      </div>
    </header>

    <aside class="app-sidebar" aria-label="集合栏">
      <ShellCollectionRail
        :workspace="currentWorkspace"
        :active-collection="activeCollection"
        @select="selectCollection"
      />
    </aside>

    <main class="app-main">
      <RouterView />
    </main>

    <aside class="context-panel" aria-label="Inspector">
      <ShellInspector
        :workspace="currentWorkspace"
        :project-name="projectLabel"
        :collection-name="activeCollectionLabel"
      />
    </aside>

    <footer class="status-bar">
      <span>本地优先</span>
      <span>Windows</span>
      <JobStatusBar />
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import JobStatusBar from '@/components/jobs/JobStatusBar.vue'
import ShellCollectionRail from '@/components/layout/ShellCollectionRail.vue'
import ShellInspector from '@/components/layout/ShellInspector.vue'
import ShellTopTabs from '@/components/layout/ShellTopTabs.vue'
import {
  getWorkspaceCollections,
  resolveWorkspaceKey,
  workspaceMenuEntries,
  type WorkspaceKey,
} from './workspaceModel'
import { useProjectStore } from '@/stores/projectStore'

const route = useRoute()
const projectStore = useProjectStore()
const currentWorkspace = ref<WorkspaceKey>('project')
const activeCollections = reactive<Record<WorkspaceKey, string>>({
  project: 'recent',
  library: 'all',
  character: 'all',
  event: 'all',
  rule: 'all',
  relation: 'all',
  analysis: 'search',
})

watch(
  () => route.name,
  (name) => {
    currentWorkspace.value = resolveWorkspaceKey(name, currentWorkspace.value)
    const collections = getWorkspaceCollections(currentWorkspace.value)
    if (!collections.some((item) => item.key === activeCollections[currentWorkspace.value])) {
      activeCollections[currentWorkspace.value] = collections[0]?.key ?? ''
    }
  },
  { immediate: true },
)

const projectLabel = computed(() => projectStore.activeProject?.name ?? '选择或创建一个项目')

const activeCollection = computed(() => activeCollections[currentWorkspace.value])

const activeCollectionLabel = computed(() => {
  return (
    getWorkspaceCollections(currentWorkspace.value).find(
      (item) => item.key === activeCollections[currentWorkspace.value],
    )?.label ?? '未选择集合'
  )
})

function selectCollection(collectionKey: string) {
  activeCollections[currentWorkspace.value] = collectionKey
}
</script>
```

### Step 4: Run the shell test again

Run:

```powershell
pnpm test:unit -- tests/unit/AppShell.spec.ts
```

Expected: pass.

### Step 5: Commit

```powershell
git add src/components/layout/ShellTopTabs.vue src/components/layout/ShellCollectionRail.vue src/components/layout/ShellInspector.vue src/components/layout/AppShell.vue tests/unit/AppShell.spec.ts
git commit -m "feat: rebuild app shell chrome"
```

---

## Task 3: Linear Theme and Shell Layout

**Files:**
- Modify: `src/styles/theme.css`

### Step 1: Write the failing visual expectation

Update `tests/e2e/smoke.spec.ts`:

```ts
import { expect, test } from '@playwright/test'

test('opens the new object-workspace shell', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('link', { name: '项目' })).toBeVisible()
  await expect(page.getByRole('link', { name: '资料' })).toBeVisible()
  await expect(page.getByText('Inspector')).toBeVisible()
  await expect(page.getByText('任务空闲')).toBeVisible()
})
```

### Step 2: Run it to confirm the failure

Run:

```powershell
pnpm test:e2e -- tests/e2e/smoke.spec.ts
```

Expected: fail until the new shell CSS lands and the layout renders in the right regions.

### Step 3: Implement the style system

Update `src/styles/theme.css` with the shell layout and the new shell-specific classes:

```css
:root {
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
    sans-serif;
  color: #1f2328;
  background: #f5f6f8;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  letter-spacing: 0;
}

body {
  margin: 0;
  min-width: 960px;
  min-height: 100vh;
  background: #f5f6f8;
}

.app-shell {
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr) 320px;
  grid-template-rows: 56px minmax(0, 1fr) 32px;
  grid-template-areas:
    "header header header"
    "sidebar main inspector"
    "status status status";
  min-height: 100vh;
}

.app-header {
  grid-area: header;
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr) auto;
  align-items: center;
  gap: 16px;
  padding: 0 16px;
  border-bottom: 1px solid #e5e7eb;
  background: rgba(245, 246, 248, 0.92);
  backdrop-filter: blur(12px);
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.brand small {
  color: #6b7280;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-action,
.shell-menu summary,
.shell-menu-panel a {
  border-radius: 6px;
}

.workspace-tabs {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  overflow: auto;
}

.workspace-tab {
  display: grid;
  gap: 2px;
  min-width: 118px;
  padding: 8px 10px;
  border: 1px solid transparent;
  border-radius: 8px;
  color: #374151;
  text-decoration: none;
}

.workspace-tab strong,
.workspace-tab span {
  display: block;
  white-space: nowrap;
}

.workspace-tab span {
  color: #6b7280;
  font-size: 12px;
}

.workspace-tab.active {
  border-color: #cbd5e1;
  background: #ffffff;
  color: #111827;
}

.app-sidebar {
  grid-area: sidebar;
  border-right: 1px solid #e5e7eb;
  background: #fafbfc;
  overflow: auto;
}

.collection-rail {
  display: grid;
  gap: 12px;
  padding: 16px 14px;
}

.collection-rail-header,
.shell-inspector .inspector-section {
  display: grid;
  gap: 8px;
}

.collection-list {
  display: grid;
  gap: 6px;
}

.collection-item {
  display: grid;
  gap: 2px;
  padding: 9px 10px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  text-align: left;
}

.collection-item span {
  color: #6b7280;
  font-size: 12px;
}

.collection-item.active {
  border-color: #cbd5e1;
  background: #ffffff;
}

.app-main {
  grid-area: main;
  min-width: 0;
  overflow: auto;
  padding: 20px;
}

.context-panel {
  grid-area: inspector;
  border-left: 1px solid #e5e7eb;
  background: #fbfbfc;
  overflow: auto;
}

.shell-inspector {
  display: grid;
  gap: 14px;
  padding: 16px 14px;
}

.shell-inspector-summary {
  margin: 0;
  color: #6b7280;
}

.inspector-section {
  padding-top: 12px;
  border-top: 1px solid #e5e7eb;
}

.status-bar {
  grid-area: status;
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 0 14px;
  border-top: 1px solid #e5e7eb;
  background: #111827;
  color: #f3f4f6;
  font-size: 12px;
}
```

Keep the rest of the existing component-level styles unless they conflict with the new shell. Avoid reintroducing warm beige or saturated green as the dominant page color.

### Step 4: Run the e2e smoke test again

Run:

```powershell
pnpm test:e2e -- tests/e2e/smoke.spec.ts
```

Expected: pass.

### Step 5: Commit

```powershell
git add src/styles/theme.css tests/e2e/smoke.spec.ts
git commit -m "feat: restyle shell for object workspace"
```

---

## Task 4: Command Palette and App Menu Entries

**Files:**
- Modify: `src/components/command/CommandPalette.vue`
- Create: `tests/unit/CommandPalette.spec.ts`

### Step 1: Write the failing test

Create `tests/unit/CommandPalette.spec.ts`:

```ts
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import CommandPalette from '@/components/command/CommandPalette.vue'
import { router } from '@/router'

describe('CommandPalette', () => {
  it('shows workspace and app-level commands', async () => {
    await router.push('/')
    await router.isReady()

    const wrapper = mount(CommandPalette, {
      global: {
        plugins: [router],
      },
    })

    window.dispatchEvent(new KeyboardEvent('keydown', { ctrlKey: true, key: 'k' }))
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('项目')
    expect(wrapper.text()).toContain('资料')
    expect(wrapper.text()).toContain('设置')
    expect(wrapper.text()).toContain('Prompt 模板')

    wrapper.unmount()
  })
})
```

### Step 2: Run it to confirm the failure

Run:

```powershell
pnpm test:unit
```

Expected: fail until the command list is driven by the workspace model.

### Step 3: Reuse the workspace model inside the palette

Modify `src/components/command/CommandPalette.vue` so it reads the same tab and menu definitions as the shell:

```vue
<template>
  <div v-if="open" class="command-palette">
    <button v-for="command in commands" :key="command.to" type="button" @click="go(command.to)">
      {{ command.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { workspaceMenuEntries, workspaceTabs } from '@/components/layout/workspaceModel'

const router = useRouter()
const open = ref(false)
const commands = computed(() => [
  ...workspaceTabs.map((tab) => ({ label: tab.label, to: tab.to })),
  ...workspaceMenuEntries,
])

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
```

### Step 4: Run the test again

Run:

```powershell
pnpm test:unit
```

Expected: pass.

### Step 5: Commit

```powershell
git add src/components/command/CommandPalette.vue
git add tests/unit/CommandPalette.spec.ts
git commit -m "feat: unify command palette with shell model"
```

---

## Task 5: Final Verification and Handoff

**Files:**
- No new files required

### Step 1: Run the full verification set

Run:

```powershell
pnpm typecheck
pnpm test:unit
pnpm test:e2e
pnpm build
```

Expected: all pass.

### Step 2: Check the shell in the browser

Open the app and confirm all of the following are visible together:

- the `项目 / 资料 / 角色 / 事件 / 规则 / 关系 / 分析` tabs
- the narrow left collection rail
- the right Inspector scaffold
- the bottom job status bar

Confirm that the shell still renders on routes such as `/`, `/projects`, `/graph`, and `/settings`.

### Step 3: Commit the full slice

```powershell
git add src/components/layout/workspaceModel.ts src/components/layout/ShellTopTabs.vue src/components/layout/ShellCollectionRail.vue src/components/layout/ShellInspector.vue src/components/layout/AppShell.vue src/components/command/CommandPalette.vue src/styles/theme.css tests/unit/workspaceModel.spec.ts tests/unit/CommandPalette.spec.ts tests/unit/AppShell.spec.ts tests/e2e/smoke.spec.ts
git commit -m "feat: ship object workspace shell"
```

---

## Self-Review

- Every file in scope is named explicitly.
- The plan stays inside one shippable slice and leaves deep page splitting for UX-M2.
- The pure model layer is tested before shell composition.
- The shell components are introduced before the style rewrite.
- The command palette reuses the same tab/menu source of truth.
- The verification commands are concrete and match the current toolchain.

## Next Plan

After UX-M1 lands, write a second plan for UX-M2:

- split `src/views/ProjectView.vue` into object-aware workspaces
- move complex editing into secondary detail panes
- make the Inspector selection-driven instead of only shell-driven
- begin the object-specific list/detail architecture for 资料、角色、事件、规则、关系
