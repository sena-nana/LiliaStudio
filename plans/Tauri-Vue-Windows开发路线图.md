# Ameya Tauri + Vue Windows 开发路线图

> **For agentic workers:** REQUIRED SUB-SKILL: 实现本路线图中的任一任务时，使用 `superpowers:subagent-driven-development` 或 `superpowers:executing-plans`。每个任务都按可独立分发给 Claude 或 Codex 的粒度编写，执行时只改任务列出的范围。

**Goal:** 将 `plans/世界观人设工具功能架构设计.md` 中的逻辑驱动世界观、人设、向量检索、Agent 审计与模拟功能，落地为只面向 Windows 的 Tauri + Vue 桌面应用。

**Architecture:** Ameya 是本地优先的单用户桌面工具。Vue 负责编辑、检索、图谱、时间线和审计报告 UI；Tauri/Rust 负责 SQLite 持久化、向量索引、AI Provider、CLI 无头调用、后台任务和 Windows 文件系统集成。

**Tech Stack:** Tauri 2、Vue 3、TypeScript、Vite、Pinia、Vue Router Hash History、Rust、SQLite、rusqlite/sqlx 二选一但全仓保持一致、Vitest、Cargo Test、Playwright。AI Provider 支持 OpenAI-compatible HTTP API、Claude Code CLI headless、Codex CLI headless。

---

## 0. 硬约束与解释

- 只考虑 Windows 桌面运行，不做 macOS、Linux、移动端适配。
- 需求中的 `openid兼容api` 按 `OpenAI-compatible API` 理解：用户可配置 `base_url`、`api_key`、`chat_model`、`embedding_model`、自定义请求头。如果实际是 OpenID Connect 登录认证，应另开认证专项。
- Claude 和 Codex 不作为内嵌 SDK，统一走本机 CLI 无头交互能力：
  - Claude 默认命令模板：`claude -p "{prompt}" --output-format json --cwd "{workspace}" --max-turns {max_turns}`
  - Codex 默认命令模板：`codex exec --cd "{workspace}" --ask-for-approval never --sandbox workspace-write "{prompt}"`
  - 两者都必须允许用户在设置页覆盖完整命令模板，避免 CLI 版本变化导致核心业务失效。
- API Key 与 CLI 路径配置只保存在本机。优先使用 Windows Credential Manager；如果首期实现复杂，可先使用 Tauri app config 目录中的加密/权限受限配置文件，并在任务验收中标注风险。
- 不引入外部 Neo4j、Qdrant、PostgreSQL 服务。世界观图谱、向量、结构化数据统一本地 SQLite 持久化；后续可在存储接口稳定后替换为专用引擎。
- 所有 AI 功能必须可降级：无 API Key、无 Claude CLI、无 Codex CLI 时，核心资料编辑、搜索、图谱、导入导出仍可使用。

## 1. 目标信息架构

### 1.1 核心对象

- `Project`: 一个创作宇宙或作品项目。
- `Entry`: 通用词条，覆盖世界观设定、物品、地点、阵营、资源、技术、魔法、文化。
- `Character`: 角色实体，包含基础档案、经历、关系、属性演化。
- `Event`: 事件实体，绑定时间、地点、参与角色、因果关系。
- `Axiom`: 叙事公理或规则，以原子命题/三元组形式存储。
- `Relation`: 任意对象之间的边，支持因果、从属、来源、敌对、依赖、矛盾、相似。
- `DocumentChunk`: 可向量化的文本片段，来自 Entry、Character、Event、Axiom。
- `Embedding`: chunk 的向量、模型、维度、元数据。
- `AuditReport`: AI 或规则引擎产出的逻辑审计、冲突诊断、修复建议、模拟报告。
- `AiJob`: 后台 AI 任务，记录 provider、prompt、输入上下文、状态、日志和结果。

### 1.2 推荐目录结构

```text
ameya/
  package.json
  vite.config.ts
  tsconfig.json
  src/
    main.ts
    App.vue
    router/index.ts
    stores/
    api/
    views/
    components/
    styles/
    types/
  src-tauri/
    tauri.conf.json
    Cargo.toml
    migrations/
    src/
      lib.rs
      main.rs
      commands/
      db/
      domain/
      ai/
      vector/
      logic/
      jobs/
      windows/
  tests/
  plans/
```

### 1.3 每个任务的通用完成标准

- 任务完成后，相关命令可以运行，失败时必须在交付说明中写明失败原因。
- Rust 侧修改至少通过 `cd src-tauri; cargo test`。
- 前端修改至少通过 `pnpm typecheck` 和 `pnpm test:unit`。
- 涉及 UI 的任务需要给出可手测路径；后续 Playwright 建好后补自动化。
- 不把 API Key、用户内容、CLI 输出日志写入 git。
- 不跨任务重构，除非任务明确要求。

## 2. 里程碑

| 里程碑 | 目标 | 完成标志 |
| --- | --- | --- |
| M0 工程基线 | Tauri + Vue + Windows 开发环境可运行 | `pnpm tauri dev` 能打开空壳应用 |
| M1 本地资料库 | 项目、词条、角色、事件、公理、关系可 CRUD | 无 AI 也能作为世界观整理工具使用 |
| M2 可视化编辑 | 编辑器、搜索、反链、图谱、时间线可用 | 用户能浏览复杂设定关系 |
| M3 AI Provider | OpenAI-compatible API、Claude CLI、Codex CLI 三类 Provider 可配置和调用 | 同一 prompt 可走不同 Provider |
| M4 向量/RAG | 文本切片、embedding、语义检索、上下文包可用 | Agent 能检索项目资料回答 |
| M5 逻辑一致性 | 公理、冲突检测、MUS 诊断、修复建议可用 | 新设定能触发明确冲突报告 |
| M6 角色成长 | 事件驱动角色属性提取、快照、热图可用 | 角色成长由经历自动分析 |
| M7 Agent 工作流 | 逻辑审计、补完问题、情景模拟、平行时空报告可用 | 用户能让系统解释矛盾和推演后果 |
| M8 Windows 交付 | 安装包、日志、备份、崩溃诊断文档齐备 | Windows 机器可安装使用 |

## 3. 可分发任务清单

