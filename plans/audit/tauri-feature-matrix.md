# Tauri 2 + Vue 3 前端功能矩阵（Parity Contract）

- **基准版本**: git HEAD = `d6093cf` ("Add MIT License")——最后一个包含 Tauri Vue 前端的完整状态。
  （初版任务指定 `883e4e1`，已按 lead 修正为 HEAD；HEAD 相对 883e4e1 新增 Agent 问答、语义搜索、Embedding 索引、关系邻域、角色成长工作台、对象浏览器版项目工作台。下文全部按 HEAD 描述，883e4e1 的差异单独标注。）
- **技术栈**: Tauri 2 (Wry, 无装饰窗口) + Vue 3 (Composition API, `<script setup>`) + Pinia (options API stores) + vue-router (hash history) + TipTap 富文本。`main.ts` 挂载 `#root`，全局安装 context-menu 指令。
- **应用名**: Ameya，v0.1.0，identifier `local.ameya.app`，NSIS perMachine 安装。
- **窗口**: label `main`，1280×800，min 960×640，decorations:false（自绘 TitleBar），backgroundColor #181818，`visible:false` 启动隐藏直到窗口状态恢复后 `window.show()`。

---

## 1. PER-VIEW FEATURE MATRIX

路由根 `/` → AppShell，子路由全部由 `src/features/{home,workspace,tools}.ts` 聚合。所有业务视图遵循同一模式：`projectId = route.params.projectId ?? projectStore.activeProjectId`（可回退）；无项目时显示 `未选择项目` 空态。视图间跳转靠 hash 路由（`/projects/:id`、`/graph/:id`…）。

### 1.1 HomeView（`/`，名称 home）
- **用途**: 项目列表与快速创建（"项目库"）。
- **布局**: view-header（eyebrow `项目库` / h1 `项目` / 右侧 `新建项目` 按钮）→ quick-create 表单 → 项目列表。
- **控件**:
  - `新建项目` → `createProject({name:"未命名宇宙", description:""})` + 跳转 `/projects/:id`。
  - 表单: `项目名称`(required) + `描述` + `创建` 提交 → project_create 后跳转。
  - 每行: 打开按钮（名称 + `description || "无描述"`，active 态跟随 activeProjectId）→ `selectProject` + 跳转；行内编辑: `项目名称`/`描述` 输入 + `保存`(project_update) + `归档`(project_archive，从列表移除并重指 active)。
- **数据**: projectStore（project_list/project_create/project_update/project_archive）。
- **状态**: loading `正在加载`；空 `还没有项目`。

### 1.2 ProjectView（`/projects/:projectId?`，HEAD 重写为"对象工作台"）
- **用途**: 项目资料库五类对象的浏览/编辑/关系检视（883e4e1 是 5 列面板并排；HEAD 改为三栏对象浏览器）。
- **布局**: header（eyebrow `对象工作台` / h1 项目名，error 状态条）→ `.object-workspace` 三栏:
  1. **LibraryCollectionRail**（左栏 `对象`）: 5 个集合按钮 `资料(n) 角色(n) 事件(n) 规则(n) 关系(n)`——注意 label 用 资料/规则（非 词条/公理）。
  2. **中栏** = LibraryObjectBrowser + LibraryRecordEditor:
     - Browser: h2 集合名 + `N 个对象`；entry 集合时显示 EntryTemplatePanel（world_rule 世界规则/item 物品/location 地点/faction 阵营）；`新增` 按钮（relation 且实体<2 时禁用）。对象表 5 列: 标题 / 类型标签 / 状态 / `N 关联` / updatedAt(`未记录` 兜底)。行 meta: entry=`类型 · 状态`；character=阵营或 `未分配阵营`；event=timeLabel 或 `未定时间`；axiom=`predicate = object`；relation=confidence<0.6→`低置信度` 否则 `已确认`。
     - RecordEditor（LibraryEditorShell 包裹: 标题 词条/角色/事件/公理/关系 或 `资料`；`删除`+`保存` 无选中禁用；空态 `未选择`）字段与 883e4e1 相同:
       - entry: 标题/类型/状态 输入 + 标签 输入 + `摘要`/`正文` TipTap 富文本（Suspense 兜底 `编辑器加载中`）。
       - character: 姓名/阵营/别名/标签 + 摘要(3行)/外貌(4)/目标(4)/动机(4)/恐惧(4)。
       - event: 标题/时间/排序(number)/地点/开始/结束/重要度(0-10)/标签/描述(4)/结果(4)。
       - axiom: 主体/谓词/对象/确定性(0-1 step .05)/时间范围/地点范围/来源类型/来源标识/标签/自然语言(5行)。
       - relation: 来源/目标 下拉（entityOptions，键 `type:id`，标签 `词条：标题/角色：名/事件：标题/公理：主体`）、关系类型 输入 + datalist `#relation-type-presets`（预置: 参与事件/发生于/属于阵营/导致/支撑规则/约束/来源于/冲突候选/语义相关，来自 library_relation_type_presets，失败回退前端常量）、置信度(0-1)、有方向 checkbox、描述(5行)。
  3. **LibraryObjectInspector**（右栏）:
     - 实体选中时: eyebrow 实体类型 + 标题 + subtitle(`未分类` 兜底) + 关系数；`摘要`(`暂无摘要` 兜底)；`已有关联` 边列表（点击边跳到另一端实体；空 `暂无已确认关系`）；`待确认建议`（`relationType · strength` + `src -> tgt` + reason + `确认` 按钮→create relation；空 `暂无待确认建议`）；`缺失项` chips（空时 `结构完整`）；`邻域预览` 节点按钮网格（点击选中实体）。数据来自 library_relation_neighborhood(projectId, target, depth=1)。
     - relation 选中时: 置信度(2位小数) + `连接` 两端 entityLabel。
     - 空态 `未选择`。
