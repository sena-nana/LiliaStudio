import { Bot, Info, Palette, Wand2 } from "@lucide/vue";
import { defineAsyncComponent, type Component } from "vue";
import {
  FEATURE_FOOTER_LINKS,
  FEATURE_GLOBAL_ACTIONS,
  FEATURE_GROUPS,
  FEATURE_PRIMARY_NAV,
  SIDEBAR_FOOTER_STATUS,
} from "@/features";
import type { SettingsTab, SettingsTabKey } from "@/features";
export type {
  SettingsTab,
  SettingsTabKey,
  SidebarActionItem,
  SidebarFooterLink,
  SidebarFooterStatus,
  SidebarGroup,
  SidebarNavItem,
} from "@/features";

export const APP_TITLE = "Ameya";

export const SIDEBAR_CONFIG = {
  widthStorageKey: "ameya.sidebarWidth",
  collapsedStorageKey: "ameya.sidebarCollapsed",
  minWidth: 180,
  maxWidth: 480,
  defaultWidth: 240,
} as const;

export const SIDEBAR_GLOBAL_ACTIONS = FEATURE_GLOBAL_ACTIONS;
export const SIDEBAR_NAV = FEATURE_PRIMARY_NAV;
export const SIDEBAR_GROUPS = FEATURE_GROUPS;
export const SIDEBAR_FOOTER_LINKS = FEATURE_FOOTER_LINKS;
export { SIDEBAR_FOOTER_STATUS };

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