### T001 初始化 Tauri + Vue 工程

**依赖:** 无

**范围:** 创建 Tauri 2 + Vue 3 + TypeScript + Vite 工程骨架，只保留 Windows 目标。

**建议文件:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`

**验收:**
- `pnpm install` 成功。
- `pnpm tauri dev` 能打开窗口。
- `pnpm typecheck` 成功。
- `cd src-tauri; cargo test` 成功。

**分发提示词:**

```text
阅读 plans/Tauri-Vue-Windows开发路线图.md 的 T001。请在当前仓库初始化 Tauri 2 + Vue 3 + TypeScript + Vite 工程，只考虑 Windows。只实现工程骨架和基础运行脚本，不实现业务功能。完成后运行 pnpm typecheck、cd src-tauri; cargo test，并说明结果。
```

### T002 建立开发脚本与质量门禁

**依赖:** T001

**范围:** 增加统一脚本、格式化、lint、类型检查、Rust 测试命令。

**建议文件:**
- Modify: `package.json`
- Create: `.editorconfig`
- Create: `.gitignore`
- Create: `src-tauri/rustfmt.toml`
- Create: `README.md`

**验收:**
- `pnpm check` 串行执行前端 typecheck/test 和 Rust test。
- `.gitignore` 排除 `node_modules`、`dist`、`src-tauri/target`、本地数据库、日志、密钥文件。
- README 写明 Windows 依赖：Microsoft C++ Build Tools、WebView2、Rust MSVC、Node LTS、pnpm。

**分发提示词:**

```text
阅读 T002。请为现有 Tauri + Vue 工程补齐 Windows 开发脚本、README 和忽略规则。不要实现业务功能。pnpm check 必须能调用前端类型检查、单元测试和 cargo test。
```

### T003 应用外壳、路由和基础布局

**依赖:** T001

**范围:** 实现桌面应用主布局：左侧项目/导航栏、中间内容区、右侧上下文面板、底部任务状态栏。

**建议文件:**
- Modify: `src/App.vue`
- Create: `src/router/index.ts`
- Create: `src/views/HomeView.vue`
- Create: `src/views/ProjectView.vue`
- Create: `src/views/SettingsView.vue`
- Create: `src/components/layout/AppShell.vue`
- Create: `src/styles/theme.css`

**验收:**
- 使用 Vue Router `createWebHashHistory()`。
- 三个路由可切换：主页、项目、设置。
- 文字在 1280x720 和 1920x1080 下不重叠。
- 无营销式落地页，启动后直接进入工具界面。

**分发提示词:**

```text
阅读 T003。请实现 Ameya 桌面应用外壳、hash 路由和基础布局。只做结构和少量占位状态，不接数据库。确保 Windows Tauri WebView 下可用，完成后运行 pnpm typecheck。
```

### T004 Tauri 命令契约与前端 API 封装

**依赖:** T001, T003

**范围:** 建立前后端 IPC 规范，所有业务从 `src/api` 调用 Rust command，不在组件里直接散落 invoke。

**建议文件:**
- Create: `src/api/client.ts`
- Create: `src/api/errors.ts`
- Create: `src/types/result.ts`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/health.rs`
- Modify: `src-tauri/src/lib.rs`

**验收:**
- 前端 `api.healthCheck()` 调用 Rust `health_check` 返回版本、平台、app 数据目录。
- 前端有统一错误类型和显示文案。
- Vitest 使用 Tauri mock IPC 测试 `api.healthCheck()`。

**分发提示词:**

```text
阅读 T004。请建立 Tauri command 的前后端调用规范，增加 health_check 示例、TypeScript API wrapper 和 Vitest mock 测试。不要接入数据库或 AI。
```

### T005 SQLite 数据库初始化与迁移系统

**依赖:** T004

**范围:** Rust 后端在 Tauri app data 目录创建 SQLite 数据库，并支持版本化迁移。

**建议文件:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/connection.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Create: `src-tauri/migrations/0001_init.sql`
- Modify: `src-tauri/src/lib.rs`

**验收:**
- App 启动时创建 `ameya.db`。
- `schema_migrations` 记录已执行迁移。
- 重复启动不会重复执行迁移。
- Rust 测试使用临时目录数据库验证迁移幂等。

**分发提示词:**

```text
阅读 T005。请实现 Rust 侧 SQLite 初始化和迁移系统，数据库位于 Tauri app data 目录。只实现基础连接、迁移表和 0001_init.sql，不创建完整业务表。运行 cargo test。
```

### T006 项目 Project 模型与 CRUD

**依赖:** T005

**范围:** 支持创建、打开、重命名、归档项目。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/project.rs`
- Create: `src-tauri/src/commands/projects.rs`
- Create: `src/api/projects.ts`
- Create: `src/stores/projectStore.ts`
- Modify: `src/views/HomeView.vue`

**验收:**
- 项目字段包含 `id`、`name`、`description`、`created_at`、`updated_at`、`archived_at`。
- 首页可列出、创建、打开项目。
- 归档项目默认隐藏，可在设置或筛选中显示。
- Rust repository 有 CRUD 单元测试。

**分发提示词:**

```text
阅读 T006。请实现 Project 数据表、Rust command、TypeScript API、Pinia store 和首页项目列表/创建入口。只做项目 CRUD，不做词条。
```

### T007 通用实体 Entry 模型与 CRUD

**依赖:** T006

**范围:** 实现世界观通用词条，用于物品、地点、阵营、资源、文化、技术、魔法等。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/entry.rs`
- Create: `src-tauri/src/commands/entries.rs`
- Create: `src/api/entries.ts`
- Create: `src/stores/entryStore.ts`
- Create: `src/views/EntryListView.vue`
- Create: `src/views/EntryDetailView.vue`

**验收:**
- Entry 字段包含 `project_id`、`type`、`title`、`summary`、`body`、`tags`、`status`。
- 支持类型：`world_rule`、`item`、`location`、`faction`、`resource`、`culture`、`technology`、`magic`、`note`。
- 项目内可创建、编辑、删除、恢复词条。
- 删除采用软删除。

**分发提示词:**

```text
阅读 T007。请实现通用 Entry 词条 CRUD，包括数据库、Rust command、前端 API、store、列表页和详情编辑页。依赖已有 Project。
```

