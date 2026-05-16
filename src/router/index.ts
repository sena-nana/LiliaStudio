import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import ProjectView from '@/views/ProjectView.vue'
import SettingsView from '@/views/SettingsView.vue'
import SearchView from '@/views/SearchView.vue'
import GraphView from '@/views/GraphView.vue'
import TimelineView from '@/views/TimelineView.vue'
import BackupView from '@/views/BackupView.vue'
import IndexingView from '@/views/IndexingView.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/projects/:projectId?', name: 'project', component: ProjectView },
    { path: '/search/:projectId?', name: 'search', component: SearchView },
    { path: '/graph/:projectId?', name: 'graph', component: GraphView },
    { path: '/timeline/:projectId?', name: 'timeline', component: TimelineView },
    { path: '/backup/:projectId?', name: 'backup', component: BackupView },
    { path: '/indexing/:projectId?', name: 'indexing', component: IndexingView },
    { path: '/settings', name: 'settings', component: SettingsView },
  ],
})
