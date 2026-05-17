<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">帮助</p>
        <h1>帮助</h1>
      </div>
    </header>
    <div class="tool-grid">
      <article v-for="item in helpItems" :key="item.title">
        <h2>{{ item.title }}</h2>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
const helpItems = [{ title: '项目' }, { title: '词条' }, { title: '审计' }, { title: 'AI' }]
</script>