### T008 词条模板与本体类型

**依赖:** T007

**范围:** 为不同 Entry 类型提供结构化模板，支撑“逻辑依存关系”和“生态位锚定”。

**建议文件:**
- Create: `src/types/ontology.ts`
- Create: `src/domain/entryTemplates.ts`
- Create: `src/components/entry/EntryTemplatePanel.vue`
- Modify: `src/views/EntryDetailView.vue`

**验收:**
- 物品模板包含来源、制造者、材料、使用限制、关联规则。
- 地点模板包含地理、资源、政治归属、历史事件。
- 阵营模板包含意识形态、法律、资源、敌友关系。
- 新建词条时能选择模板并生成默认结构。

**分发提示词:**

```text
阅读 T008。请实现 Entry 类型模板和模板选择 UI。模板内容要贴合世界观/物品/地点/阵营/资源/文化/技术/魔法，不接 AI。
```

### T009 角色 Character 模型与基础编辑

**依赖:** T006

**范围:** 实现角色档案，不做自动属性分析。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/character.rs`
- Create: `src-tauri/src/commands/characters.rs`
- Create: `src/api/characters.ts`
- Create: `src/stores/characterStore.ts`
- Create: `src/views/CharacterListView.vue`
- Create: `src/views/CharacterDetailView.vue`

**验收:**
- Character 字段包含姓名、别名、简介、外貌、目标、动机、恐惧、阵营、标签。
- 支持项目内 CRUD 和软删除。
- 角色详情页能编辑长文本字段。

**分发提示词:**

```text
阅读 T009。请实现 Character 基础模型、CRUD、前端列表和详情编辑。不要实现 AI 提取或属性热图。
```

### T010 事件 Event 与时间轴模型

**依赖:** T006, T009

**范围:** 实现事件数据、时间表达和参与者关联。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/event.rs`
- Create: `src-tauri/src/commands/events.rs`
- Create: `src/api/events.ts`
- Create: `src/stores/eventStore.ts`
- Create: `src/views/EventListView.vue`
- Create: `src/views/EventDetailView.vue`

**验收:**
- Event 字段包含标题、描述、时间点、时间范围、地点、重要性、结果。
- 可关联多个 Character 和 Entry。
- 时间字段允许虚构纪年文本，同时保留可排序字段。

**分发提示词:**

```text
阅读 T010。请实现 Event 事件模型、CRUD、参与者关联和基础列表/详情页。只做数据与编辑，不做时间轴可视化。
```

### T011 Relation 关系图谱基础

**依赖:** T007, T009, T010

**范围:** 支持任意实体之间的关系边。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/relation.rs`
- Create: `src-tauri/src/commands/relations.rs`
- Create: `src/api/relations.ts`
- Create: `src/components/relation/RelationEditor.vue`
- Create: `src/components/relation/BacklinkPanel.vue`

**验收:**
- Relation 支持源对象、目标对象、关系类型、描述、置信度、方向。
- 支持类型：`causes`、`depends_on`、`belongs_to`、`conflicts_with`、`derived_from`、`located_in`、`ally_of`、`enemy_of`、`similar_to`。
- 任意详情页可查看反链和添加关系。

**分发提示词:**

```text
阅读 T011。请实现跨 Entry/Character/Event 的 Relation 数据表、命令、API、关系编辑器和反链面板。不要做图谱画布。
```

### T012 Axiom 公理与原子命题编辑

**依赖:** T007, T011

**范围:** 将世界观规则拆成可校验的原子命题。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/axiom.rs`
- Create: `src-tauri/src/commands/axioms.rs`
- Create: `src/api/axioms.ts`
- Create: `src/views/AxiomListView.vue`
- Create: `src/views/AxiomDetailView.vue`
- Create: `src/components/axiom/TripleEditor.vue`

**验收:**
- Axiom 字段包含 `subject`、`predicate`、`object`、`scope_time`、`scope_location`、`certainty`、`source_entity`、`natural_language`。
- 可从 Entry 详情页创建关联公理。
- 公理列表可按 subject/predicate/tag 搜索。

**分发提示词:**

```text
阅读 T012。请实现 Axiom 公理和三元组编辑器，用于世界观物理法则、社会契约、资源规则和历史约束。不要实现冲突检测。
```

### T013 全局搜索与筛选

**依赖:** T007, T009, T010, T012

**范围:** 实现本地关键词搜索，作为向量检索前的基础能力。

**建议文件:**
- Create: `src-tauri/src/commands/search.rs`
- Create: `src/api/search.ts`
- Create: `src/stores/searchStore.ts`
- Create: `src/views/SearchView.vue`
- Create: `src/components/search/SearchBox.vue`

**验收:**
- 可搜索 Entry、Character、Event、Axiom 标题和正文。
- 支持 project 范围、类型筛选、标签筛选。
- 搜索结果能跳转详情页。
- 无 AI、无向量时可用。

**分发提示词:**

```text
阅读 T013。请实现项目内全局关键词搜索和筛选 UI。优先使用 SQLite FTS；如果未启用 FTS，请实现可测试的 LIKE 搜索并在代码注释中说明后续替换点。
```

### T014 图谱可视化视图

**依赖:** T011

**范围:** 展示 Entry、Character、Event、Axiom 之间的关系图。

**建议文件:**
- Create: `src/views/GraphView.vue`
- Create: `src/components/graph/GraphCanvas.vue`
- Create: `src/components/graph/GraphFilters.vue`
- Create: `src/api/graph.ts`

**验收:**
- 可按实体类型和关系类型过滤。
- 点击节点打开详情。
- 点击边显示关系详情。
- 大于 200 个节点时默认按当前实体邻域加载，不一次渲染全项目。

**分发提示词:**

```text
阅读 T014。请实现关系图谱视图，只消费已有 Relation/实体 API。可选用 @vue-flow/core 或轻量 canvas/SVG 实现，但必须支持过滤、点击节点跳转和邻域加载。
```

### T015 时间线视图

**依赖:** T010

**范围:** 用时间轴展示事件与角色成长节点。

**建议文件:**
- Create: `src/views/TimelineView.vue`
- Create: `src/components/timeline/TimelineLane.vue`
- Create: `src/components/timeline/TimelineFilters.vue`
- Create: `src/api/timeline.ts`

