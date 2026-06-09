import { createRouter, createWebHashHistory } from "vue-router";
import AppShell from "@/layouts/AppShell.vue";
import { FEATURE_ROUTES } from "@/features";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      component: AppShell,
      meta: { sidebar: "main", returnable: true },
      children: FEATURE_ROUTES,
    },
    { path: "/:pathMatch(.*)*", redirect: "/" },
  ],
});
