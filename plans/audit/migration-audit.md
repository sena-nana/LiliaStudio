# NanaUI 迁移重新检查报告（Migration Re-Audit）

- 审计日期: 2026-11-10
- 审计基准: Tauri 版功能 = git HEAD `d6093cf`（最后一个含 Vue 前端的完整状态）；NanaUI 规范 = rev `596930d`（native/Cargo.toml pin）
- 依据材料: [tauri-feature-matrix.md](tauri-feature-matrix.md)（功能对照契约）、[nanaui-api-reference.md](nanaui-api-reference.md)（API/规范参考）
- 用户要求（m00001）: ① 符合 NanaUI 开发规范 ② 原本 Tauri 版本功能完整实现 ③ 所有界面按实际作用有自己的布局设计

---

## 一、总体结论

**迁移未完成，当前状态不满足三项要求中的任何一项。** `native/` 目前是一个"骨架壳"：
能编译、能通过自建的离线快照验收，但 hosted 视图是静态占位内容，命令边界有 8 组桩实现，
每条路由共用同一个通用渲染器，没有按页面作用设计的布局。离线验收管道校验的是一套
与真实 UI 平行的手搓几何模型——artifacts README 自己也承认"这些像素图不是实际 hosted
NanaUI GPU 窗口的像素基线"。

## 二、差距清单（按三项要求）

### 要求① 符合 NanaUI 开发规范 —— 不满足

| # | 违反点 | 现状 | 规范要求 |
| --- | --- | --- | --- |
| 1 | `pub use nana_ui as framework` 别名（native/src/lib.rs:23） | `framework` 模块在 NanaUI 中不存在，canonical 根是 `nana_ui::runtime` | 删除别名，按 canonical 路径导入 |
| 2 | 每次更新整树重挂（native/src/main.rs `update()` 里 re-mount view） | `RuntimeMessage` 处理后 `mounted_view` 重建 + `redraw_all` | 树挂载时建一次；改信号/store，禁止点击处理里重挂整页（application.md 警告原文） |
| 3 | 可见控件未接真实状态 | 16 条路由的卡片/表格/输入是硬编码字符串（"+12%"、"▰▰▰▱▱ 60%"、"节点 A — 关系 — 节点 B"）；Settings 4 个输入框不落库 | "Visible actions must be wired to real state. Do not add placeholder routes… or unconnected controls to product UI"（NanaUI AGENTS.md） |
| 4 | 无响应式状态层 | `NativeApp` 是普通 struct + 全量重建 | 单一顶层 `store(AppState)`（`#[derive(Store)]`），keyed 列表按行更新 |
| 5 | 未用框架组件体系 | 只用 Card/Stack/Table/TextInput 等少数叶子；无 DesktopShell/SidebarFrame/CommandPalette/Toast/Dialog | 桌面应用用 DesktopShell slots + SidebarFrame/Section/Row 导航 + overlay 体系 |
| 6 | 快照几何是手搓平行模型 | `ViewBuilder`/`Rect`/`Group` 手算坐标，与真实 hosted 树无关 | `nana_ui::headless` 驱动真程序跑真布局，几何从 RuntimeDocument 读回（权威值） |
| 7 | 主题/外观未接 | 无主题切换；DESIGN.md 双主题要求未实现 | 语义颜色角色 + ThemeAppearance/CompiledTheme；深浅主题都可读 |

### 要求② Tauri 功能完整实现 —— 不满足

命令边界（native/src/host.rs）桩/缺失清单：