**验收:**
- 事件按排序字段和虚构纪年展示。
- 可按角色、地点、标签过滤。
- 点击事件打开详情。
- 支持缺少精确日期的事件显示在“未定时间”分组。

**分发提示词:**

```text
阅读 T015。请实现项目事件时间线视图。只基于已有 Event 数据，不做 AI 分析。
```

### T016 导入导出与备份

**依赖:** T006-T012

**范围:** 支持项目 JSON 导入导出和本地备份。

**建议文件:**
- Create: `src-tauri/src/commands/import_export.rs`
- Create: `src-tauri/src/domain/project_archive.rs`
- Create: `src/api/importExport.ts`
- Create: `src/views/BackupView.vue`

**验收:**
- 导出单项目为 JSON，包含 entries、characters、events、axioms、relations。
- 导入时生成新 project，不覆盖现有项目。
- 导入校验版本号和必需字段。
- 备份文件不包含 API Key 和 CLI 日志。

**分发提示词:**

```text
阅读 T016。请实现项目级 JSON 导入导出和备份页。保证导入不会覆盖现有项目，导出不包含密钥与运行日志。
```

### T017 设置页与本机密钥存储

**依赖:** T003, T004

**范围:** 用户可配置 AI Provider、API Key、base_url、模型、CLI 路径。

**建议文件:**
- Create: `src-tauri/src/commands/settings.rs`
- Create: `src-tauri/src/windows/credential.rs`
- Create: `src/api/settings.ts`
- Create: `src/stores/settingsStore.ts`
- Modify: `src/views/SettingsView.vue`
- Create: `src/components/settings/ProviderSettings.vue`

**验收:**
- 设置支持 OpenAI-compatible、Claude CLI、Codex CLI 三类 provider。
- API Key 显示时脱敏。
- 保存后重新启动仍能读取非敏感配置。
- 密钥读取失败时提示用户重新输入。

**分发提示词:**

```text
阅读 T017。请实现设置页和本机密钥存储。优先 Windows Credential Manager；若使用文件存储，必须限制在 app config 目录并清楚标记后续加固点。
```

### T018 OpenAI-compatible HTTP Provider

**依赖:** T017

**范围:** 实现兼容 OpenAI Chat Completions 和 Embeddings 风格的 HTTP 调用适配器。

**建议文件:**
- Create: `src-tauri/src/ai/mod.rs`
- Create: `src-tauri/src/ai/provider.rs`
- Create: `src-tauri/src/ai/openai_compatible.rs`
- Create: `src-tauri/src/commands/ai.rs`
- Create: `src/api/ai.ts`

**验收:**
- 支持 `base_url`、`api_key`、`model`、`temperature`、`messages`。
- 支持 embedding 输入文本数组，返回向量数组和维度。
- HTTP 错误、鉴权错误、模型错误有结构化错误码。
- 有 Rust 单元测试覆盖 URL 拼接和响应解析。

**分发提示词:**

```text
阅读 T018。请实现 OpenAI-compatible HTTP AI Provider，包含 chat 和 embeddings 两类调用。不要接入业务工作流，只提供可测试的 provider 和 test command。
```

### T019 CLI Provider 抽象与进程执行器

**依赖:** T017

**范围:** 建立 Claude/Codex 共享的 CLI 无头调用框架。

**建议文件:**
- Create: `src-tauri/src/ai/cli_provider.rs`
- Create: `src-tauri/src/ai/process_runner.rs`
- Create: `src-tauri/src/ai/prompt_template.rs`
- Create: `src-tauri/src/commands/cli_providers.rs`

**验收:**
- 支持命令模板变量：`prompt`、`workspace`、`max_turns`、`output_format`。
- 进程有超时、stdout/stderr 捕获、退出码记录。
- 禁止 shell 拼接执行；使用 Windows 进程参数数组或经过明确解析的命令模板。
- 单元测试覆盖模板渲染和超时错误。

**分发提示词:**

```text
阅读 T019。请实现 AI CLI Provider 抽象和安全进程执行器。不要实现 Claude/Codex 具体适配，只提供模板渲染、超时、日志捕获和结构化结果。
```

### T020 Codex CLI Provider

**依赖:** T019

**范围:** 实现 Codex CLI 无头适配器。

**建议文件:**
- Create: `src-tauri/src/ai/codex_cli.rs`
- Modify: `src-tauri/src/commands/cli_providers.rs`
- Modify: `src/components/settings/ProviderSettings.vue`

**验收:**
- 默认命令模板使用 `codex exec`。
- 可检测 `codex --help` 是否存在。
- 可执行简单 prompt 并返回文本结果。
- 失败时展示缺失 CLI、鉴权失败、执行失败、超时四类错误。

**分发提示词:**

```text
阅读 T020。请基于现有 CLI Provider 抽象实现 Codex CLI provider。默认使用 codex exec，但允许用户覆盖命令模板。不要实现 Claude。
```

### T021 Claude CLI Provider

**依赖:** T019

**范围:** 实现 Claude Code CLI 无头适配器。

**建议文件:**
- Create: `src-tauri/src/ai/claude_cli.rs`
- Modify: `src-tauri/src/commands/cli_providers.rs`
- Modify: `src/components/settings/ProviderSettings.vue`

**验收:**
- 默认命令模板使用 `claude -p` 和 `--output-format json`。
- 可检测 `claude --help` 是否存在。
- 支持解析 text/json 输出，无法解析 json 时保留原始 stdout。
- 失败分类同 T020。

**分发提示词:**

```text
阅读 T021。请基于现有 CLI Provider 抽象实现 Claude CLI provider。默认使用 claude -p --output-format json，但允许用户覆盖命令模板。不要实现 Codex。
```

### T022 AI 后台任务队列与日志

**依赖:** T018, T020, T021

