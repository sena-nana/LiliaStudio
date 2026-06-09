<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import {
  SETTINGS_TABS,
  SETTINGS_SECTIONS,
  normalizeSettingsTab,
} from "../config/appShell";
import "../styles/page.css";

const route = useRoute();
const activeTab = computed(() => normalizeSettingsTab(route.query.tab));
const activeTabSection = computed(() => SETTINGS_SECTIONS[activeTab.value]);
const activeTabLabel = computed(
  () => SETTINGS_TABS.find((tab) => tab.key === activeTab.value)?.label ?? "设置",
);
</script>

<template>
  <section class="settings-page">
    <div class="page-header">
      <div>
        <h1>{{ activeTabLabel }}</h1>
        <p>管理 Ameya 的外观、人工智能提供方、命令行提供方和应用信息。</p>
      </div>
    </div>

    <component :is="activeTabSection" />
  </section>
</template>
