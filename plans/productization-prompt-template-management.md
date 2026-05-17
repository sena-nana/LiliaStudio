# 产品化 P7：Prompt 模板管理实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** 对照路线图 T023，提供可重置内置模板、可复制编辑用户模板、变量说明和最终 prompt 预览的管理界面。

**Architecture:** 沿用现有 `services/prompts.rs` 与 `commands/prompts.rs`，把 Prompt 模板作为本地 SQLite 资料管理能力，不接真实 AI 工作流。后端负责内置模板重置、用户副本保存和变量渲染；前端通过独立 `src/api/promptTemplates.ts`、`promptTemplateStore` 和 `PromptTemplateView` 访问，不在组件中散落 `invoke`。

**Tech Stack:** Tauri 2、Vue 3、TypeScript、Pinia、rusqlite、Vitest、Cargo tests。

---

## 文件边界

- Modify: `src-tauri/src/services/prompts.rs`，扩展 PromptTemplate 类型、内置模板定义、复制、编辑、重置和预览逻辑。
- Modify: `src-tauri/src/commands/prompts.rs`，新增 copy/save/reset/preview commands。
- Modify: `src-tauri/src/lib.rs`，注册新增 prompt commands。
- Create: `src-tauri/tests/prompt_templates.rs`，覆盖内置重置、复制编辑、变量说明、预览渲染。
- Modify: `src/api/ai.ts`，移除 prompt template API，避免重复入口。
- Create: `src/api/promptTemplates.ts`，封装模板管理 commands。
- Create: `src/stores/promptTemplateStore.ts`，集中管理列表、选中、表单、预览。
- Modify: `src/types/ai.ts`，补充 PromptTemplateDraft、变量、预览类型。
- Create: `src/views/PromptTemplateView.vue`，实现管理页面。
- Modify: `src/components/layout/AppShell.vue`、`src/router/index.ts`、`src/styles/theme.css`，加入导航、路由和页面样式。
- Create: `tests/unit/promptTemplateStore.spec.ts`，覆盖前端 store command 契约。
- Modify: `plans/stage-context.md`，阶段完成后记录 P7 结果和下一阶段决策。

## Task 1：后端 Prompt 模板服务

- [ ] **Step 1: 写失败测试**

Create `src-tauri/tests/prompt_templates.rs`:

```rust
use ameya_lib::{
    db::{connection::open_memory_database, migrations::run_migrations},
    services::prompts::{
        copy_prompt_template, list_prompt_templates, preview_prompt_template,
        reset_builtin_prompt_templates, save_prompt_template, PromptTemplateDraft,
        PromptTemplatePreviewRequest, PromptTemplateVariableValue,
    },
};

#[test]
fn builtin_templates_reset_and_expose_variable_descriptions() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");

    let templates = list_prompt_templates(&connection).expect("templates");
    assert!(templates.iter().any(|item| item.purpose == "completion_questions"));
    assert!(templates.iter().any(|item| {
        item.variables.iter().any(|variable| variable.name == "project_context")
    }));

    let reset = reset_builtin_prompt_templates(&connection).expect("reset");
    assert!(reset.iter().all(|item| item.built_in));
    assert!(reset.len() >= 4);
}

#[test]
fn users_can_copy_and_edit_templates_without_mutating_builtins() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");
    let builtin = list_prompt_templates(&connection).expect("templates")[0].clone();

    let copy = copy_prompt_template(&connection, &builtin.id).expect("copy");
    assert!(!copy.built_in);
    assert_ne!(copy.id, builtin.id);

    let saved = save_prompt_template(
        &connection,
        PromptTemplateDraft {
            id: Some(copy.id.clone()),
            name: "自定义审计".into(),
            purpose: "logic_audit".into(),
            template: "上下文：{{project_context}}\n目标：{{target_entity}}".into(),
        },
    )
    .expect("save");

    assert_eq!(saved.name, "自定义审计");
    assert!(saved.variables.iter().any(|variable| variable.name == "target_entity"));
}

#[test]
fn prompt_preview_renders_known_values_and_reports_missing_variables() {
    let preview = preview_prompt_template(PromptTemplatePreviewRequest {
        template: "上下文：{{project_context}}\n问题：{{question}}\n目标：{{target_entity}}".into(),
        values: vec![
            PromptTemplateVariableValue {
                name: "project_context".into(),
                value: "北境粮食规则".into(),
            },
            PromptTemplateVariableValue {
                name: "question".into(),
                value: "是否冲突".into(),
            },
        ],
    });

    assert!(preview.prompt.contains("北境粮食规则"));
    assert_eq!(preview.missing_variables, vec!["target_entity"]);
}
```

- [ ] **Step 2: 验证 RED**

Run: `cd src-tauri; cargo test --locked --test prompt_templates`

Expected: FAIL because the new service functions and structs do not exist.

- [ ] **Step 3: 实现后端服务**

Modify `src-tauri/src/services/prompts.rs`:

- Add `PromptTemplateVariable`, `PromptTemplateDraft`, `PromptTemplateVariableValue`, `PromptTemplatePreviewRequest`, `PromptTemplatePreview`.
- Replace the hard-coded three built-ins with a `builtin_prompt_templates()` registry containing at least `logic_audit`、`completion_questions`、`character_analysis`、`behavior_audit`、`simulation`。
- Add `variables` to each returned `PromptTemplate` by parsing `{{variable_name}}` placeholders and attaching descriptions/examples from a local registry.
- Implement:
  - `reset_builtin_prompt_templates(connection) -> rusqlite::Result<Vec<PromptTemplate>>`
  - `copy_prompt_template(connection, template_id) -> rusqlite::Result<PromptTemplate>`
  - `save_prompt_template(connection, draft) -> rusqlite::Result<PromptTemplate>`
  - `preview_prompt_template(request) -> PromptTemplatePreview`

