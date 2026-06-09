import { Activity, ClipboardList, HelpCircle, Settings, Sparkles } from "lucide-vue-next";
import type { FeatureModule } from "./types";

export const toolsFeature: FeatureModule = {
  footerLinks: [
    { to: "/settings", label: "设置", icon: Settings },
    { to: "/help", label: "帮助", icon: HelpCircle },
    { to: "/diagnostics", label: "诊断", icon: Activity },
    { to: "/jobs", label: "任务", icon: ClipboardList },
    { to: "/prompt-templates", label: "提示词", title: "提示词模板", icon: Sparkles },
  ],
  routes: [
    {
      path: "diagnostics",
      name: "diagnostics",
      component: () => import("@/views/DiagnosticsView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "jobs",
      name: "jobs",
      component: () => import("@/views/JobsView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "prompt-templates",
      name: "promptTemplates",
      component: () => import("@/views/PromptTemplateView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "help",
      name: "help",
      component: () => import("@/views/HelpView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "settings",
      name: "settings",
      component: () => import("@/pages/Settings.vue"),
      meta: { sidebar: "settings", lockSidebar: true, returnable: false },
    },
  ],
};