| 命令 | 现状 | Tauri 版行为（feature-matrix §4） |
| --- | --- | --- |
| `ai_agent_ask` | `Err("requires an explicitly configured provider")` | openAiCompatible 解析 + 固定中文系统提示词 + 编号引用上下文，temperature 0.2 |
| `ai_load_provider_settings` / `ai_save_provider_settings` | `Err("requires a native secret store")` | config JSON（仅存 hasApiKey 布尔）+ Windows 凭据管理器（service `ameya.ai`）；`WindowsCredentialStore` 未移植 |
| `ai_test_openai_provider` / `ai_test_codex_cli_provider` / `ai_test_claude_cli_provider` | `Err(...)` | 真实测试调用（chat "Return the word ok." / CLI 模板 workspace 沙箱跑 test_prompt） |
| `rag_index_embeddings` | 返回硬编码 `{"embeddingCount":0,"model":""}` | 切片(600)→解析 embedding provider→批量向量→upsert SQLite；数量不一致报错；降级中文提示语逐级精确 |
| `search_semantic` | 永远降级为关键词回退 | `{status ready\|degraded\|failed, message, model, items}`，查询向量化→余弦 top-N→回表；从不抛错 |
| `diagnostics_summary` | 桩 `{status:"ready",dataDir}` | `{appVersion, platform, database:"sqlite"}` |
| provider 解析层 | 根核心 src/ai/ 无 provider_resolver.rs（Tauri 版在 src-tauri/src/ai/） | OPENAI_{TEST,AGENT,RAG,SEARCH}_MESSAGES 上下文化中文报错 |

已实现且正确的命令（project/library/jobs/prompts/simulation/growth/archive/rag_index_chunks/
rag_index_status/search_entities/logic_*/vector_preview/health_check 等）与 Tauri 行为一致，可保留。

### 要求③ 界面按实际作用有自己的布局 —— 不满足

现状: `render_workspace_body()`（native/src/lib.rs:132-514）给 16 条路由各自堆了一组**静态**
卡片，之后所有路由统一走 `render_page_sections()` 通用渲染器（native/src/lib.rs:516-630）。
没有一条路由拥有工作台级布局（分栏、检视器、工具栏、过滤器），也没有任何真实数据流。

Tauri 版有明确布局语义的页面（feature-matrix §1）在 NanaUI 版的对应落差:

| 页面 | Tauri 布局（作用） | NanaUI 现状 |
| --- | --- | --- |
| 资料库 /projects | 三栏对象工作台: CollectionRail(5 集合+计数) + ObjectBrowser(5 列表格+模板面板) + RecordEditor(五类分表单) + ObjectInspector(关系邻域/待确认建议/缺失项) | 1 行硬编码表格 + 两个未接线输入框 |
| 关系图谱 /graph | SVG 边 + 同心环节点 + 类型过滤 + 单击聚焦 + 双击跳转对象 | 一行文字"节点 A — 关系 — 节点 B" |
| 搜索 /search | 混合检索工作台: 关键词+语义并行、来源 chip、相似度、降级状态行 | 空表格 |
| 智能体 /agent | 问答控制台: 问题表单→混合检索→引用列表→回答卡→保存为报告/任务 | 静态"Codex CLI"文本 |
| 索引 /indexing | 5 指标卡(切片/Embedding/缺失/过期/模型) + 状态机文本 + 重建按钮 | 假进度条"▰▰▰▱▱ 60%" |
| 角色成长 /growth | 角色栏 + trait 状态卡(±X.XX) + 表单→预览→保存 + 来源记录 + 输入检查 | 假数据矩阵"+12%" |
| 时间线 /timeline | sortKey 排序事件流(timeLabel+标题+描述) | 1 条假数据 |
| 提示词模板 /prompt-templates | 三栏: 列表｜编辑器(内置只读/复制/保存)｜预览({{VAR}} 提取+填充) | 未接线编辑器 |
| 任务 /jobs | 任务卡(jobTypeLabel/provider/status) + 日志/取消/重试 | 1 空行 |
| 设置 /settings | 独立侧栏 4 分区(外观/AI/CLI 提供方/关于) + 密钥掩码 + 测试按钮 | 4 个不落库的输入框 |
| 其余(Home/Backup/Simulation/Audit/Help/Diagnostics) | 各有表单/卡片/导出导入流 | 静态文本卡 |

