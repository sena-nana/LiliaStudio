import type { Component } from "vue";
import type { RouteLocationRaw, RouteRecordRaw } from "vue-router";

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

export type SettingsTabKey = "appearance" | "ai" | "cli" | "about";

export interface SettingsTab {
  key: SettingsTabKey;
  label: string;
  icon: Component;
  to: RouteLocationRaw;
}

export interface FeatureModule {
  routes?: RouteRecordRaw[];
  primaryNav?: SidebarNavItem[];
  groups?: SidebarGroup[];
  globalActions?: SidebarActionItem[];
  footerLinks?: SidebarFooterLink[];
}
