# Ameya 阶段上下文

## 当前阶段

- 当前分支：`codex/ameya-implementation`
- 已完成：M0 工程基线、M1 本地资料库、M2 可视化编辑、M3-M4 AI 与向量层
- 下一阶段：M5-M8 逻辑审计、角色成长、Agent、模拟与交付

## 固定开发规范

- 技术栈固定：Tauri 2 + Vue 3 + TypeScript + Rust + SQLite，只考虑 Windows。
- 前端业务调用必须经由 `src/api/*`，组件不得直接散落 `invoke`。
- Rust 侧使用 `rusqlite` 和显式迁移，测试不得依赖用户真实数据库。
- 每个阶段完成前必须运行 fresh verification：`pnpm typecheck`、`pnpm test:unit`、`cd src-tauri; cargo test`。涉及可运行应用时再补 `pnpm build` 或 `pnpm tauri build`。
- AI 功能必须可降级；无密钥、无 CLI 时本地资料库仍可使用。
- 不写入 API Key、用户内容、CLI 输出日志到 git。

## M0 已实现

- 前端工程：Vite、Vue、TypeScript、Pinia、Vue Router hash history、Vitest。
- 应用外壳：左侧导航、中间内容区、右侧上下文面板、底部状态栏。
- 前端 API 契约：`src/api/client.ts`、`src/api/errors.ts`、`src/api/health.ts`。
- Tauri/Rust 工程：`src-tauri`、health command、SQLite 连接和迁移系统。
- 测试基线：前端 AppShell/API health tests，Rust health serialization、migration idempotency、file DB migration tests。

## M0 验证结果

- `pnpm typecheck`: pass
- `pnpm test:unit`: pass, 2 files, 3 tests
- `cd src-tauri; cargo test`: pass, 3 tests

## M1 已实现

- SQLite schema 扩展：projects、entries、characters、events、event_participants、relations、axioms。
- Rust domain/repository：Project、Entry、Character、Event、Relation、Axiom。
- Tauri commands：Project CRUD，Entry/Character/Event/Axiom 创建与列表，Relation 创建与反链查询。
- 前端 API/store：Project store、Library store。
- UI：项目库支持创建/打开项目；工作台支持加载项目内词条、角色、事件、公理并创建基础记录。
- 测试：repository 集成测试覆盖 Project/Entry/Character/Event/Relation/Axiom；前端 store 测试覆盖 Project 和 Library。

## M1 验证结果

- `pnpm typecheck`: pass
- `pnpm test:unit`: pass, 4 files, 5 tests
- `cd src-tauri; cargo test`: pass, 9 tests
- `pnpm build`: pass

## M2 已实现

- Entry 模板：世界规则、物品、地点、阵营四类结构化模板。
- 搜索服务：跨 Entry、Character、Event、Axiom 的本地关键词搜索。
- 导入导出：项目 JSON archive 导出，导入时创建新项目副本。
- 视图：搜索、图谱、时间线、备份页面和导航入口。
- 后端 commands：`search_entities`、`export_project_archive`、`import_project_archive`。
- 测试：后端搜索/导入导出集成测试，前端 search store 测试。

## M2 验证结果

- `pnpm typecheck`: pass
- `pnpm test:unit`: pass, 5 files, 6 tests
- `cd src-tauri; cargo test`: pass, 11 tests
- `pnpm build`: pass

## M3-M4 已实现

- AI 基础：Provider 配置模型、OpenAI-compatible URL/响应解析、Claude/Codex CLI 默认模板、CLI 模板渲染和命令拆分。
- AI 任务：`ai_jobs` 表、创建 queued job、任务列表。
- Prompt 模板：内置逻辑审计、角色行为校验、叙事模拟模板。
- 向量基础：文本切片、embedding 存储、cosine similarity、本地 vector search。
- RAG：项目文本切片索引、context pack 预览。
- 前端：设置页 provider/prompt/job 展示，索引页重建 DocumentChunk。
- 测试：AI/CLI/OpenAI-compatible/vector 纯逻辑测试，RAG pipeline 测试，AI store 测试。

## M3-M4 验证结果

- `pnpm typecheck`: pass
- `pnpm test:unit`: pass, 6 files, 8 tests
- `cd src-tauri; cargo test`: pass, 16 tests
- `pnpm build`: pass

## M5-M8 目标

按路线图实现上层智能工作流和 Windows 交付：

1. 确定性逻辑冲突检测和 QuickXplain 风格 MUS。
2. 冲突修复建议与审计报告。
3. 角色事件影响、属性快照和成长热图。
4. 行为合理性校验、Agent 聊天、补完问题、叙事模拟和平行时空。
5. 附件、命令面板、诊断、帮助、Playwright 冒烟测试和 Windows 打包。

## M5-M8 设计约束

- 智能工作流先提供无 AI 的规则型/结构化结果，AI 只增强说明。
- 审计和修复建议不得自动改写用户资料。
- 角色最终属性必须可追溯到事件影响记录。
- 模拟报告必须保存为本地报告，可复制比较。
- Windows 诊断导出必须脱敏密钥和完整 prompt。