**范围:** 所有耗时 AI 调用进入后台队列，可查看状态、取消、重试。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/jobs/mod.rs`
- Create: `src-tauri/src/jobs/ai_jobs.rs`
- Create: `src-tauri/src/commands/jobs.rs`
- Create: `src/api/jobs.ts`
- Create: `src/stores/jobStore.ts`
- Create: `src/components/jobs/JobStatusBar.vue`
- Create: `src/views/JobsView.vue`

**验收:**
- Job 状态包含 queued、running、succeeded、failed、cancelled。
- 日志只记录摘要和错误，不记录 API Key。
- 前端底部状态栏展示当前运行任务。
- 可取消 CLI 进程。

**分发提示词:**

```text
阅读 T022。请实现 AI 后台任务队列、任务日志、状态栏和任务列表页。统一调度 OpenAI-compatible、Codex CLI、Claude CLI provider。
```

### T023 Prompt 模板管理

**依赖:** T022

**范围:** 管理逻辑审计、补完问题、角色分析、模拟等 prompt 模板。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/prompt_template.rs`
- Create: `src-tauri/src/commands/prompt_templates.rs`
- Create: `src/api/promptTemplates.ts`
- Create: `src/views/PromptTemplateView.vue`

**验收:**
- 内置模板可重置。
- 用户可复制并编辑模板。
- 模板支持变量说明，如 `{{project_context}}`、`{{target_entity}}`、`{{question}}`。
- 模板执行前能预览最终 prompt。

**分发提示词:**

```text
阅读 T023。请实现 Prompt 模板管理，包括内置模板、用户副本、变量说明和预览。不实现具体 AI 工作流。
```

### T024 文本切片 DocumentChunk

**依赖:** T007, T009, T010, T012

**范围:** 将项目文本统一切成可检索片段。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/vector/chunking.rs`
- Create: `src-tauri/src/commands/chunks.rs`
- Create: `src/api/chunks.ts`
- Create: `src/views/IndexingView.vue`

**验收:**
- Entry、Character、Event、Axiom 保存后可生成 chunks。
- chunk 记录 source type/id、文本、hash、token 估算、更新时间。
- 内容未变化时不重复生成。
- 有 Rust 测试覆盖切片边界。

**分发提示词:**

```text
阅读 T024。请实现 DocumentChunk 文本切片系统，覆盖 Entry/Character/Event/Axiom。不要调用 embedding，只生成可复用 chunks。
```

### T025 Embedding 生成与向量存储

**依赖:** T018, T024

**范围:** 调用 OpenAI-compatible embedding provider 生成向量并保存。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/vector/embedding.rs`
- Create: `src-tauri/src/commands/embeddings.rs`
- Modify: `src/views/IndexingView.vue`

**验收:**
- 支持按项目重建索引。
- 保存模型名、维度、chunk hash。
- chunk 内容变化后标记旧 embedding 失效。
- 无 embedding provider 时 UI 明确提示配置。

**分发提示词:**

```text
阅读 T025。请实现 embedding 生成和本地存储，复用 T018 OpenAI-compatible provider 和 T024 chunks。不要实现语义搜索排序 UI。
```

### T026 本地向量相似度检索

**依赖:** T025

**范围:** 实现 cosine similarity 检索和元数据过滤。

**建议文件:**
- Create: `src-tauri/src/vector/search.rs`
- Create: `src-tauri/src/commands/vector_search.rs`
- Create: `src/api/vectorSearch.ts`
- Modify: `src/views/SearchView.vue`

**验收:**
- 输入 query 后先生成 query embedding，再返回相似 chunks。
- 支持过滤 source type、时间范围、标签。
- 返回 score、source、snippet。
- Rust 测试覆盖 cosine similarity。

**分发提示词:**

```text
阅读 T026。请实现本地向量相似度检索，使用已存 embedding 和 query embedding。先用内存计算 cosine 即可，但接口要便于未来替换为 sqlite-vec 或 HNSW。
```

### T027 RAG 上下文包构建器

**依赖:** T026, T011

**范围:** 为 Agent 生成可解释的上下文包，结合向量命中、反链、直接关联实体。

**建议文件:**
- Create: `src-tauri/src/ai/context_pack.rs`
- Create: `src-tauri/src/commands/context_pack.rs`
- Create: `src/api/contextPack.ts`
- Create: `src/components/ai/ContextPackPreview.vue`

**验收:**
- 输入目标实体或用户问题，输出 context pack。
- 包含直接实体正文、相关关系、向量相似片段、公理。
- 每条上下文保留 source 引用，便于报告解释。
- 前端可预览上下文包。

**分发提示词:**

```text
阅读 T027。请实现 RAG 上下文包构建器，把实体详情、关系、Axiom 和向量搜索结果合并为有 source 引用的上下文。不要调用 LLM。
```

### T028 叙事共鸣与突兀度预警

**依赖:** T026

**范围:** 基于向量距离识别新设定与既有设定的相似、孤立或突兀程度。

**建议文件:**
- Create: `src-tauri/src/vector/resonance.rs`
- Create: `src-tauri/src/commands/resonance.rs`
- Create: `src/components/entry/ResonancePanel.vue`
- Modify: `src/views/EntryDetailView.vue`

**验收:**
- 对当前 Entry 返回最相近设定、平均相似度、低相关预警。
- 不把低相似度视为绝对错误，只提示“缺少铺垫或关系”。
- 结果可跳转相关词条。

**分发提示词:**

```text
阅读 T028。请实现叙事共鸣/突兀度预警，用向量相似度帮助用户发现设定是否缺少铺垫。不要做 LLM 审计。
```

### T029 规则约束校验引擎

**依赖:** T012

**范围:** 基于 Axiom 和 Relation 做第一版确定性冲突检测。

**建议文件:**
- Create: `src-tauri/src/logic/mod.rs`
- Create: `src-tauri/src/logic/rules.rs`
- Create: `src-tauri/src/logic/conflict.rs`
- Create: `src-tauri/src/commands/logic.rs`
- Create: `src/api/logic.ts`

**验收:**
- 检测同一 scope 下互斥谓词冲突，如 `state = solid` 与 `state = liquid`。
- 检测时间矛盾，如实体来源晚于使用事件。
- 检测关系矛盾，如同一 scope 下 `ally_of` 与 `enemy_of`。
- 返回冲突涉及的最小候选事实列表。

**分发提示词:**

```text
阅读 T029。请实现第一版确定性逻辑约束校验引擎，基于 Axiom 和 Relation 检测明确冲突。不要调用 AI，不做修复建议。
```