Shell 层缺失: Ctrl+K 命令面板(14 命令)、自绘标题栏(最小化/最大化/关闭)、侧栏宽度拖拽
(180-480, 双击复位)+折叠、设置页锁定侧栏、本地优先状态胶囊、主题持久化(ameya.theme)、
窗口状态恢复(main-window-state.json, ≥960×600, 最大化保留正常几何)。

### 继承性风险

- **整个迁移未提交**: 工作树 280 处变更（Vue/Tauri 全删 + 新 Rust 未跟踪）。审计与实现期间不得误提交/误还原。
- **富文本数据契约**: 存量 entry.summary/body 是 TipTap JSON。NanaUI RichText 无 HTML/TipTap
  导入路径，需要 app 侧转换器（读时 TipTap JSON→RichText，存时回写 TipTap JSON 保持库兼容）。
- **残留功能确认跳过**: ViewTabs、JobStatusBar、library_list_backlinks、rag_preview_context_pack
  在 Tauri 版也无 UI 消费（feature-matrix §2/§4），不属"功能完整"范围。

## 三、修复架构（已定）

```
native/src/
├── main.rs        NanaApplication 入口 + headless --render-pages（经真实布局导出快照）
├── lib.rs         模块 wiring + re-export（删 framework 别名）
├── app.rs         ApplicationState 实现: initialize/build/update/theme/close_requested
├── state.rs       顶层 AppState store（#[derive(Store)]）+ Route 枚举 + 后台任务消息
├── commands.rs    NativeCommandHost 之上的异步封装（网络类命令走后台线程+消息回投）
├── secrets.rs     WindowsCredentialStore（keyring service "ameya.ai"）+ 测试用内存实现
├── host.rs        命令边界（补齐全部桩 → 全真实现）
├── shell.rs       DesktopShell + AppTitleBar + Sidebar 导航(4 组/16 路由) + StatusBar + CommandPalette + Toast
├── rich_convert.rs TipTap JSON ⇄ RichText 双向转换（存量数据兼容）
├── views/         16 条路由，每页一个模块，按作用设计布局（对齐 feature-matrix §1）
└── page_model.rs  保留为内容契约源（列定义/空态文案/action 映射），供视图与快照共用
根核心 src/ai/
├── provider_resolver.rs  （移植自 src-tauri，四类中文报错上下文）
└── agent_ask.rs          （移植: 系统提示词 + 引用拼装 + temperature 0.2）
```

关键决策:
1. **快照从真实布局导出**: `nana_ui::headless` 驱动同一 ApplicationState，present 后从
   RuntimeDocument 读真实几何与内容记录，替代手搓 ViewBuilder——现有 C++ 验收器与
   content-acceptance.tsv 契约继续消费同一记录格式（闭环 README 承认的缺口）。
2. **网络命令异步化**: AI 问答/嵌入索引/提供方测试走后台线程，完成后以 Message 回投
   update；SQLite 本地命令同步调用（快）。
3. **数据契约零漂移**: 命令名/参数/返回 camelCase 与 feature-matrix §6 逐字段对齐；
   富文本存储格式保持 TipTap JSON（转换器隔离在 UI 层）。
4. **主题**: ThemeAppearance Light/Dark 双预设，持久化沿用 `ameya.theme` 语义（NanaUI
   侧存 theme_id），颜色全部走语义角色。

## 四、实施阶段

