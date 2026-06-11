import type { AiProviderKind } from "@/types/ai";

const entityTypeLabels: Record<string, string> = {
  entry: "词条",
  character: "角色",
  event: "事件",
  axiom: "公理",
  relation: "关系",
};

const entryTypeLabels: Record<string, string> = {
  world_rule: "世界规则",
  item: "物品",
  location: "地点",
  faction: "阵营",
  report: "报告",
};

const entryStatusLabels: Record<string, string> = {
  draft: "草稿",
  active: "有效",
  archived: "已归档",
};

const jobStatusLabels: Record<string, string> = {
  queued: "排队中",
  running: "运行中",
  succeeded: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

const jobTypeLabels: Record<string, string> = {
  logicAudit: "逻辑审计",
  indexProject: "项目索引",
  promptRun: "提示词运行",
  agentChat: "Agent 问答",
};

const logLevelLabels: Record<string, string> = {
  info: "信息",
  warning: "警告",
  error: "错误",
};

const providerKindLabels: Record<string, string> = {
  openAiCompatible: "OpenAI 兼容接口",
  codexCli: "Codex 命令行",
  claudeCli: "Claude 命令行",
};

const platformLabels: Record<string, string> = {
  windows: "Windows",
  linux: "Linux",
  macos: "macOS",
};

const databaseLabels: Record<string, string> = {
  sqlite: "SQLite",
};

const conflictTypeLabels: Record<string, string> = {
  mutually_exclusive_axioms: "互斥公理",
};

function labelFrom(labels: Record<string, string>, value: string): string {
  return labels[value] ?? value;
}

export function entityTypeLabel(value: string): string {
  return labelFrom(entityTypeLabels, value);
}

export function entryTypeLabel(value: string): string {
  return labelFrom(entryTypeLabels, value);
}

export function entryStatusLabel(value: string): string {
  return labelFrom(entryStatusLabels, value);
}

export function jobStatusLabel(value: string): string {
  return labelFrom(jobStatusLabels, value);
}

export function jobTypeLabel(value: string): string {
  return labelFrom(jobTypeLabels, value);
}

export function logLevelLabel(value: string): string {
  return labelFrom(logLevelLabels, value);
}

export function providerKindLabel(value: AiProviderKind | string): string {
  return labelFrom(providerKindLabels, value);
}

export function platformLabel(value: string): string {
  return labelFrom(platformLabels, value);
}

export function databaseLabel(value: string): string {
  return labelFrom(databaseLabels, value);
}

export function conflictTypeLabel(value: string): string {
  return labelFrom(conflictTypeLabels, value);
}
