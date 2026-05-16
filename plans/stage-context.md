# Ameya 阶段上下文

## 当前阶段

- 当前分支：`codex/ameya-implementation`
- 已完成：M0 工程基线、M1 本地资料库
- 下一阶段：M2 可视化编辑

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

## M2 目标

按路线图实现可视化编辑和基础资料流转：

1. Entry 模板与结构化创建表单。
2. 全局关键词搜索。
3. 关系反链面板和基础关系编辑。
4. 图谱视图和时间线视图。
5. 项目 JSON 导入导出与备份。

## M2 设计约束

- 搜索和图谱必须无 AI 可用。
- 图谱默认加载邻域，不一次渲染全项目。
- 导入必须创建新项目，不覆盖现有数据。
- 导出不得包含密钥、日志或完整 AI prompt。
- UI 控件保持工作台风格，避免营销页和大面积装饰。

