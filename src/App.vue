<template>
  <RouterView />
  <CommandPalette />
  <ContextMenuHost />
</template>

<script setup lang="ts">
import { defineAsyncComponent } from "vue";
import { RouterView } from "vue-router";
import CommandPalette from "./components/command/CommandPalette.vue";

const ContextMenuHost = defineAsyncComponent(() => import("./components/ContextMenuHost.vue"));
</script>
