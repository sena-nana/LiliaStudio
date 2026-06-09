# Ameya 开发 Todo

## 当前状态

- 已完成到产品化 P7：Prompt 模板管理。
- 下一步从 `plans/Tauri-Vue-Windows开发路线图.md` 的 T024 开始复核。
- 若现有 M3-M4 的文本切片能力已满足 T024，记录对照结果后推进 T025 Embedding 生成与向量存储。

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

1. T024 文本切片 DocumentChunk
   - 复核现有切片实现是否覆盖 Entry、Character、Event、Axiom。
   - 若已满足路线图验收，记录对照结果并推进 T025。
   - 若未满足，补齐可复用 chunk 生成，不调用 embedding。

2. T025 Embedding 生成与向量存储
   - 复用 OpenAI-compatible Provider 和 DocumentChunk。
   - 生成并保存 embedding，不实现语义搜索排序 UI。

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
  - `yarn typecheck`
  - `yarn test:unit`
- Rust 逻辑、迁移、Provider 或持久化变更优先运行：
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo check --manifest-path src-tauri/Cargo.toml`
- 涉及路由、壳层布局、构建配置或用户关键路径时优先运行：
  - `yarn build`
  - `yarn test:e2e`
  - `yarn verify`

## 维护规则

- 每完成一个阶段，只更新本文件的当前状态、完成项、下一步和必要验证结果。
- 不再追加长篇历史流水；完整路线图继续以 `plans/Tauri-Vue-Windows开发路线图.md` 为准。
- 新阶段计划若需要记录实施细节，另建独立计划文件并在本 todo 中只保留入口和结论。
