<template>
  <section class="view-stack">
    <header class="view-header">
      <div>
        <p class="eyebrow">帮助</p>
        <h1>Ameya 工作流</h1>
      </div>
    </header>
    <div class="tool-grid">
      <article v-for="item in helpItems" :key="item.title">
        <h2>{{ item.title }}</h2>
        <p>{{ item.body }}</p>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
const helpItems = [
  { title: '项目', body: '一个项目对应一个创作宇宙，所有资料都围绕项目组织。' },
  { title: '词条', body: '词条用于记录世界规则、物品、地点、阵营和资源。' },
  { title: '审计', body: '审计报告只提出冲突和建议，不会自动改写设定。' },
  { title: 'AI', body: 'AI Provider 是可选能力；没有配置时，本地资料库仍然可用。' },
]
</script>
