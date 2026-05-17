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
}

export interface WorkspaceCollection {
  key: string
  label: string
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
  { key: 'project', label: '项目', to: '/' },
  { key: 'library', label: '资料', to: '/projects' },
  { key: 'character', label: '角色', to: '/growth' },
  { key: 'event', label: '事件', to: '/timeline' },
  { key: 'rule', label: '规则', to: '/audit' },
  { key: 'relation', label: '关系', to: '/graph' },
  { key: 'analysis', label: '分析', to: '/search' },
]

export const workspaceMenuEntries: WorkspaceMenuEntry[] = [
  { label: '设置', to: '/settings' },
  { label: '帮助', to: '/help' },
  { label: '诊断', to: '/diagnostics' },
  { label: '任务', to: '/jobs' },
  { label: 'Prompt 模板', to: '/prompt-templates' },
]

const collectionMap: Readonly<Record<WorkspaceKey, ReadonlyArray<WorkspaceCollection>>> = {
  project: [
    { key: 'recent', label: '最近项目' },
    { key: 'all', label: '全部项目' },
    { key: 'archived', label: '已归档' },
    { key: 'io', label: '导入导出' },
    { key: 'backup', label: '备份' },
  ],
  library: [
    { key: 'all', label: '全部资料' },
    { key: 'entry', label: '词条' },
    { key: 'draft', label: '草稿' },
    { key: 'recent', label: '最近编辑' },
    { key: 'orphan', label: '缺少关系' },
  ],
  character: [
    { key: 'all', label: '全部角色' },
    { key: 'main', label: '主角' },
    { key: 'support', label: '配角' },
    { key: 'faction', label: '按阵营' },
    { key: 'motivation', label: '缺少动机' },
    { key: 'goal', label: '缺少目标' },
    { key: 'growth', label: '有成长记录' },
    { key: 'recent', label: '最近编辑' },
  ],
  event: [
    { key: 'all', label: '全部事件' },
    { key: 'undated', label: '未定时间' },
    { key: 'chapter', label: '按章节' },
    { key: 'location', label: '按地点' },
    { key: 'impact', label: '高影响事件' },
    { key: 'participant', label: '缺少参与者' },
    { key: 'recent', label: '最近编辑' },
  ],
  rule: [
    { key: 'all', label: '全部规则' },
    { key: 'physics', label: '物理法则' },
    { key: 'social', label: '社会规则' },
    { key: 'magic', label: '魔法规则' },
    { key: 'exception', label: '例外' },
    { key: 'low-confidence', label: '低置信度' },
    { key: 'conflict', label: '冲突候选' },
  ],
  relation: [
    { key: 'all', label: '全部关系' },
    { key: 'person', label: '人物关系' },
    { key: 'causal', label: '因果关系' },
    { key: 'source', label: '来源关系' },
    { key: 'constraint', label: '约束关系' },
    { key: 'low-confidence', label: '低置信度' },
    { key: 'orphan', label: '孤立对象' },
  ],
  analysis: [
    { key: 'search', label: '搜索' },
    { key: 'audit', label: '审计' },
    { key: 'simulation', label: '模拟' },
    { key: 'agent', label: 'Agent' },
    { key: 'indexing', label: '索引' },
    { key: 'jobs', label: '任务' },
    { key: 'report', label: '报告' },
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
  return collectionMap[workspace].map((item) => ({ ...item }))
}
