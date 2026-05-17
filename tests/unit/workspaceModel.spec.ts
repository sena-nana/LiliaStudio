import { describe, expect, it } from 'vitest'
import {
  getWorkspaceCollections,
  resolveWorkspaceKey,
  type WorkspaceKey,
  workspaceMenuEntries,
  workspaceTabs,
} from '@/components/layout/workspaceModel'

const workspaceKeys = [
  'project',
  'library',
  'character',
  'event',
  'rule',
  'relation',
  'analysis',
] as const satisfies readonly WorkspaceKey[]

describe('workspaceModel', () => {
  it('maps routes to workspaces and keeps app-level routes on the previous workspace', () => {
    expect(resolveWorkspaceKey('home')).toBe('project')
    expect(resolveWorkspaceKey('project')).toBe('library')
    expect(resolveWorkspaceKey('search')).toBe('analysis')
    expect(resolveWorkspaceKey('graph')).toBe('relation')
    expect(resolveWorkspaceKey('timeline')).toBe('event')
    expect(resolveWorkspaceKey('backup')).toBe('project')
    expect(resolveWorkspaceKey('indexing')).toBe('analysis')
    expect(resolveWorkspaceKey('audit')).toBe('rule')
    expect(resolveWorkspaceKey('growth')).toBe('character')
    expect(resolveWorkspaceKey('simulation')).toBe('analysis')
    expect(resolveWorkspaceKey('agent')).toBe('analysis')
    expect(resolveWorkspaceKey('jobs')).toBe('analysis')
    expect(resolveWorkspaceKey('promptTemplates')).toBe('analysis')
    expect(resolveWorkspaceKey('settings', 'event')).toBe('event')
    expect(resolveWorkspaceKey('help', 'relation')).toBe('relation')
    expect(resolveWorkspaceKey('diagnostics', 'project')).toBe('project')
  })

  it('exposes the Linear-style top tab order and paths', () => {
    expect(workspaceTabs).toEqual([
      { key: 'project', label: '项目', to: '/' },
      { key: 'library', label: '资料', to: '/projects' },
      { key: 'character', label: '角色', to: '/growth' },
      { key: 'event', label: '事件', to: '/timeline' },
      { key: 'rule', label: '规则', to: '/audit' },
      { key: 'relation', label: '关系', to: '/graph' },
      { key: 'analysis', label: '分析', to: '/search' },
    ])
  })

  it('exposes the app-level menu labels and paths', () => {
    expect(workspaceMenuEntries).toEqual([
      { label: '设置', to: '/settings' },
      { label: '帮助', to: '/help' },
      { label: '诊断', to: '/diagnostics' },
      { label: '任务', to: '/jobs' },
      { label: 'Prompt 模板', to: '/prompt-templates' },
    ])
  })

  it('returns the right collection rail presets for each workspace', () => {
    expect(getWorkspaceCollections('project').map((item) => item.label)).toEqual([
      '最近项目',
      '全部项目',
      '已归档',
      '导入导出',
      '备份',
    ])
    expect(getWorkspaceCollections('library').map((item) => item.label)).toEqual([
      '全部资料',
      '词条',
      '草稿',
      '最近编辑',
      '缺少关系',
    ])
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
    expect(getWorkspaceCollections('event').map((item) => item.label)).toEqual([
      '全部事件',
      '未定时间',
      '按章节',
      '按地点',
      '高影响事件',
      '缺少参与者',
      '最近编辑',
    ])
    expect(getWorkspaceCollections('rule').map((item) => item.label)).toEqual([
      '全部规则',
      '物理法则',
      '社会规则',
      '魔法规则',
      '例外',
      '低置信度',
      '冲突候选',
    ])
    expect(getWorkspaceCollections('relation').map((item) => item.label)).toEqual([
      '全部关系',
      '人物关系',
      '因果关系',
      '来源关系',
      '约束关系',
      '低置信度',
      '孤立对象',
    ])
    expect(getWorkspaceCollections('analysis').map((item) => item.label)).toEqual([
      '搜索',
      '审计',
      '模拟',
      'Agent',
      '索引',
      '任务',
      '报告',
    ])
  })

  it('does not expose shared collection item objects', () => {
    const collections = getWorkspaceCollections('project')
    collections[0].label = '已被外部修改'
    collections.push({ key: 'external', label: '外部项' })

    expect(getWorkspaceCollections('project')[0].label).toBe('最近项目')
    expect(getWorkspaceCollections('project').map((item) => item.label)).not.toContain('外部项')
  })

  it('does not expose explanatory metadata for shell labels', () => {
    workspaceKeys.forEach((workspace) => {
      getWorkspaceCollections(workspace).forEach((collection) => {
        expect(collection).toEqual({
          key: expect.any(String),
          label: expect.any(String),
        })
      })
    })

    workspaceTabs.forEach((tab) => {
      expect(tab).toEqual({
        key: expect.any(String),
        label: expect.any(String),
        to: expect.any(String),
      })
    })
  })
})
