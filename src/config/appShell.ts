import {
  Activity,
  Bot,
  ClipboardList,
  FilePlus2,
  Folder,
  GitBranch,
  HelpCircle,
  Home,
  Info,
  Library,
  MoreHorizontal,
  Palette,
  Search,
  Settings,
  Sparkles,
  UserRound,
  Wand2,
} from "lucide-vue-next";
import { defineAsyncComponent, type Component } from "vue";
import type { RouteLocationRaw } from "vue-router";

export const APP_TITLE = "Ameya";

export const SIDEBAR_CONFIG = {
  widthStorageKey: "ameya.sidebarWidth",
  collapsedStorageKey: "ameya.sidebarCollapsed",
  minWidth: 180,
  maxWidth: 480,
  defaultWidth: 240,
} as const;

export interface SidebarActionItem {
  key: string;
  label: string;
  icon: Component;
  to?: string;
  disabled?: boolean;
}

export interface SidebarNavItem {
  to?: string;
  label: string;
  icon: Component;
  disabled?: boolean;
}

export interface SidebarGroup {
  title: string;
  tools?: SidebarActionItem[];
  items?: SidebarNavItem[];
  emptyText?: string;
}

export interface SidebarFooterLink {
  to: string;
  label: string;
  title?: string;
  icon: Component;
}

export interface SidebarFooterStatus {
  to: string;
  label: string;
  title: string;
  tone: "ok" | "warn" | "error";
  icon: Component;
}

export const SIDEBAR_GLOBAL_ACTIONS: SidebarActionItem[] = [
  { key: "new", label: "新建项目", icon: FilePlus2, to: "/projects" },
  { key: "search", label: "搜索", icon: Search, to: "/search" },
];

export const SIDEBAR_NAV: SidebarNavItem[] = [
  { to: "/", label: "项目", icon: Home },
  { to: "/projects", label: "资料库", icon: Library },
  { to: "/growth", label: "角色", icon: UserRound },
  { to: "/timeline", label: "事件", icon: Activity },
  { to: "/audit", label: "规则", icon: ClipboardList },
  { to: "/graph", label: "关系", icon: GitBranch },
  { to: "/search", label: "分析", icon: Search },
];

export const SIDEBAR_GROUPS: SidebarGroup[] = [
  {
    title: "项目工具",
    tools: [{ key: "more", label: "更多", icon: MoreHorizontal, disabled: true }],
    items: [
      { to: "/indexing", label: "索引", icon: Activity },
      { to: "/backup", label: "备份", icon: Folder },
      { to: "/simulation", label: "模拟", icon: Wand2 },
      { to: "/agent", label: "智能体", icon: Bot },
    ],
  },
];

export const SIDEBAR_FOOTER_LINKS: SidebarFooterLink[] = [
  { to: "/settings", label: "设置", icon: Settings },
  { to: "/help", label: "帮助", icon: HelpCircle },
  { to: "/diagnostics", label: "诊断", icon: Activity },
  { to: "/jobs", label: "任务", icon: ClipboardList },
  { to: "/prompt-templates", label: "提示词", title: "提示词模板", icon: Sparkles },
];

export const SIDEBAR_FOOTER_STATUS: SidebarFooterStatus = {
  to: "/jobs",
  label: "本地优先",
  title: "本地优先工作区。打开人工智能后台队列。",
  tone: "ok",
  icon: Sparkles,
};

export type SettingsTabKey = "appearance" | "ai" | "cli" | "about";

export interface SettingsTab {
  key: SettingsTabKey;
  label: string;
  icon: Component;
  to: RouteLocationRaw;
}

export const SETTINGS_TABS: SettingsTab[] = [
  { key: "appearance", label: "外观", icon: Palette, to: { path: "/settings", query: { tab: "appearance" } } },
  { key: "ai", label: "人工智能设置", icon: Bot, to: { path: "/settings", query: { tab: "ai" } } },
  { key: "cli", label: "命令行提供方", icon: Wand2, to: { path: "/settings", query: { tab: "cli" } } },
  { key: "about", label: "关于", icon: Info, to: { path: "/settings", query: { tab: "about" } } },
];

export const DEFAULT_SETTINGS_TAB: SettingsTabKey = "appearance";

export const SETTINGS_SECTIONS: Record<SettingsTabKey, Component> = {
  appearance: defineAsyncComponent(() => import("../pages/settings/AppearanceSection.vue")),
  ai: defineAsyncComponent(() => import("../pages/settings/AiSettingsSection.vue")),
  cli: defineAsyncComponent(() => import("../pages/settings/CliProvidersSection.vue")),
  about: defineAsyncComponent(() => import("../pages/settings/AboutSection.vue")),
};

export function normalizeSettingsTab(value: unknown): SettingsTabKey {
  const candidate = Array.isArray(value) ? value[0] : value;
  return SETTINGS_TABS.some((tab) => tab.key === candidate)
    ? (candidate as SettingsTabKey)
    : DEFAULT_SETTINGS_TAB;
}