### T030 QuickXplain 风格 MUS 诊断

**依赖:** T029

**范围:** 对冲突候选事实做最小不满足子集诊断。

**建议文件:**
- Create: `src-tauri/src/logic/quickxplain.rs`
- Modify: `src-tauri/src/logic/conflict.rs`
- Create: `src-tauri/src/commands/conflict_diagnosis.rs`

**验收:**
- 输入一组 facts 和 consistency checker，输出一个或多个最小冲突集合。
- 有 Rust 测试覆盖 3 条、5 条、10 条事实的最小冲突定位。
- 大集合超时后返回部分诊断和明确状态。

**分发提示词:**

```text
阅读 T030。请实现 QuickXplain 风格的最小冲突集合诊断。保持算法接口独立，后续可替换 checker。只做 Rust 逻辑和 command。
```

### T031 冲突修复建议生成

**依赖:** T023, T027, T030

**范围:** 根据 MUS 和上下文包生成修复方案。

**建议文件:**
- Create: `src-tauri/src/logic/repair.rs`
- Create: `src-tauri/src/commands/repair_suggestions.rs`
- Create: `src/api/repairSuggestions.ts`
- Create: `src/components/logic/RepairSuggestionPanel.vue`

**验收:**
- 无 AI 时给出规则型建议：修改新设定、添加例外、调整全局公理、补充中间事件。
- 有 AI 时调用所选 provider 生成解释和方案。
- 每个建议包含影响范围、修改目标、风险说明。
- 不自动修改用户数据。

**分发提示词:**

```text
阅读 T031。请实现冲突修复建议生成。必须先提供无 AI 的规则型建议，再可选调用 AI Provider 丰富说明。不要自动写入修复。
```

### T032 逻辑审计报告 UI

**依赖:** T029, T030, T031

**范围:** 将冲突检测、MUS、修复建议整合成可读报告。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/audit_report.rs`
- Create: `src-tauri/src/commands/audit_reports.rs`
- Create: `src/api/auditReports.ts`
- Create: `src/views/AuditReportView.vue`
- Create: `src/components/logic/ConflictReport.vue`

**验收:**
- 可对单个实体或整个项目运行审计。
- 报告持久化，可重新打开。
- 报告列出事实来源、冲突原因、建议、生成时间、provider。
- 报告不覆盖原设定。

**分发提示词:**

```text
阅读 T032。请实现逻辑审计报告的数据模型和 UI，把冲突检测、MUS 诊断和修复建议串起来。报告必须可保存和重开。
```

### T033 角色事件影响提取

**依赖:** T009, T010, T022, T023, T027

**范围:** 从角色关联事件中提取性格、技能、动机、情绪影响。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/character_trait.rs`
- Create: `src-tauri/src/commands/character_analysis.rs`
- Create: `src/api/characterAnalysis.ts`
- Create: `src/components/character/EventImpactPanel.vue`

**验收:**
- 输入角色和事件，输出 trait deltas。
- 维度包含 OCEAN、核心动机、技能、情感状态、创伤/稳定性。
- AI 输出必须解析为结构化 JSON；解析失败保留原文并标记失败。
- 用户确认后才写入 trait deltas。

**分发提示词:**

```text
阅读 T033。请实现角色事件影响提取工作流。用 AI Provider 分析事件对角色性格、技能、动机和情绪的影响，但写入前必须让用户确认。
```

### T034 角色属性快照与成长热图

**依赖:** T033

**范围:** 基于 trait deltas 生成属性时间序列和可视化热图。

**建议文件:**
- Create: `src-tauri/src/domain/character_snapshot.rs`
- Create: `src-tauri/src/commands/character_snapshots.rs`
- Create: `src/api/characterSnapshots.ts`
- Create: `src/components/character/GrowthHeatmap.vue`
- Modify: `src/views/CharacterDetailView.vue`

**验收:**
- 可按事件顺序生成角色属性快照。
- 热图展示时间点与属性维度变化。
- 用户可查看每个数值变化的来源事件。
- 不允许用户直接改最终属性，只能改事件影响或手动添加修正记录。

**分发提示词:**

```text
阅读 T034。请实现角色属性快照和成长热图。属性来自已确认的事件影响记录，必须能追溯来源事件。
```

### T035 角色画像综合报告

**依赖:** T034, T027

**范围:** 生成角色当前 persona 报告。

**建议文件:**
- Create: `src-tauri/src/commands/character_profile.rs`
- Create: `src/api/characterProfile.ts`
- Create: `src/components/character/ProfileReportPanel.vue`

**验收:**
- 报告包含当前性格、核心动机、能力边界、人际关系、矛盾点。
- 每段结论引用来源事件或设定。
- 支持无 AI 基础摘要和 AI 深度分析两种模式。

**分发提示词:**

```text
阅读 T035。请实现角色画像综合报告。必须基于角色档案、事件、trait snapshots 和上下文包生成，并在报告中保留来源引用。
```

### T036 行为合理性校验

**依赖:** T027, T032, T035

**范围:** 用户输入一个角色行为，系统判断是否符合角色与世界观逻辑。

**建议文件:**
- Create: `src-tauri/src/commands/behavior_audit.rs`
- Create: `src/api/behaviorAudit.ts`
- Create: `src/views/BehaviorAuditView.vue`
- Create: `src/components/agent/BehaviorAuditForm.vue`

**验收:**
- 输入角色、拟议行为、场景背景。
- 输出连贯性评分、支持证据、矛盾证据、需要补足的铺垫。
- 支持选择 provider。
- 报告可保存为 AuditReport。

**分发提示词:**

```text
阅读 T036。请实现角色行为合理性校验工作流。输入角色行为和背景，基于 RAG 上下文与角色画像输出评分、证据和补足建议。
```

### T037 Agent 聊天工作台

**依赖:** T022, T027

