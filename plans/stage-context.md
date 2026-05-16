# Ameya 阶段上下文

## 当前阶段

- 当前分支：`codex/ameya-implementation`
- 已完成：M0 工程基线、M1 本地资料库、M2 可视化编辑
- 下一阶段：M3-M4 AI 与向量层

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

## M3-M4 目标

按路线图实现 AI Provider、任务队列、向量/RAG：

1. 设置页和本机 provider 配置。
2. OpenAI-compatible HTTP provider。
3. CLI provider 抽象、Codex CLI、Claude CLI。
4. AI job 队列与任务日志。
5. Prompt 模板管理。
6. DocumentChunk、Embedding、本地向量相似度检索。
7. RAG 上下文包和叙事共鸣预警。

## M3-M4 设计约束

- AI Provider 必须可降级；未配置时不影响本地资料库。
- API Key 不进入导出、日志或 git。
- CLI 调用必须通过进程参数/模板解析，不用 shell 字符串拼接。
- 向量检索先用本地 SQLite 存储和内存 cosine，接口保留替换空间。
- 每个 AI 输出都保留 provider、输入摘要、状态和错误码。

