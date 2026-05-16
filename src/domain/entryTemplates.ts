export interface EntryTemplate {
  type: string
  label: string
  summary: string
  body: string
  tags: string[]
}

export const entryTemplates: EntryTemplate[] = [
  {
    type: 'world_rule',
    label: '世界规则',
    summary: '定义世界底层限制、社会契约或超自然逻辑。',
    body: '规则内容：\n适用范围：\n例外边界：\n对事件的约束：',
    tags: ['公理'],
  },
  {
    type: 'item',
    label: '物品',
    summary: '记录物品来源、材料、使用限制和关联规则。',
    body: '来源：\n制造者：\n材料：\n使用限制：\n关联规则：',
    tags: ['物品'],
  },
  {
    type: 'location',
    label: '地点',
    summary: '记录地理、资源、政治归属和历史事件。',
    body: '地理位置：\n资源：\n政治归属：\n历史事件：',
    tags: ['地点'],
  },
  {
    type: 'faction',
    label: '阵营',
    summary: '记录意识形态、法律、资源和敌友关系。',
    body: '意识形态：\n法律框架：\n资源：\n敌友关系：',
    tags: ['阵营'],
  },
]

export function getTemplate(type: string): EntryTemplate {
  return entryTemplates.find((template) => template.type === type) ?? entryTemplates[0]
}