**范围:** 提供项目内 Agent 对话界面。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/conversation.rs`
- Create: `src-tauri/src/commands/conversations.rs`
- Create: `src/api/conversations.ts`
- Create: `src/views/AgentChatView.vue`
- Create: `src/components/agent/ChatPanel.vue`
- Create: `src/components/agent/SourceCitationList.vue`

**验收:**
- 支持新建对话、保存消息、选择 provider。
- 每次提问可选择是否附带 RAG 上下文。
- 回答显示引用来源列表。
- Provider 失败时不丢失用户消息。

**分发提示词:**

```text
阅读 T037。请实现 Agent 聊天工作台，支持保存对话、选择 AI Provider、附带 RAG 上下文和展示来源引用。
```

### T038 设定补完问题生成器

**依赖:** T023, T027, T037

**范围:** 对新设定自动提出支撑逻辑问题，如能源、伦理、经济影响、历史来源。

**建议文件:**
- Create: `src-tauri/src/commands/completion_questions.rs`
- Create: `src/api/completionQuestions.ts`
- Create: `src/components/entry/CompletionQuestionsPanel.vue`

**验收:**
- 可对 Entry 或 Axiom 生成问题。
- 问题按类别分组：物理来源、资源消耗、社会影响、历史因果、例外边界。
- 用户可将问题转为待补充 note 或新 Entry。

**分发提示词:**

```text
阅读 T038。请实现设定补完问题生成器。它应基于目标设定和上下文包提出结构化问题，并允许用户把问题转为 note 或新词条。
```

### T039 叙事模拟工作流

**依赖:** T023, T027, T035, T037

**范围:** 针对“如果发生 X，未来 Y 时间会怎样”生成模拟报告。

**建议文件:**
- Create: `src-tauri/src/commands/simulation.rs`
- Create: `src/api/simulation.ts`
- Create: `src/views/SimulationView.vue`
- Create: `src/components/simulation/ScenarioForm.vue`
- Create: `src/components/simulation/SimulationReport.vue`

**验收:**
- 输入情景、时间跨度、涉及阵营/角色/地点。
- 输出阶段性推演、关键角色行为、资源变化、潜在冲突、创作建议。
- 报告引用上下文来源。
- 保存为 AuditReport 或 SimulationReport。

**分发提示词:**

```text
阅读 T039。请实现叙事模拟工作流。用户输入假设情景和时间跨度，系统基于角色画像、世界观规则和上下文包生成模拟报告。
```

### T040 平行时空报告管理

**依赖:** T039

**范围:** 管理多次模拟结果，便于比较不同假设。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/simulation_report.rs`
- Create: `src-tauri/src/commands/simulation_reports.rs`
- Create: `src/api/simulationReports.ts`
- Create: `src/views/ParallelWorldsView.vue`

**验收:**
- 可保存多个 scenario 和结果。
- 可复制一个 scenario 改参数重新运行。
- 可并排比较关键结论、风险、涉及实体。
- 可从报告跳转到相关实体。

**分发提示词:**

```text
阅读 T040。请实现平行时空/模拟报告管理。重点是保存、复制、比较和跳转，不重新实现模拟生成。
```

### T041 多模态附件与资料引用

**依赖:** T007, T009, T010

**范围:** 支持给词条、角色、事件挂载图片、PDF、文本文件等本地附件。

**建议文件:**
- Modify: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/domain/attachment.rs`
- Create: `src-tauri/src/commands/attachments.rs`
- Create: `src/api/attachments.ts`
- Create: `src/components/attachment/AttachmentPanel.vue`

**验收:**
- 附件复制到项目 app data 目录或记录安全路径引用，策略全仓一致。
- 显示文件名、类型、大小、添加时间。
- 图片可预览。
- 删除实体时附件不立即物理删除，提供清理孤儿附件命令。

**分发提示词:**

```text
阅读 T041。请实现本地附件系统，支持 Entry/Character/Event 关联图片和文件。不要实现 OCR 或图片理解。
```

### T042 Windows 语音输入入口

**依赖:** T037

**范围:** 为 Agent 输入框和长文本编辑器提供 Windows 可用的语音输入入口。

**建议文件:**
- Create: `src-tauri/src/windows/speech.rs`
- Create: `src-tauri/src/commands/speech.rs`
- Create: `src/api/speech.ts`
- Create: `src/components/input/VoiceInputButton.vue`

**验收:**
- 如果无法稳定调用系统语音 API，提供明确的“使用 Windows 系统听写快捷键”的 UI 降级方案。
- 不阻塞文本输入。
- 用户可关闭语音入口。

**分发提示词:**

```text
阅读 T042。请实现 Windows 语音输入入口或可靠降级。若本机系统 API 接入复杂，先实现可关闭的入口和系统听写指引，不影响文本工作流。
```

### T043 命令面板与快捷操作

**依赖:** T003, T006-T013

**范围:** 提供全局命令面板，快速创建实体、搜索、运行审计、打开设置。

**建议文件:**
- Create: `src/components/command/CommandPalette.vue`
- Create: `src/stores/commandStore.ts`
- Modify: `src/App.vue`

**验收:**
- `Ctrl+K` 打开命令面板。
- 可搜索实体并跳转。
- 可执行创建 Entry/Character/Event/Axiom。
- 可跳转到 AI、设置、审计、模拟页面。

**分发提示词:**

```text
阅读 T043。请实现全局命令面板和 Ctrl+K 快捷键。只组合已有功能入口，不新增业务模型。
```

### T044 Windows 日志、诊断与错误报告

**依赖:** T004, T022

**范围:** 统一应用日志、AI 调用错误、崩溃诊断导出。

**建议文件:**
- Create: `src-tauri/src/windows/logging.rs`
- Create: `src-tauri/src/commands/diagnostics.rs`
- Create: `src/api/diagnostics.ts`
- Create: `src/views/DiagnosticsView.vue`

**验收:**
- 日志写入 app data logs 目录。
- 诊断导出包含版本、平台、数据库迁移版本、最近错误摘要。
- 导出前自动脱敏 API Key、用户密钥、完整 prompt 正文。
- 设置页可打开日志目录。

**分发提示词:**

```text
阅读 T044。请实现 Windows 本地日志和诊断导出。必须脱敏密钥和完整 prompt，不上传任何数据。
```

### T045 前端单元测试基线

**依赖:** T003, T004

**范围:** 建立 Vitest + Vue Test Utils + Tauri mock 测试基线。

**建议文件:**
- Create: `vitest.config.ts`
- Create: `tests/unit/AppShell.spec.ts`
- Create: `tests/unit/apiHealth.spec.ts`
- Modify: `package.json`

**验收:**
- `pnpm test:unit` 成功。
- 覆盖路由渲染、API wrapper 错误处理、至少一个 store。
- 测试不依赖真实 Tauri 窗口。

**分发提示词:**

```text
阅读 T045。请建立前端单元测试基线，使用 Vitest、Vue Test Utils 和 Tauri mock IPC。不要改业务功能。
```

### T046 Rust 后端测试基线

**依赖:** T005

**范围:** 为 db、domain、logic、vector 模块提供统一测试辅助。

**建议文件:**
- Create: `src-tauri/src/test_support.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/tests/db_migrations.rs`

**验收:**
- 测试可创建临时 app data 和临时 SQLite。
- `cargo test` 不依赖用户真实数据库。
- 后续 domain repository 可复用 helper。

**分发提示词:**

```text
阅读 T046。请建立 Rust 后端测试基线，包括临时数据库、迁移测试和可复用 test_support。不要新增业务功能。
```

### T047 Playwright 冒烟测试

**依赖:** T003, T006

**范围:** 建立桌面 UI 的关键路径冒烟测试。

**建议文件:**
- Create: `playwright.config.ts`
- Create: `tests/e2e/smoke.spec.ts`
- Modify: `package.json`

**验收:**
- 可启动前端 dev server 或 Tauri dev 可测入口。
- 测试覆盖打开首页、创建项目、进入项目。
- CI 不存在时也能本地运行。

**分发提示词:**

```text
阅读 T047。请建立 Playwright 冒烟测试，覆盖打开应用、创建项目、进入项目。若 Tauri 自动化有阻碍，先测 Vite 前端入口并记录后续 Tauri 桌面测试方案。
```

### T048 Windows 打包与安装包

**依赖:** M1 至少完成，建议 M7 后执行

**范围:** 配置 Tauri Windows 构建产物、图标、应用名称、安装包目标。

**建议文件:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/icons/`
- Modify: `README.md`
- Modify: `package.json`

