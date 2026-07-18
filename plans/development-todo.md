# Ameya 开发 Todo

## 当前状态

- 已完成到 M4：T024 文本切片 DocumentChunk。
- T024 复核结论：原实现已有 Entry chunk 基础能力，但未覆盖 Character、Event、Axiom，也未返回 token 估算和更新时间。
- T024 验收结论：`rag_index_chunks` 现已覆盖 Entry、Character、Event、Axiom；chunk 记录 source type/id、文本、hash、token 估算、更新时间；内容未变化时不重复更新时间；流程不调用 embedding。
- 下一步推进 T025 Embedding 生成与向量存储。

## 固定开发约束

- 技术栈固定：Tauri 2 + Vue 3 + TypeScript + Rust + SQLite，只考虑 Windows。
- 前端业务调用必须经由 `src/api/*`，组件不得直接散落 `invoke`。
- Rust 侧使用 `rusqlite` 和显式迁移，测试不得依赖用户真实数据库。
- AI 功能必须可降级；无密钥、无 CLI 时本地资料库仍可使用。
- 不写入 API Key、用户内容、CLI 输出日志到 git。
- 审计和修复建议不得自动改写用户资料。
- 角色最终属性必须可追溯到事件影响记录。
- 模拟报告必须保存为本地报告，可复制比较。
- Windows 诊断导出必须脱敏密钥和完整 prompt。

## 开发 Todo

1. T024 文本切片 DocumentChunk（已完成）
   - 已补齐 Entry、Character、Event、Axiom 的可复用 chunk 生成。
   - 已补齐 token 估算、更新时间返回、未变化内容不重复更新、软删除来源清理。
   - 已验证不调用 embedding，索引测试中 `embeddings` 表保持为空。

2. T025 Embedding 生成与向量存储
   - 复用 OpenAI-compatible Provider 和 DocumentChunk。
   - 生成并保存 embedding，不实现语义搜索排序 UI。

## 最近验证

- 2026-06-10 T024：
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `pnpm test:unit`
  - `pnpm typecheck`

3. T026 语义检索
   - 基于本地向量存储实现相似度检索。
   - 保持无密钥或 Provider 不可用时可降级。

4. T027 RAG 上下文包构建器
   - 合并实体详情、关系、Axiom 和向量搜索结果。
   - 每条上下文保留 source 引用，便于报告解释。

5. T028 上下文预览与索引体验
   - 提供可解释的上下文包预览和索引状态反馈。
   - 不在预览流程中调用 LLM。

## 验证策略

- 文档、注释、配置说明等低风险改动可不跑测试，但最终说明需写清楚原因。
- 前端类型或 store/API 变更优先运行：
  - `pnpm typecheck`
  - `pnpm test:unit`
- Rust 逻辑、迁移、Provider 或持久化变更优先运行：
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo check --manifest-path src-tauri/Cargo.toml`
- 涉及路由、壳层布局、构建配置或用户关键路径时优先运行：
  - `pnpm build`
  - `pnpm test:e2e`
  - `pnpm verify`

## 维护规则

- 每完成一个阶段，只更新本文件的当前状态、完成项、下一步和必要验证结果。
- 不再追加长篇历史流水；完整路线图继续以 `plans/Tauri-Vue-Windows开发路线图.md` 为准。
- 新阶段计划若需要记录实施细节，另建独立计划文件并在本 todo 中只保留入口和结论。
