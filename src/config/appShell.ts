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
  { key: "new", label: "New project", icon: FilePlus2, to: "/projects" },
  { key: "search", label: "Search", icon: Search, to: "/search" },
];

export const SIDEBAR_NAV: SidebarNavItem[] = [
  { to: "/", label: "Projects", icon: Home },
  { to: "/projects", label: "Library", icon: Library },
  { to: "/growth", label: "Characters", icon: UserRound },
  { to: "/timeline", label: "Events", icon: Activity },
  { to: "/audit", label: "Rules", icon: ClipboardList },
  { to: "/graph", label: "Relations", icon: GitBranch },
  { to: "/search", label: "Analysis", icon: Search },
];

export const SIDEBAR_GROUPS: SidebarGroup[] = [
  {
    title: "Project tools",
    tools: [{ key: "more", label: "More", icon: MoreHorizontal, disabled: true }],
    items: [
      { to: "/indexing", label: "Indexing", icon: Activity },
      { to: "/backup", label: "Backup", icon: Folder },
      { to: "/simulation", label: "Simulation", icon: Wand2 },
      { to: "/agent", label: "Agent", icon: Bot },
    ],
  },
];

export const SIDEBAR_FOOTER_LINKS: SidebarFooterLink[] = [
  { to: "/settings", label: "Settings", icon: Settings },
  { to: "/help", label: "Help", icon: HelpCircle },
  { to: "/diagnostics", label: "Diagnostics", icon: Activity },
  { to: "/jobs", label: "Jobs", icon: ClipboardList },
  { to: "/prompt-templates", label: "Prompt", title: "Prompt templates", icon: Sparkles },
];

export const SIDEBAR_FOOTER_STATUS: SidebarFooterStatus = {
  to: "/jobs",
  label: "Local first",
  title: "Local-first workspace. Open the AI job queue.",
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
  { key: "appearance", label: "Appearance", icon: Palette, to: { path: "/settings", query: { tab: "appearance" } } },
  { key: "ai", label: "AI Settings", icon: Bot, to: { path: "/settings", query: { tab: "ai" } } },
  { key: "cli", label: "CLI Provider", icon: Wand2, to: { path: "/settings", query: { tab: "cli" } } },
  { key: "about", label: "About", icon: Info, to: { path: "/settings", query: { tab: "about" } } },
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
