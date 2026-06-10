import { Home } from "@lucide/vue";
import type { FeatureModule } from "./types";

export const homeFeature: FeatureModule = {
  routes: [
    {
      path: "",
      name: "home",
      component: () => import("@/views/HomeView.vue"),
      meta: { sidebar: "main", returnable: true },
    },
  ],
  primaryNav: [{ to: "/", label: "项目", icon: Home }],
};
