# Ameya 阶段上下文

## 当前阶段

- 当前分支：`codex/ameya-implementation`
- 已完成：M0 工程基线
- 下一阶段：M1 本地资料库

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

## M1 目标

按路线图实现无 AI 本地资料库：

1. Project CRUD：创建、列表、打开、重命名、归档。
2. Entry CRUD：世界观、物品、地点、阵营、资源、文化、技术、魔法、笔记。
3. Character CRUD：角色档案。
4. Event CRUD：事件和参与者关联。
5. Relation CRUD：跨实体关系和反链。
6. Axiom CRUD：公理/三元组编辑。
7. 前端页面接入本地数据库，基础导航可操作。

## M1 设计约束

- 数据库主键统一使用前端/Rust 生成的字符串 ID，避免 SQLite rowid 泄漏到 UI。
- 所有删除先软删除，保留 `deleted_at` 或 `archived_at`。
- 列表默认隐藏软删除/归档数据。
- 每个 repository 先写 Rust 测试，再实现。
- 前端 store 和 API 先写 Vitest mock 测试，再接组件。

