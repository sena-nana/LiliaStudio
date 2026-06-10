import { Sparkles } from "@lucide/vue";
import { homeFeature } from "./home";
import { toolsFeature } from "./tools";
import type {
  FeatureModule,
  SidebarActionItem,
  SidebarFooterLink,
  SidebarFooterStatus,
  SidebarGroup,
  SidebarNavItem,
} from "./types";
import { workspaceFeature } from "./workspace";

export const FEATURES: FeatureModule[] = [homeFeature, workspaceFeature, toolsFeature];

export const FEATURE_ROUTES = FEATURES.flatMap((feature) => feature.routes ?? []);

export const FEATURE_GLOBAL_ACTIONS: SidebarActionItem[] = FEATURES.flatMap(
  (feature) => feature.globalActions ?? [],
);

export const FEATURE_PRIMARY_NAV: SidebarNavItem[] = FEATURES.flatMap(
  (feature) => feature.primaryNav ?? [],
);

export const FEATURE_GROUPS: SidebarGroup[] = FEATURES.flatMap((feature) => feature.groups ?? []);

export const FEATURE_FOOTER_LINKS: SidebarFooterLink[] = FEATURES.flatMap(
  (feature) => feature.footerLinks ?? [],
);

export const SIDEBAR_FOOTER_STATUS: SidebarFooterStatus = {
  to: "/jobs",
  label: "本地优先",
  title: "本地优先工作区。打开人工智能后台队列。",
  tone: "ok",
  icon: Sparkles,
};

export type {
  SettingsTab,
  SettingsTabKey,
  SidebarActionItem,
  SidebarFooterLink,
  SidebarFooterStatus,
  SidebarGroup,
  SidebarNavItem,
} from "./types";