- **新记录工厂**: 新词条 N+1（按 entryType 模板预填 summary/body/tags，status draft）；新角色 N+1；新事件 N+1（sortKey=Date.now()）；公理 新主体/定义/新对象（certainty 1）；关系 取前两个实体、类型 `关联`（confidence 1, directed true）。
- **深度链接**: 路由 query `?kind=<kind>&id=<id>` 选中对象（GraphView 跳转用）。
- **标签/别名解析**: split `/[,，]/` 去空；来源类型/标识 trim 后空→null。
- **数据**: libraryStore + library_* 命令族 + library_relation_type_presets + library_relation_neighborhood；表单状态抽到 useLibraryRecordForms/useLibraryWorkspace。

### 1.3 SearchView（`/search/:projectId?`）
- **用途**: 关键词 + 语义混合搜索（"搜索"）。
- **控件**: 输入框 placeholder `搜索` + `搜索` 按钮（loading 或无项目禁用）。
- **流程**: searchStore.run → `Promise.all([search_entities, search_semantic(limit 8)])`；语义失败降级不抛错。
- **结果行**: 标题 + 来源 chip（`关键词`/`语义`，语义用 accent 色）+ `entityTypeLabel · (语义时 "相似度 X.XX") · snippet||无摘要`；关键词结果优先，语义按 entityType:entityId 去重追加。
- **状态**: 无项目 `未选择项目`（提示 `先选择一个项目，再执行关键词和语义搜索。`）；loading `正在搜索`；语义 degraded/failed → status-note（failed 为 error 样式）显示 semanticMessage；空结果 `暂无结果` + `输入查询后会先显示关键词结果，再补充语义命中。`

### 1.4 AgentChatView（`/agent/:projectId?`，HEAD 新实现"项目问答"）
- **用途**: 基于项目资料的 AI 问答（RAG-lite：前端检索 → 引用传入 ai_agent_ask）。
- **布局**: header（eyebrow `智能体` / h1 `项目问答` / 右侧 `项目` 下拉，含 `未选择` 选项，选中即 selectProject）→ 问答表单 → 状态行 → 回答卡 → 引用列表。
- **控件**:
  - `问题` textarea rows4，placeholder `基于当前项目资料提问`（asking 或无项目禁用）。
  - `提交`（asking 时文案 `生成中`；无项目/空问题/asking 禁用）。
  - `保存为报告` / `保存为任务`（均需已有回答；busy 禁用）。
- **提交流程** (useAgentChatWorkspace): ① searchStore.run(混合检索) ② 取前 8 条结果转 AgentAskReference ③ `ai_agent_ask {projectId, question, references}` ④ 显示 answer + providerKindLabel · model。
- **保存为报告**: library_create_entry，entryType `report`，title `Agent 问答：<问题前28字>...`，summary=问题，body=Markdown（`# 问题/# 回答/# 引用来源` 编号列表，无引用时 `无引用来源`），tags `[agent, report]`，status active → 状态 `已保存为本地报告`。
- **保存为任务**: job_create {providerKind:"openAiCompatible", jobType:"agentChat", inputSummary:问题} → 状态 `已保存为任务`。（注意：后端 jobs 服务只是 SQLite 状态机，没有执行器——任务只是排队记录。）
- **状态**: 无项目 `未选择项目`/`先选择一个项目，再提交问题。`；statusMessage/errorMessage/semanticNotice（语义 degraded|failed 时显示 semanticMessage）状态行。
- **引用列表**: header `引用来源` + `N 条`；行: 标题 + chip（语义/关键词）+ `entityTypeLabel · (语义: 相似度 X.XX) · snippet||无摘要`；score 保留两位。

### 1.5 GraphView（`/graph/:projectId?`，HEAD 重写为真实图）
- **用途**: 关系图谱（"关系邻域"）。
- **工具栏**: `关系类型` 下拉（`全部关系` + 出现过的类型去重排序）、`实体类型` 下拉（`全部实体` + entityTypeLabel）、`清除聚焦` 按钮（无聚焦禁用）。
- **画布**: SVG viewBox 0 0 100 100 画边（line，聚焦时非相关边 muted）；节点为绝对定位按钮（title + `entityTypeLabel · N 关联`），布局为同心环（中心 1 个 + 每环 12 个，半径 18+12*环号，上限 40）；单击聚焦/取消，**双击跳转** `/projects/:id?kind=<type>&id=<id>`。
- **侧栏**: `真实关系`（`N 条边 · N 个节点`）；聚焦节点卡（title/subtitle + `编辑对象` 按钮）；`边` 列表（relationType + `src -> tgt`/` - `（无向），点击跳 `/projects/:id?kind=relation&id=<edgeId>`）。
- **状态**: 无项目；空 `暂无匹配关系`；error 状态条。
- **数据**: libraryStore 全量 relations + 本地布局计算（domain/relationGraph.ts）；不调用后端图命令。

### 1.6 TimelineView（`/timeline/:projectId?`）
- **用途**: 事件演化时间线。events 按 sortKey 升序；每条: `<time>` timeLabel||`未定时间` + 标题 + `description||outcome||暂无描述`。onMounted loadProject。

### 1.7 BackupView（`/backup/:projectId?`）
- **用途**: 项目导入导出。`导出数据` → archive_export_project → pretty JSON 写入 textarea（placeholder `归档数据`）；`导入为新项目` → JSON.parse → archive_import_project → 前端 prepend + 设 active。