- [ ] **Step 4: 验证 GREEN**

Run: `cd src-tauri; cargo test --locked --test prompt_templates`

Expected: PASS.

## Task 2：Commands、API 和 Store

- [ ] **Step 1: 写失败测试**

Create `tests/unit/promptTemplateStore.spec.ts`:

```ts
import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { usePromptTemplateStore } from '@/stores/promptTemplateStore'

const invokeMock = vi.mocked(invoke)

describe('promptTemplateStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('loads templates and selects the first template', async () => {
    invokeMock.mockResolvedValueOnce([
      { id: 'prompt_1', name: '逻辑审计', builtIn: true, variables: [] },
    ])

    const store = usePromptTemplateStore()
    await store.loadTemplates()

    expect(store.templates[0].id).toBe('prompt_1')
    expect(store.selectedTemplate?.id).toBe('prompt_1')
    expect(invokeMock).toHaveBeenCalledWith('list_prompt_templates')
  })

  it('copies, saves, resets, and previews through commands', async () => {
    invokeMock
      .mockResolvedValueOnce({ id: 'prompt_copy', builtIn: false })
      .mockResolvedValueOnce({ id: 'prompt_copy', name: '自定义审计', builtIn: false })
      .mockResolvedValueOnce([{ id: 'prompt_builtin', builtIn: true }])
      .mockResolvedValueOnce({ prompt: '最终 prompt', missingVariables: [] })

    const store = usePromptTemplateStore()
    await store.copyTemplate('prompt_1')
    await store.saveTemplate({
      id: 'prompt_copy',
      name: '自定义审计',
      purpose: 'logic_audit',
      template: '{{project_context}}',
    })
    await store.resetBuiltins()
    await store.preview('{{question}}', [{ name: 'question', value: '是否冲突' }])

    expect(invokeMock).toHaveBeenCalledWith('copy_prompt_template', { templateId: 'prompt_1' })
    expect(invokeMock).toHaveBeenCalledWith('save_prompt_template', {
      draft: {
        id: 'prompt_copy',
        name: '自定义审计',
        purpose: 'logic_audit',
        template: '{{project_context}}',
      },
    })
    expect(invokeMock).toHaveBeenCalledWith('reset_builtin_prompt_templates')
    expect(store.previewResult?.prompt).toBe('最终 prompt')
  })
})
```

- [ ] **Step 2: 验证 RED**

Run: `pnpm test:unit -- tests/unit/promptTemplateStore.spec.ts`

Expected: FAIL because `promptTemplateStore` and API wrappers do not exist.

- [ ] **Step 3: 实现 commands/API/store**

- Modify `src-tauri/src/commands/prompts.rs` and `src-tauri/src/lib.rs` to expose:
  - `list_prompt_templates`
  - `copy_prompt_template`
  - `save_prompt_template`
  - `reset_builtin_prompt_templates`
  - `preview_prompt_template`
- Create `src/api/promptTemplates.ts` with one wrapper per command.
- Create `src/stores/promptTemplateStore.ts` with actions `loadTemplates`、`selectTemplate`、`copyTemplate`、`saveTemplate`、`resetBuiltins`、`preview`。
- Move `listPromptTemplates` imports in `src/stores/aiStore.ts` from `src/api/ai.ts` to `src/api/promptTemplates.ts` and remove the duplicate from `src/api/ai.ts`。

- [ ] **Step 4: 验证 GREEN**

Run: `pnpm test:unit -- tests/unit/promptTemplateStore.spec.ts`

Expected: PASS.

## Task 3：Prompt 模板管理页面

- [ ] **Step 1: 实现 UI**

Create `src/views/PromptTemplateView.vue`:

- Left list: built-in/custom templates, selected state, copy button.
- Editor: name、purpose、template textarea; built-ins read-only with copy first; custom templates can save.
- Variable panel: show variable name, description, example; clicking an example fills preview values.
- Preview panel: user-edited variable values and rendered prompt from backend preview command.

Modify:

- `src/router/index.ts`: add `/prompt-templates` route.
- `src/components/layout/AppShell.vue`: add sidebar link `模板`。
- `src/styles/theme.css`: add compact prompt template layout classes.
- `src/views/SettingsView.vue`: replace the old prompt list block with a link or short status pointing to the new page.

- [ ] **Step 2: 验证 UI 编译**

Run: `pnpm typecheck`

Expected: PASS.

## Task 4：阶段验证与提交

- [ ] **Step 1: 完整验证**

Run:

```powershell
pnpm typecheck
pnpm test:unit
cd src-tauri; cargo test --locked
cd src-tauri; cargo check --locked
pnpm build
pnpm test:e2e
```

Expected: all pass. If Playwright creates `test-results/`, remove it before staging.

- [ ] **Step 2: 更新上下文并提交**

Modify `plans/stage-context.md`:

- Add “产品化 P7 已实现”
- Record verification results.
- Set next stage to T024 文本切片 DocumentChunk unless route map shows a higher-priority blocker.

Commit:

```powershell
git add ...
git commit -m "实现Prompt模板管理" -m "写明内置重置、用户副本、变量说明、预览和验证结果。"
```