**验收:**
- `pnpm tauri build` 在 Windows 生成可安装包。
- app 名称为 Ameya。
- README 写明未签名构建的 SmartScreen 预期和代码签名后续方案。
- 不启用 macOS/Linux bundle。

**分发提示词:**

```text
阅读 T048。请配置 Windows Tauri 打包、图标、应用名和构建脚本。只面向 Windows，完成后运行 pnpm tauri build 或说明本机缺失的构建依赖。
```

### T049 性能与大项目基准

**依赖:** T013, T014, T026

**范围:** 构造测试数据并检查搜索、图谱、向量检索性能。

**建议文件:**
- Create: `src-tauri/src/dev_seed.rs`
- Create: `scripts/seed-large-project.ps1`
- Create: `plans/performance-baseline.md`

**验收:**
- 可生成至少 1000 entries、200 characters、2000 relations、500 events 的测试项目。
- 记录关键词搜索、邻域图谱、向量检索耗时。
- 对超过阈值的路径提出具体优化任务。

**分发提示词:**

```text
阅读 T049。请实现大项目测试数据生成和性能基准记录。不要优化业务代码，除非是让基准能稳定运行所需的小修。
```

### T050 用户帮助与内置任务说明

**依赖:** M7

**范围:** 写用户可读帮助，不解释技术实现，聚焦工作流。

**建议文件:**
- Create: `src/views/HelpView.vue`
- Create: `src/content/help.ts`
- Modify: `src/router/index.ts`
- Modify: `README.md`

**验收:**
- 帮助页覆盖：项目、词条、角色、事件、公理、关系、向量索引、审计、模拟、AI 设置。
- 文案不承诺 AI 一定正确，强调需要用户确认。
- README 给出从安装到首次创建项目的步骤。

**分发提示词:**

```text
阅读 T050。请实现用户帮助页和 README 使用说明。只写面向创作者的操作说明，不写开发者内部实现细节。
```

## 4. 推荐执行顺序

1. 先完成 T001-T006，得到可运行工程和 Project CRUD。
2. 并行分发 T007、T009、T010、T017、T045、T046。
3. 在实体模型稳定后完成 T011-T016，形成无 AI 的可用资料库。
4. 并行分发 T018-T023，打通三类 AI Provider 与后台任务。
5. 完成 T024-T028，建立向量化与 RAG 上下文。
6. 完成 T029-T032，建立逻辑一致性审计闭环。
7. 完成 T033-T040，落地角色成长、Agent、模拟和平行时空。
8. 视优先级完成 T041-T044 的附件、语音、命令面板和诊断能力。
9. 最后完成 T047-T050 的冒烟测试、Windows 打包、性能基准和用户帮助。

## 5. 分发给 Claude/Codex 的统一约束模板

```text
你在实现 Ameya 的一个独立任务。请先阅读：
1. plans/Tauri-Vue-Windows开发路线图.md
2. plans/世界观人设工具功能架构设计.md

只实现我指定的任务编号，不实现后续任务，不做无关重构。
技术栈固定为 Tauri 2 + Vue 3 + TypeScript + Rust + SQLite，只考虑 Windows。
如果任务涉及 AI，必须通过已定义的 Provider 抽象，不在组件中直接调用外部 API 或 CLI。
如果任务涉及数据库，必须提供迁移和测试，不破坏已有数据。
完成后运行相关验证命令，并在回复中列出：
- 修改文件
- 验证命令和结果
- 未完成项或阻塞点
```

## 6. 官方资料依据

- Tauri Windows 前置依赖：Microsoft C++ Build Tools、WebView2、Rust MSVC、Node.js。参考：https://v2.tauri.app/start/prerequisites/
- Vue 3 新项目推荐 Vite。参考：https://vuejs.org/guide/quick-start.html
- Vue Router hash history 适合无服务端回退的桌面/文件环境。参考：https://router.vuejs.org/guide/essentials/history-mode
- Claude Code CLI 支持 `claude -p` / `--print` 无头模式和 `--output-format json`。参考：https://code.claude.com/docs/en/headless 与 https://code.claude.com/docs/en/cli-usage
- Codex CLI 的 `codex exec` 是非交互执行入口。参考：https://github.com/openai/codex/blob/main/docs/exec.md