### 1.8 IndexingView（`/indexing/:projectId?`，HEAD 升级为 embedding 索引面板）
- **控件**: `重建` 按钮（loading 或无项目禁用）→ aiStore.rebuildEmbeddingIndex(projectId, 600)。
- **指标卡 5 张**: `切片` chunkCount / `Embedding` embeddingCount / `缺失` missingEmbeddingCount / `过期` staleEmbeddingCount / `模型` model||`未生成`。
- **状态行**: loading `正在重建索引`；ready `当前 embedding 索引可用`；degraded→message；failed→message（error 样式）；idle `尚未执行索引`。进入视图立即 rag_index_status（按 lastProjectId 过滤显示）。
- **降级逻辑**（aiStore.rebuildEmbeddingIndex）: openAiCompatible 未启用/缺 key/baseUrl/embeddingModel → 只建切片（rag_index_chunks）+ 状态 degraded + 提示 `请先在 AI 设置中启用并配置 OpenAI 兼容接口的嵌入模型和密钥。`；就绪则 rag_index_embeddings（切片→embed→upsert）→ 刷新状态。
- **空态**: `未选择项目` + `先选择一个项目，再重建切片和 embedding 索引。`

### 1.9 AuditReportView（`/audit/:projectId?`）
- `运行样例审计` → logic_audit_facts（硬编码 2 条公理: 月光金属 state=solid/liquid, 第三纪/北方 → 互斥公理冲突）+ logic_repair_suggestions(conflicts[0])。渲染冲突卡（conflictTypeLabel 互斥公理 + message）与修复建议卡（title + `description · impact`）。纯演示无项目依赖。

### 1.10 CharacterGrowthView（`/growth/:projectId?`，HEAD 新实现"成长工作台"）
- **布局**: 左栏 `角色`（`N 个` + 行: 名称 + 阵营||未分配阵营，按名称 zh-Hans-CN 排序）→ 中间三块（当前 trait 状态 / 输入表单 + 预览 / 来源记录）→ 右栏结果状态。
- **表单**（`来源事件输入`: `选择项目内事件，填写本次成长影响。`）: `来源事件` 下拉（`请选择事件` + `title · timeLabel||未定时间`，按 sortKey 排序）、`Trait 名称`（placeholder `例如 responsibility`）、`Delta`（decimal，`例如 0.35`）、`原因` textarea（`描述事件如何影响角色 trait`）；按钮 `预览变化`（previewing 时 `预览中`）/ `保存记录`（saving 时 `保存中`，需先预览）。
- **流程**: character_growth_preview_trait_delta(state, delta) 纯计算 → 预览卡（事件 title + trait ±delta + 全量预览 trait 网格 + 原因）→ character_growth_create_record 落库 → character_growth_workspace 重载（records + states 映射）。
- **状态卡**: 顶部 h2 角色名 + summary||`暂无角色摘要` + chips `已保存记录 N`/`来源事件 N`；trait 卡 name + 有符号值（±X.XX，服务端 clamp [-1,1]）。`来源记录` 列表倒序: 事件名 + trait ±delta + reason||`未填写原因` + `timeLabel||未定时间 · createdAt`。
- **右栏**: `结果状态`（statusLabel: 空闲/预览中/已生成预览/保存中/已保存/失败）+ `当前角色` + `输入检查` chips（事件/Trait/Delta/原因，满足变 ok）+ `提示`: `第一版不会校验该事件是否真的包含当前角色参与记录。`
- **空态**: 未选项目 / `正在加载`（正在读取角色、事件和成长记录。）/ `暂无角色`（先在资料库里创建角色…）/ `暂无事件`（先创建项目事件…）。

### 1.11 SimulationView（`/simulation/:projectId?`）
- `运行样例模拟` → simulation_run(activeProjectId||`project_sample`, `如果北方发生饥荒`, `[粮食, 北方城墙]`)；渲染 scenario h2 + 每阶段 `label：summary`。（后端为规则化推演，不调 AI。）

### 1.12 JobsView（`/jobs`）
- **用途**: 人工智能后台队列。header `刷新` 按钮 → job_list + job_current。
- **任务卡**: h2 jobTypeLabel（logicAudit 逻辑审计/indexProject 项目索引/promptRun 提示词运行/agentChat Agent 问答——agentChat 在 HEAD displayLabels 新增）、`providerKindLabel · jobStatusLabel`（排队中/运行中/已完成/失败/已取消）、按钮 `日志`(job_logs)/`取消`(job_cancel)/`重试`(job_retry，新建 queued 记录 retryOfJobId 指向原任务，日志 `已从 <id> 重试`)；正文 inputSummary；错误 errorMessage；日志列表 `logLevelLabel message`（信息/警告/错误）。

### 1.13 PromptTemplateView（`/prompt-templates`）
- **布局**: header（`刷新` + `重置内置`）→ 三栏: 模板列表（name + `purposeLabel · 内置|自定义`）｜编辑器（h2 name||`未选择模板` + `内置模板/自定义模板` 标签；按钮 `复制`/`预览`/`保存`——builtIn 只读，仅副本可存；字段 名称/用途/模板 textarea14）｜预览面板（`变量`: 从模板正则 `/\{\{\s*([A-Za-z0-9_]+)\s*\}\}/g` 提取，输入框 placeholder=example；`预览`: `填充示例` 按钮 + pre prompt||template + `缺少变量：a、b`）。
- purposeLabel: logic_audit→逻辑审计 / completion_questions→补完问题 / character_analysis→角色分析 / behavior_audit→行为校验 / simulation→叙事模拟。
- **命令**: prompt_list_templates / prompt_copy_template / prompt_save_template / prompt_reset_builtin_templates / prompt_preview_template{request:{template, values}}。

