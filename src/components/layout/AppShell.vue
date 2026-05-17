<template>
  <div class="app-shell">
    <aside class="app-sidebar" aria-label="主导航">
      <div class="brand">
        <span class="brand-mark">A</span>
        <div>
          <strong>Ameya</strong>
          <small>雨夜世界观整理</small>
        </div>
      </div>
      <nav class="nav-list">
        <RouterLink to="/">项目</RouterLink>
        <RouterLink to="/projects">工作台</RouterLink>
        <RouterLink to="/search">搜索</RouterLink>
        <RouterLink to="/graph">图谱</RouterLink>
        <RouterLink to="/timeline">时间线</RouterLink>
        <RouterLink to="/backup">备份</RouterLink>
        <RouterLink to="/indexing">索引</RouterLink>
        <RouterLink to="/audit">审计</RouterLink>
        <RouterLink to="/growth">成长</RouterLink>
        <RouterLink to="/simulation">模拟</RouterLink>
        <RouterLink to="/agent">Agent</RouterLink>
        <RouterLink to="/diagnostics">诊断</RouterLink>
        <RouterLink to="/jobs">任务</RouterLink>
        <RouterLink to="/help">帮助</RouterLink>
        <RouterLink to="/settings">设置</RouterLink>
      </nav>
    </aside>

    <main class="app-main">
      <RouterView />
    </main>

    <aside class="context-panel" aria-label="上下文">
      <h2>上下文</h2>
      <p>选择项目或实体后，这里显示反链、任务和审计摘要。</p>
    </aside>

    <footer class="status-bar">
      <span>本地优先</span>
      <span>Windows</span>
      <JobStatusBar />
    </footer>
  </div>
</template>

<script setup lang="ts">
import JobStatusBar from '@/components/jobs/JobStatusBar.vue'
</script>
