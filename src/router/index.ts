import { createRouter, createWebHashHistory } from "vue-router";
import AppShell from "@/layouts/AppShell.vue";
import HomeView from "@/views/HomeView.vue";
import ProjectView from "@/views/ProjectView.vue";
import SettingsPage from "@/pages/Settings.vue";
import SearchView from "@/views/SearchView.vue";
import GraphView from "@/views/GraphView.vue";
import TimelineView from "@/views/TimelineView.vue";
import BackupView from "@/views/BackupView.vue";
import IndexingView from "@/views/IndexingView.vue";
import AuditReportView from "@/views/AuditReportView.vue";
import CharacterGrowthView from "@/views/CharacterGrowthView.vue";
import SimulationView from "@/views/SimulationView.vue";
import AgentChatView from "@/views/AgentChatView.vue";
import DiagnosticsView from "@/views/DiagnosticsView.vue";
import JobsView from "@/views/JobsView.vue";
import PromptTemplateView from "@/views/PromptTemplateView.vue";
import HelpView from "@/views/HelpView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      component: AppShell,
      meta: { sidebar: "main", returnable: true },
      children: [
        { path: "", name: "home", component: HomeView, meta: { sidebar: "main", returnable: true } },
        { path: "projects/:projectId?", name: "project", component: ProjectView, meta: { sidebar: "main", returnable: true } },
        { path: "search/:projectId?", name: "search", component: SearchView, meta: { sidebar: "main", returnable: true } },
        { path: "graph/:projectId?", name: "graph", component: GraphView, meta: { sidebar: "main", returnable: true } },
        { path: "timeline/:projectId?", name: "timeline", component: TimelineView, meta: { sidebar: "main", returnable: true } },
        { path: "backup/:projectId?", name: "backup", component: BackupView, meta: { sidebar: "main", returnable: true } },
        { path: "indexing/:projectId?", name: "indexing", component: IndexingView, meta: { sidebar: "main", returnable: true } },
        { path: "audit/:projectId?", name: "audit", component: AuditReportView, meta: { sidebar: "main", returnable: true } },
        { path: "growth/:projectId?", name: "growth", component: CharacterGrowthView, meta: { sidebar: "main", returnable: true } },
        { path: "simulation/:projectId?", name: "simulation", component: SimulationView, meta: { sidebar: "main", returnable: true } },
        { path: "agent/:projectId?", name: "agent", component: AgentChatView, meta: { sidebar: "main", returnable: true } },
        { path: "diagnostics", name: "diagnostics", component: DiagnosticsView, meta: { sidebar: "main", returnable: true } },
        { path: "jobs", name: "jobs", component: JobsView, meta: { sidebar: "main", returnable: true } },
        { path: "prompt-templates", name: "promptTemplates", component: PromptTemplateView, meta: { sidebar: "main", returnable: true } },
        { path: "help", name: "help", component: HelpView, meta: { sidebar: "main", returnable: true } },
        { path: "settings", name: "settings", component: SettingsPage, meta: { sidebar: "settings", lockSidebar: true, returnable: false } },
      ],
    },
    { path: "/:pathMatch(.*)*", redirect: "/" },
  ],
});
