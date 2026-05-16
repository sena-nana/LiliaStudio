# Ameya 阶段上下文

## 当前阶段

- 当前分支：`codex/ameya-implementation`
- 已完成：M0 工程基线、M1 本地资料库、M2 可视化编辑、M3-M4 AI 与向量层、M5-M8 逻辑审计/角色成长/Agent/模拟/交付骨架
- 下一阶段：产品化增强和真实 Provider 联调

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

## M5-M8 已实现

- 逻辑审计：确定性事实冲突检测、最小冲突集合、规则型修复建议。
- 角色成长：事件影响 TraitDelta 和来源追踪的属性状态。
- 模拟：规则型结构化模拟报告。
- Commands/API/UI：审计、角色成长、模拟、Agent、诊断、帮助页面和工作流 store。
- 命令面板：`Ctrl+K` 打开基础命令入口。
- 诊断：本机版本、平台和数据库摘要。
- 测试与交付：Playwright 冒烟测试、Windows Tauri build 脚本、README 打包说明。

## M5-M8 验证结果

- `pnpm typecheck`: pass
- `pnpm test:unit`: pass, 7 files, 9 tests
- `cd src-tauri; cargo test`: pass, 19 tests
- `pnpm build`: pass
- `pnpm test:e2e`: pass, 1 test
- `pnpm tauri build`: pass, produced `src-tauri/target/release/bundle/nsis/Ameya_0.1.0_x64-setup.exe`

## 后续产品化目标

当前已完成全功能骨架。后续应优先增强：

1. OpenAI-compatible、Claude CLI、Codex CLI 的真实调用和错误分类。
2. 设置保存与 Windows Credential Manager 密钥存储。
3. 审计报告、模拟报告、角色成长记录的持久化详情页。
4. 图谱交互、时间线过滤和附件管理。
5. UI 视觉 QA、安装包签名、升级策略和大项目性能基准。

## 后续设计约束

- 智能工作流先提供无 AI 的规则型/结构化结果，AI 只增强说明。
- 审计和修复建议不得自动改写用户资料。
- 角色最终属性必须可追溯到事件影响记录。
- 模拟报告必须保存为本地报告，可复制比较。
- Windows 诊断导出必须脱敏密钥和完整 prompt。