### 1.14 DiagnosticsView（`/diagnostics`）
- `刷新` → diagnostics_summary；显示 platformLabel（Windows/Linux/macOS）+ `版本 appVersion · SQLite`。（后端仅返回 app_version=CARGO_PKG_VERSION / platform=std::env::consts::OS / database="sqlite"，无导出或脱敏逻辑。）

### 1.15 HelpView（`/help`）
- 静态卡片: 项目/词条/审计/人工智能。

### 1.16 Settings 页（`/settings?tab=`，meta sidebar:"settings" lockSidebar:true）
- **Settings.vue**: h1 = tab label；副标题 `管理 Ameya 的外观、人工智能提供方、命令行提供方和应用信息。`；渲染对应 Section。
- **AppearanceSection（外观）**: `主题` segmented radiogroup `暗色`/`浅色`（hint `选择应用配色，立即生效并记忆到本地。`）→ useTheme.setTheme；`语言`: 固定文案 `简体中文`。
- **AiSettingsSection（人工智能设置）**: 副题 `配置 OpenAI 兼容接口、模型和密钥。`；顶部 `保存`。每个 openAiCompatible 供应商块: `启用` checkbox + 状态 hint 已启用/已停用；字段 `基础地址`(placeholder https://api.example.com/v1)/`对话模型`(gpt-4.1)/`嵌入模型`(text-embedding-3-small)/`接口密钥`(password, autocomplete=new-password, placeholder `已保存：<preview>` 或 `未保存`)/`清除已保存密钥` checkbox；`测试` 按钮 → ai_test_openai_provider，结果 span ok/error + message（成功 `Provider 测试调用成功` / 空响应 `Provider 已响应，但返回内容为空`）。保存 → ai_save_provider_settings（apiKey 空串→null 不覆盖）→ 重载 → `设置已保存` 或错误文案。
- **CliProvidersSection（命令行提供方）**: 副题 `配置 Codex 命令行和 Claude 命令行的命令模板。`；每块 `启用` + `命令模板` textarea3（占位模板见 §5）+ `测试`（ai_test_codex_cli_provider / ai_test_claude_cli_provider，成功文案 `Codex CLI 测试调用成功`/`Claude CLI 测试调用成功`，失败含 CliProviderError code: missingCli/authFailed/executionFailed/timedOut + exitCode + output）。
- **AboutSection（关于）**: kv: 名称 Ameya / 版本 0.1.0 / 用途 `面向 Windows 的世界观与角色逻辑工具` / 技术栈 `Tauri 2 + Vue 3 + Pinia`。

---

## 2. SHELL FEATURES

- **AppShell**: `.shell`（class is-resizing / is-sidebar-collapsed / is-settings-mode；CSS 变量 --sidebar-width）= TitleBar + 侧栏（settings 模式用 SettingsSidebar，否则 SecondaryPanel）+ 拖拽分隔条（role=separator，title `拖动调整侧栏宽度（双击恢复默认）`，pointerdown 拖动、dblclick 复位 240px）+ main RouterView。route.meta.lockSidebar 时禁折叠/禁拖。
- **主侧栏（SecondaryPanel）**，自上而下:
  1. 全局动作 icon 行: `新建项目`→/projects、`搜索`→/search。
  2. `工作区` 组（header 带禁用的 `添加` 工具钮）主导航: 项目(/)、资料库(/projects)、角色(/growth)、事件(/timeline)、规则(/audit)、关系(/graph)、分析(/search)。
  3. `项目工具` 组（工具钮 `更多` 禁用）: 索引(/indexing)、备份(/backup)、模拟(/simulation)、智能体(/agent)。
  4. SidebarFooter: icon 链接 设置(/settings)、帮助(/help)、诊断(/diagnostics)、任务(/jobs)、提示词(/prompt-templates, title 提示词模板) + 状态胶囊 `本地优先`（tone ok，title `本地优先工作区。打开人工智能后台队列。`，链接 /jobs）。
- **SettingsSidebar**: `返回` 按钮（useRouteReturnTarget 记录进入设置前的路由，兜底 `/`）+ 4 个 tab（外观/人工智能设置/命令行提供方/关于，/settings?tab=key，默认 appearance）。
- **TitleBar**: 自绘（decorations:false）。左: 折叠左侧栏/展开左侧栏 toggle（aria-pressed，locked 禁用）；中: 品牌名 Ameya（data-tauri-drag-region）；右: 最小化 / 最大化↔还原（isMaximized 由 onResized 同步，图标 Square/Copy）/ 关闭（danger hover）。全部走 @tauri-apps/api/window。
- **CommandPalette**（App 根挂载）: **Ctrl+K** 开关，Escape 关闭；命令=全局动作+主导航+footer 链接（label 取 title??label）: 新建项目/搜索/项目/资料库/角色/事件/规则/关系/分析/设置/帮助/诊断/任务/提示词模板。无过滤输入框——纯列表，首项自动聚焦，点击 router.push。
- **上下文菜单**（App 级）: window contextmenu preventDefault + DOM 上冒泡找 provider（v-context-menu 指令或 registerContextMenu）；条目 {id?, label, icon?, disabled?, danger?, confirmLabel?, onSelect}；confirmLabel 条目需点两次（第一次换成 confirmLabel 并 danger 样式）；外部 pointerdown/Escape/scroll/resize/blur 关闭；Teleport 渲染，视口内钳位。
- **快捷键汇总**: Ctrl+K 命令面板；Esc 关闭面板/菜单/对话框；富文本编辑器继承 TipTap StarterKit 键位（Mod-B/I、Mod-Shift-1? 实为二级标题按钮、列表、引用、Mod-Z/Mod-Shift-Z）。无其他全局快捷键。
- **主题**: useTheme——"dark"|"light"，localStorage `ameya.theme`，默认 dark，写 documentElement.dataset.theme。全站 CSS 变量双主题。
- **持久化 key（localStorage）**: `ameya.theme`、`ameya.sidebarWidth`（数字 180–480，默认 240）、`ameya.sidebarCollapsed`（"1"/"0"，默认展开）。
- **其他组件**: ConfirmDialog（open/title/message/confirmText 确认/cancelText 取消/danger/busy/busyText 处理中...，Esc+遮罩取消）；Dropdown（pill+listbox，top/bottom placement，外点+Esc 关闭）；ViewTabs（概览/看板[即将上线]/待办[即将上线]——**未挂载的残留**）；JobStatusBar（/jobs 链接，`运行中：类型 · 提供方` 或 `任务空闲`，5s 轮询 job_current——**未挂载的残留**）；RichTextEditor（TipTap StarterKit: 加粗/斜体/二级标题/无序列表/有序列表/引用/撤销/重做，modelValue 为 JSON doc 字符串，EMPTY_DOCUMENT=`{"type":"doc","content":[{"type":"paragraph"}]}`，纯文本自动转 doc）。
- **错误处理**: executeCommand 统一 catch → ApiError{code:"TAURI_COMMAND_FAILED", message}；各视图以 status-note/error 展示。

---

## 3. STORES（Pinia，8 个）

| Store | state | 关键 actions → 命令 |
| --- | --- | --- |
| projects | projects, activeProjectId, loading, error; getter activeProject | loadProjects(project_list, 自动选第一个)/createProject/updateProject/archiveProject(重指 active)/selectProject/replaceProject |
| library | entries, characters, events, axioms, relations, loading, error | loadProject(library_project_snapshot 一次拉全量)；五类 create/update/delete（create 前插、update 替换、delete 过滤；event 带 participants 参数默认 []）；集合工具 collection.ts |
| ai | providers, providerSettings, chunks, preview, jobs, prompts, openAi/codexCli/claudeCliProviderTest, **indexState: EmbeddingIndexState**(HEAD 新增), loading | loadDefaults(ai_default_providers)/loadProviderSettings/saveProviderSettings/test×3/loadPromptsAndJobs(prompt_list_templates+job_list)/createJob/previewText(vector_preview_chunks)/indexProject(rag_index_chunks)/**loadIndexStatus(rag_index_status, 失败→failed+message)/rebuildEmbeddingIndex(条件 rag_index_embeddings, 未配置→仅切片+配置提示语)** |
| jobs | jobs, logsByJobId, activeJob, loading | loadJobs(job_list)/refreshActiveJob(job_current)/loadLogs(job_logs)/createJob(job_create, running 时设 activeJob)/cancelJob(job_cancel)/retryJob(job_retry 前插新记录) |
| promptTemplates | templates, selectedTemplate, previewResult, loading | loadTemplates(自动选首项)/selectTemplate/copyTemplate/saveTemplate/resetBuiltins(builtins+customs 拼接)/preview |
| search | query, keywordResults, semanticResults, loading, semanticStatus(idle/ready/degraded/failed), semanticMessage; getter results=关键词优先+语义去重(带 source 标记) | run → searchEntities + searchSemantic(limit 8, catch→failed 降级) 并行 |
| workflows | conflicts, repairs, simulation, diagnostics | runSampleAudit(logic_audit_facts 硬编码样例+logic_repair_suggestions)/runSampleSimulation(simulation_run 固定场景)/loadDiagnostics(diagnostics_summary) |
| （HEAD 无新 store；新工作台逻辑放 composables） | useAgentChatWorkspace / useCharacterGrowthWorkspace / useLibraryWorkspace(+useLibraryRecordForms) / useRelationGraphWorkspace | 分别封装 AgentChat/CharacterGrowth/Project/Graph 视图状态 |

---

## 4. COMMAND SURFACE（全部 49 个 Tauri 命令，src-tauri/src/commands/mod.rs register_handlers 全集）

args/returns 为 camelCase serde。`State<AppState>` 为 SQLite 连接（app_data_dir 下库文件）；`app.path().app_config_dir()` 用于 AI 设置。

**ai（commands/ai.rs + cli_providers.rs）**
- `ai_default_providers()` → Vec<AiProviderConfig>（codex_cli + claude_cli 默认模板，均 enabled）。
- `ai_load_provider_settings(app)` → Vec<AiProviderSettingsView>（读 config_dir/ai-provider-settings.json + Credential Manager 校验 hasApiKey/apiKeyPreview）。
- `ai_save_provider_settings(app, drafts)` → Vec<AiProviderSettingsView>（见 §5 secret 规则；与默认模板 merge 后写回 json）。
- `ai_test_openai_provider(app)` → OpenAiProviderTestResult{ok, message, error:ProviderError|null}。实现: provider_resolver 解析（缺配置→error_result(configMissing 类消息)）→ OpenAiCompatibleClient(UreqOpenAiTransport).chat(model, "Return the word ok.", temperature 0.0)。
- `ai_agent_ask(app, request:AgentAskRequest)` → AgentAskResponse{answer, references(回传), providerKind:"openAiCompatible", model}。实现: 空问题→Err"请输入问题"；resolve_openai_chat_provider(OPENAI_AGENT_MESSAGES)（未启用→"请先在 AI 设置中启用 OpenAI 兼容接口"）；messages = [system: 常量 AGENT_SYSTEM_PROMPT（"你是 Ameya 的项目资料问答助手。只能基于用户提供的项目上下文回答；…"）, user: `项目 ID：…\n\n问题：…\n\n项目上下文：\n[1] entry 标题（entry:id，来源：keyword，分数：2.00）\n摘要…`（无引用→"无可用引用。请明确说明上下文不足。"）]；chat(model, messages, temperature **0.2**)，错误取 error.message。**仅 openAiCompatible 一种 provider 路径；codex/claude CLI 不参与 agent ask。**
- `ai_test_codex_cli_provider(app)` / `ai_test_claude_cli_provider(app)` → CliProviderTestResult{ok, message, error{code:missingCli|authFailed|executionFailed|timedOut, message, exitCode}, output}。实现: 在 app_data_dir 建 workspace 目录，命令模板取用户设置否则默认，max_turns=1，timeout 60s，StdProcessRunner 跑 `test_prompt()`。

**provider_resolver（ai/provider_resolver.rs）**: load_openai_settings(app) → (settings, apiKey)（apiKey 从 Credential Manager 读，空白过滤）；resolve_openai_chat_provider / resolve_openai_embedding_provider（按 OPENAI_{TEST,AGENT,RAG,SEARCH}_MESSAGES 上下文化中文报错，如 "请先填写 OpenAI 兼容接口的基础地址/对话模型/嵌入模型/接口密钥"）；resolve_optional_openai_embedding_model（静默 Option，用于 rag_index_status 的 model 字段）。

**archive**: `archive_export_project(projectId)` → ProjectArchive{version, project, entries, characters, events, axioms, relations}；`archive_import_project(archive)` → ImportedProject{project}。

**diagnostics**: `diagnostics_summary()` → {appVersion: CARGO_PKG_VERSION, platform: OS, database:"sqlite"}。无其他采集、无导出。

**health**: `health_check(app)` → {appVersion, platform, appDataDir}。

**job（services/jobs.rs 纯 SQLite 状态机，无执行 worker）**: `job_create(draft)`（status=queued）/ `job_list()` / `job_current()`（运行中的那条，可 null）/ `job_logs(jobId)` / `job_cancel(jobId)`（status=cancelled + cancel_requested_at + finished_at + 日志"任务已取消"）/ `job_retry(jobId)`（复制 draft 新建 queued 记录，retry_of_job_id=原 id，日志"已从 <id> 重试"）。服务层另有 mark_job_running/complete_job_success/fail_job（错误信息 sanitize）供（未来的）执行器调用——**当前二进制没有任何后台执行循环；agent 问答是同步内联调用，"保存为任务"只落一条 queued 记录**。

**library**: `library_project_snapshot(projectId)` → {projectId, entries, characters, events, axioms, relations}；五类 `library_{create,update,delete}_{entry,character,event,axiom,relation}`（event 带 participants: EventParticipantDraft[]{entityType, entityId, role}；delete 均为 soft_delete 置 deleted_at）；`library_list_backlinks(target:EntityRef)` → Vec<Relation>（前端 api 有、视图未用）；`library_relation_type_presets()` → 9 个预置类型；`library_relation_neighborhood(projectId, target, depth)` → RelationNeighborhood{center:EntitySummary, nodes, edges, suggestions:RelationSuggestion{…, reason, strength}, missing:string[], relationCount}（services/relation_workspace.rs 聚合邻域边 + 规则式建议）。

**logic**: `logic_audit_facts(facts)` → Vec<LogicConflict>{conflictType(互斥 mutually_exclusive_axioms), factIds, message}；`logic_repair_suggestions(conflict)` → Vec<RepairSuggestion>{title, description, impact}。纯内存规则（logic/conflict.rs + quickxplain.rs + repair.rs）。

**project**: `project_list()`（仅 active，不含 archived）/ `project_create(draft)` / `project_update(id, draft)` / `project_archive(id)`（软删 archivedAt）。

**prompt**: `prompt_list_templates()` / `prompt_copy_template(templateId)`（副本可编辑）/ `prompt_save_template(draft)`（id null=新建）/ `prompt_reset_builtin_templates()`（重建内置，保留自定义）/ `prompt_preview_template(request:{template, values})` → {prompt, missingVariables[]}（{{VAR}} 替换）。

**rag**: `rag_index_chunks(projectId, maxChars)` → Vec<DocumentChunkRecord>（chunking: entry/character/event/axiom 文本按 maxChars 切片 + contentHash + estimatedTokens）；`rag_index_status(projectId)` → RagIndexStatus{chunkCount, embeddingCount, missingEmbeddingCount, staleEmbeddingCount, model, status("ready"|"degraded"—— 序列化排除 idle/loading/failed), message}（未配置嵌入模型→degraded+"请先启用 OpenAI 兼容接口并填写嵌入模型。"；0 切片→"当前项目还没有生成切片索引。"；全缺→"当前项目还没有可用的 embedding 索引。"；部分缺/过期→"当前 embedding 索引不完整或已过期。"；stale 判定=embedding.updated_at < chunk.updated_at 或 dimensions<=0）；`rag_index_embeddings(projectId, maxChars)` → {chunkCount, embeddingCount, model}（先切片→解析 embedding provider→批量 embeddings API→逐条 upsert_embedding(向量存 SQLite)→复检状态；向量数与切片数不一致→Err"嵌入模型返回的向量数量与切片数量不一致"）；`rag_preview_context_pack(projectId, query, queryVector:f32[])` → ContextPack（top 8 余弦排序，前端 api 有、视图未用）。

**search**: `search_entities(filter:{projectId, query, entityTypes})` → Vec<SearchResult>{entityType, entityId, title, snippet, score}（SQLite LIKE 关键词）；`search_semantic(request:{projectId, query, limit})` → SemanticSearchResponse{status:"ready"|"degraded"|"failed", message, model, items}（空 query→ready 空数组；provider 未就绪→degraded；索引非 ready→degraded+索引消息；embed query→candidates(按 model)→余弦 top limit→回表组 SearchResult；无候选→degraded"当前项目还没有可用的 embedding 索引。"）。

**simulation**: `simulation_run(projectId, scenario, referencedEntities)` → SimulationReport{projectId, scenario, phases[{label, summary}], risks[], referencedEntities}（规则推演）。

**vector**: `vector_preview_chunks(text, maxChars)` → Vec<TextChunk>{ordinal, text, contentHash, estimatedTokens}。

**characterGrowth**: `character_growth_preview_trait_delta(state, delta)` → 新 state（values[trait] = clamp(old+delta, -1, 1)，sources 追加）；`character_growth_workspace(projectId)` → {records: Vec<CharacterTraitDeltaRecord>, states: Record<characterId, CharacterTraitState>}（按 character 聚合）；`character_growth_create_record(draft)`（校验 character/event 存在且未删——ensure_active_growth_links）。

---

## 5. WINDOWS-SPECIFIC

- **windows/credential.rs — WindowsCredentialStore（必须移植）**: 实现 ai/settings.rs 的 `trait SecretStore { read_secret/write_secret/delete_secret }`。基于 keyring_core + windows_native_keyring_store：`ensure_windows_store()` 惰性初始化默认 store（windows_native_keyring_store::Store::new() → keyring_core::set_default_store）。Entry::new(**SERVICE_NAME="ameya.ai"**, name)。read: get_password，NoEntry→Ok(None)，其他错→"读取 Windows 凭据失败：{e}"；write→set_password（"保存 Windows 凭据失败"）；delete→delete_credential，NoEntry 视为成功（"删除 Windows 凭据失败"）。
- **secret 命名**: `api_key_secret_name(kind)` = `ameya_` + slug，slug: openAiCompatible→`openAiCompatibleApiKey`、codexCli→`codexCliApiKey`、claudeCli→`claudeCliApiKey`。即 Credential Manager 条目 (service="ameya.ai", account="ameya_openAiCompatibleApiKey" 等)。
- **设置文件**: app_config_dir/**ai-provider-settings.json**，存 StoredAiProviderSettings{kind, baseUrl, **hasApiKey(bool，不存明文)**, chatModel, embeddingModel, commandTemplate, enabled}；不存在→默认三 provider（merge_with_defaults 保证三系都在）。save 逻辑: clearApiKey→delete_secret；apiKey 非空→write_secret；其余字段 normalize（trim 空串→None）。
- **apiKeyPreview**: mask_secret——长度≤8→`********`；否则 `前3-********后4`。
- **CLI 默认命令模板**（占位符 {workspace} {prompt} {max_turns}）:
  - codex: `codex exec --cd "{workspace}" --ask-for-approval never --sandbox workspace-write "{prompt}"`
  - claude: `claude -p "{prompt}" --output-format json --cwd "{workspace}" --max-turns {max_turns}`
- **window_state.rs（必须移植）**: tauri_plugin_store JSON 文件 app_data_dir/**main-window-state.json**，key `mainWindow`，结构 {x, y, width, height, maximized}。启动: load（is_restorable_main_window_state: width≥960 && height≥600 才恢复）→ set_position/set_size/(maximize) → window.show()（配合 tauri.conf visible:false 防闪白）。关闭/销毁事件: capture(outer_position+inner_size+is_maximized) → merge（**maximized 快照保留上次正常几何**，避免存成 -8,-8 全屏值）→ store.save。MIN 960×600（窗口配置 minWidth 960/minHeight 640）。
- **lib.rs 启动链**: tauri_plugin_opener + tauri_plugin_store 插件；setup 里 AppState::initialize(app_data_dir)（SQLite）+ manage + 主窗 set_background_color(#181818) + 恢复窗口态 + show。

---

## 6. DATA CONTRACTS（camelCase，前端 types ↔ Rust serde rename_all="camelCase" 一一对应）

- **Project**: `{id, name, description, createdAt, updatedAt, archivedAt}`；Draft `{name, description}`。
- **BaseEntity**（五类库对象共用）: `{id, projectId, createdAt, updatedAt, deletedAt}`。
- **Entry**: +`{entryType, title, summary(富文本JSON), body(富文本JSON), tags[], status}`（draft 同形 + projectId；status: draft/active/archived；entryType: world_rule/item/location/faction/report…自由字符串）。
- **Character**: +`{name, aliases[], summary, appearance, goals, motivations, fears, faction, tags[]}`。
- **EventRecord**: +`{title, description, timeLabel, sortKey:number, startLabel, endLabel, location, importance:number, outcome, tags[]}`；EventParticipantDraft `{entityType, entityId, role}`。
- **Axiom**: +`{subject, predicate, object, scopeTime, scopeLocation, certainty:number, sourceEntityType:string|null, sourceEntityId:string|null, naturalLanguage, tags[]}`。
- **EntityRef**: `{entityType, entityId}`；**Relation**: `{id, projectId, source:EntityRef, target:EntityRef, relationType, description, confidence:number, directed:boolean, createdAt, updatedAt, deletedAt}`。
- **EntitySummary** `{entityType, entityId, title, subtitle, summary}`；**RelationSuggestion** `{source, target, relationType, description, confidence, directed, reason, strength}`；**RelationNeighborhood** `{center, nodes[], edges:Relation[], suggestions[], missing:string[], relationCount}`；**LibraryProjectSnapshot** `{projectId, entries, characters, events, axioms, relations}`。
- **AiProviderKind**: `"openAiCompatible"|"codexCli"|"claudeCli"`。**AiProviderSettingsView**: `{kind, baseUrl, apiKeyPreview:string|null, hasApiKey:boolean, chatModel, embeddingModel, commandTemplate, enabled}`（均为 string|null 除 kind/enabled）；**Draft**: `{kind, baseUrl, apiKey:string|null, clearApiKey:boolean, chatModel, embeddingModel, commandTemplate, enabled}`。
- **ProviderError** `{code:"configMissing"|"authFailed"|"httpError"|"networkError"|"modelResponseInvalid", message, status:number|null}`；**OpenAiProviderTestResult** `{ok, message, error|null}`；**CliProviderError** `{code:"missingCli"|"authFailed"|"executionFailed"|"timedOut", message, exitCode:number|null}`；**CliProviderTestResult** `{ok, message, error|null, output:string|null}`。
- **AgentAskReference** `{entityType, entityId, title, snippet, source:"keyword"|"semantic"|string, score:number(f32)}`；**AgentAskRequest** `{projectId, question, references[]}`；**AgentAskResponse** `{answer, references[], providerKind, model}`。
- **AiJob** `{id, projectId:string|null, providerKind, jobType, status:"queued"|"running"|"succeeded"|"failed"|"cancelled", inputSummary, outputText, errorMessage:string|null, startedAt, finishedAt, cancelRequestedAt, retryOfJobId:string|null, createdAt, updatedAt}`；**AiJobDraft** `{projectId:null-able, providerKind, jobType, inputSummary}`；**AiJobLog** `{id, jobId, level:"info"|"warning"|"error", message, createdAt}`。
- **TextChunk** `{ordinal, text, contentHash, estimatedTokens}`；**DocumentChunkRecord** `{id, projectId, sourceType, sourceId, ordinal, text, contentHash, estimatedTokens, updatedAt}`；**EmbeddingIndexResult** `{chunkCount, embeddingCount, model}`；**RagIndexStatus** `{chunkCount, embeddingCount, missingEmbeddingCount, staleEmbeddingCount, model, status:"ready"|"degraded", message}`（Rust 端还有 idle/loading/failed 供前端 store 态）；**EmbeddingIndexState** = RagIndexStatus 字段 + `status` 五态 + `lastProjectId:string|null`（仅前端）。
- **ContextPack** `{projectId, query, items:[{sourceType, sourceId, text, score}]}`。
- **SearchFilter** `{projectId, query, entityTypes[]}`；**SearchResult** `{entityType, entityId, title, snippet, score}`；**SemanticSearchRequest** `{projectId, query, limit}`；**SemanticSearchResponse** `{status:"ready"|"degraded"|"failed", message, model, items:SearchResult[]}`；**MixedSearchResult** = SearchResult + `source:"keyword"|"semantic"`（仅前端 getter）。
- **PromptTemplate** `{id, name, purpose, template, builtIn:boolean, variables:[{name, description, example}], createdAt, updatedAt}`；Draft `{id:string|null, name, purpose, template}`；VariableValue `{name, value}`；Preview `{prompt, missingVariables[]}`。
- **Fact** `{id, subject, predicate, object, scopeTime, scopeLocation}`；**LogicConflict** `{conflictType, factIds[], message}`；**RepairSuggestion** `{title, description, impact}`。
- **TraitDelta** `{sourceEventId, traitName, delta:number, reason}`；**CharacterTraitState** `{values:Record<string, number>, sources:TraitDelta[]}`；**CharacterTraitDeltaRecord** `{id, projectId, characterId, sourceEventId, traitName, delta, reason, createdAt, updatedAt, deletedAt}`；Draft 同形去审计字段；**CharacterGrowthWorkspaceSnapshot** `{records[], states:Record<characterId, CharacterTraitState>}`。
- **SimulationReport** `{projectId, scenario, phases:[{label, summary}], risks[], referencedEntities[]}`。
- **DiagnosticsSummary** `{appVersion, platform, database}`；**HealthInfo** `{appVersion, platform, appDataDir}`。
- **ProjectArchive** `{version:number, project, entries[], characters[], events[], axioms[], relations[]}`；**ImportedProject** `{project}`。
- **Result&lt;T&gt;**（types/result.ts，未实际用于命令返回）: `{ok:true, data}|{ok:false, error:{code, message}}`。
- **富文本契约**: entry.summary/body 存 TipTap JSON doc 字符串；非 JSON 文本自动包成 paragraphs（richTextContent.ts）。

---

## 附：HEAD vs 883e4e1 差异清单（parity 以 HEAD 为准）

- 新增命令: ai_agent_ask / rag_index_status / rag_index_embeddings / search_semantic / library_relation_type_presets / library_relation_neighborhood / character_growth_workspace / character_growth_create_record。
- 重写视图: ProjectView（五列面板→CollectionRail+ObjectBrowser+RecordEditor+ObjectInspector 三栏对象工作台）、GraphView（静态卡→SVG 边+同心环节点+过滤/聚焦/双击跳转）、AgentChatView（stub→完整问答）、CharacterGrowthView（stub→成长工作台）、IndexingView（切片计数→五指标卡+状态机）、SearchView（纯关键词→混合搜索）。
- 新增前端基建: composables useAgentChatWorkspace/useCharacterGrowthWorkspace/useLibraryWorkspace/useLibraryRecordForms/useRelationGraphWorkspace；domain agentChat.ts/relationGraph.ts；组件 LibraryCollectionRail/LibraryObjectBrowser/LibraryObjectInspector/LibraryRecordEditor。
- displayLabels 新增: entryType `report→报告`、jobType `agentChat→Agent 问答`。
- 其余（shell/设置/任务/模板/备份/时间线/审计/诊断/帮助、stores 结构、持久化 key）与 883e4e1 相同。
