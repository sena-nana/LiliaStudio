import { createRouter, createWebHashHistory } from "vue-router";
import HomeView from "@/views/HomeView.vue";
import ProjectView from "@/views/ProjectView.vue";
import SettingsView from "@/views/SettingsView.vue";
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
    { path: "/", name: "home", component: HomeView },
    { path: "/projects/:projectId?", name: "project", component: ProjectView },
    { path: "/search/:projectId?", name: "search", component: SearchView },
    { path: "/graph/:projectId?", name: "graph", component: GraphView },
    {
      path: "/timeline/:projectId?",
      name: "timeline",
      component: TimelineView,
    },
    { path: "/backup/:projectId?", name: "backup", component: BackupView },
    {
      path: "/indexing/:projectId?",
      name: "indexing",
      component: IndexingView,
    },
    { path: "/audit/:projectId?", name: "audit", component: AuditReportView },
    {
      path: "/growth/:projectId?",
      name: "growth",
      component: CharacterGrowthView,
    },
    {
      path: "/simulation/:projectId?",
      name: "simulation",
      component: SimulationView,
    },
    { path: "/agent/:projectId?", name: "agent", component: AgentChatView },
    { path: "/diagnostics", name: "diagnostics", component: DiagnosticsView },
    { path: "/jobs", name: "jobs", component: JobsView },
    {
      path: "/prompt-templates",
      name: "promptTemplates",
      component: PromptTemplateView,
    },
    { path: "/help", name: "help", component: HelpView },
    { path: "/settings", name: "settings", component: SettingsView },
  ],
});
