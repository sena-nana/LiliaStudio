import {
  Activity,
  Bot,
  ClipboardList,
  FilePlus2,
  Folder,
  GitBranch,
  Library,
  MoreHorizontal,
  Search,
  UserRound,
  Wand2,
} from "lucide-vue-next";
import type { FeatureModule } from "./types";

export const workspaceFeature: FeatureModule = {
  globalActions: [
    { key: "new", label: "新建项目", icon: FilePlus2, to: "/projects" },
    { key: "search", label: "搜索", icon: Search, to: "/search" },
  ],
  primaryNav: [
    { to: "/projects", label: "资料库", icon: Library },
    { to: "/growth", label: "角色", icon: UserRound },
    { to: "/timeline", label: "事件", icon: Activity },
    { to: "/audit", label: "规则", icon: ClipboardList },
    { to: "/graph", label: "关系", icon: GitBranch },
    { to: "/search", label: "分析", icon: Search },
  ],
  groups: [
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
  ],
  routes: [
    {
      path: "projects/:projectId?",
      name: "project",
      component: () => import("@/views/ProjectView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "search/:projectId?",
      name: "search",
      component: () => import("@/views/SearchView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "graph/:projectId?",
      name: "graph",
      component: () => import("@/views/GraphView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "timeline/:projectId?",
      name: "timeline",
      component: () => import("@/views/TimelineView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "backup/:projectId?",
      name: "backup",
      component: () => import("@/views/BackupView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "indexing/:projectId?",
      name: "indexing",
      component: () => import("@/views/IndexingView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "audit/:projectId?",
      name: "audit",
      component: () => import("@/views/AuditReportView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "growth/:projectId?",
      name: "growth",
      component: () => import("@/views/CharacterGrowthView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "simulation/:projectId?",
      name: "simulation",
      component: () => import("@/views/SimulationView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
    {
      path: "agent/:projectId?",
      name: "agent",
      component: () => import("@/views/AgentChatView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
  ],
};