| 阶段 | 内容 | 交付判据 |
| --- | --- | --- |
| P1 命令边界补齐 | provider_resolver/agent_ask 移植、凭据库、全部桩命令真实现 | host.rs 全命令真实现 + 单测（内存 SecretStore） |
| P2 应用骨架 | state.rs store + app.rs ApplicationState + shell.rs + main.rs 入口 | 挂载一次的真树；导航/面板/标题栏/主题可用 |
| P3 十六页视图 | 每页按作用设计布局，接真实数据流（对齐 §1 矩阵） | 无占位内容；每页有空/加载/错误态 |
| P4 快照管道 | headless 真布局导出 + artifacts 重生成 + 契约 TSV 校准 | ctest 全绿；快照反映真实树 |
| P5 终验 | cargo test --workspace / cargo check --features nana-runtime / cmake ctest / 文档更新 | 全部通过并记录命令与结果 |

## 实施结果（2026-11-10 收口）

按上表五个阶段全部实施完成，终验全绿：

- P1 命令边界：`src/ai/provider_resolver.rs`、`src/ai/agent_ask.rs`、`native/src/secrets.rs`（Windows Credential Manager，keyring 3.6.3 windows-native，service `ameya.ai`）落地；host.rs 全部 58 个命令真实现，含 ai_agent_ask / ai_test_* / rag_index_embeddings / search_semantic / ai_load+save_provider_settings / diagnostics_summary。
- P2 骨架：`native/src/{state,app,shell,commands}.rs` + `views/`；`Store<AppState>` 单一状态树，视图挂载一次不再重挂；网络命令（AI 调用/嵌入构建/语义检索）走 `run_task` 后台任务；主题与侧栏折叠持久化（FileStore kv），窗口尺寸 `persist_key("main")` + 最小 960×600。
- P3 视图：16 个路由全部按作用布局——资料库三栏对象工作台（集合栏/对象浏览/记录编辑/邻接检查含关系建议确认）、关系图谱 GraphCanvas（同心环布局、聚焦、关系/实体过滤）、成长工作台（角色梯/来源事件表单/预览确认/记录列表）、检索双列（关键词+语义状态徽章）、任务队列（当前任务/列表/日志）、提示词模板编辑+变量预览、AI 设置四分页、命令面板（Ctrl+K，14 命令）等；富文本经 `rich_convert.rs` 与 TipTap JSON 双向转换。
- P4 快照：`native/src/snapshot_export.rs` 用离屏 GPU 上下文 + `HeadlessRuntime<RuntimeApplication<AmeyaApp>>` 驱动**真实应用布局**，经无障碍树投影导出 window/group/control 几何（`AMEYA_SNAPSHOT_SEED=1` 注入确定性样例数据），业务契约记录沿用 `content_records`；artifacts 32 个快照全部重生成，metadata.json/PNG/README 同步更新；过程中暴露并修复了真实布局问题（ScrollView 内输入控件未约束宽度挤压兄弟、容器型 ListItem 与子项重叠）。
- P5 终验（全部通过）：`cargo fmt --all -- --check` ✓；`cargo check --workspace` 0 警告 ✓；`cargo test --workspace` 132 通过 ✓；`cargo test -p ameya-nana --features nana-runtime` 42 通过 ✓；`ctest -C Debug` 36/36 ✓。

遗留与有意偏差（2026-11-10 第二轮已全部收口）：①ai_default_providers 已对齐 Tauri 的 2 项契约（codex_cli + claude_cli；首选项默认仍由 load 侧 3 项提供，与 Tauri-era core 一致）；②图谱视口改为受控字段——平移/缩放经 GraphViewportChanged 回写 store，聚焦/过滤重建不再重置视角，工具栏新增「重置视图」；③侧栏宽度拖拽（180–480，默认 240）经 Workspace Resources 区原生 resize 接入，拖拽手柄双击重置为 240，宽度持久化到 ameya.sidebarWidth（关窗/松手/重置时落盘）；④凭据库改为跨平台 SystemCredentialStore（keyring 按平台选择 windows-native/apple-native/linux-native-sync-persistent 后端，非 Windows 错误文案为「系统凭据」）。已知取舍：图谱过滤重建仍会重建画布模型（视口保留、无用户可感知重置）；折叠动画 260ms 在离线快照导出中以真实等待推进。
