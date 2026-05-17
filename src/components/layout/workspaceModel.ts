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

const collectionMap: Readonly<Record<WorkspaceKey, ReadonlyArray<WorkspaceCollection>>> = {
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
  return collectionMap[workspace].map((item) => ({ ...item }))
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
