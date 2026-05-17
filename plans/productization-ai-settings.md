# AI 设置产品化阶段方案

## 阶段目标

把现有“加载默认 AI Provider 模板”的骨架升级为可长期使用的本机设置系统：

- 支持 OpenAI-compatible、Claude CLI、Codex CLI 三类 provider 的配置读取与保存。
- 非敏感配置落盘到 Tauri app config 目录。
- API Key 不写入 SQLite 或 JSON 配置，改存 Windows Credential Manager。
- 前端设置页提供可编辑表单，并只显示脱敏后的密钥状态。
- 无密钥、无 CLI 时，本地资料库、搜索、审计和模拟的规则型能力仍可运行。

## 架构设计

Rust 后端拆成两层：

- `ai::settings` 负责 provider 设置模型、默认值、JSON 文件读写、脱敏与保存合并规则。
- `windows::credential` 负责 Windows Credential Manager 读写，外部只看到 `SecretStore` 行为，不把系统 API 泄漏到业务层。

前端继续遵守现有边界：

- `src/api/ai.ts` 调用 Tauri command。
- `src/stores/aiStore.ts` 管理设置状态。
- `src/views/SettingsView.vue` 只负责表单和展示，不直接调用 `invoke`。

## 任务拆分

### P1-T001 后端设置模型与测试

**文件：**

- 修改：`src-tauri/src/ai/settings.rs`

**验收：**

- 缺少配置文件时返回三类默认 provider。
- 保存时 JSON 只包含非敏感字段和 `has_api_key`。
- 加载时返回 `api_key_preview`，不返回明文密钥。
- 清除密钥时删除 secret store 中的对应记录。

### P1-T002 Windows Credential Manager 后端

**文件：**

- 新增：`src-tauri/src/windows/mod.rs`
- 新增：`src-tauri/src/windows/credential.rs`
- 修改：`src-tauri/Cargo.toml`
- 修改：`src-tauri/src/lib.rs`

**验收：**

- 使用 Windows Credential Manager 存储 OpenAI-compatible API Key。
- 业务层通过 trait 调用，测试可使用内存 secret store。
- 密钥读写失败时返回可展示的中文错误。

### P1-T003 Tauri 命令与前端 API

**文件：**

- 修改：`src-tauri/src/commands/ai.rs`
- 修改：`src-tauri/src/lib.rs`
- 修改：`src/types/ai.ts`
- 修改：`src/api/ai.ts`
- 修改：`src/stores/aiStore.ts`

**验收：**

- `load_ai_provider_settings` 返回脱敏设置。
- `save_ai_provider_settings` 保存设置并返回最新脱敏设置。
- store 测试覆盖加载、保存、密钥不回显。

### P1-T004 设置页表单

**文件：**

- 修改：`src/views/SettingsView.vue`
- 修改：`src/styles/theme.css`

**验收：**

- 页面可编辑 OpenAI base URL、模型、API Key、Claude/Codex 命令模板和启用状态。
- 保存后 UI 显示脱敏密钥，不显示明文。
- 用户可勾选清除 OpenAI API Key。

## 验证命令

每个阶段完成前运行：

```powershell
pnpm typecheck
pnpm test:unit
cd src-tauri; cargo test --locked
pnpm build
```

涉及依赖变更后补充：

```powershell
cd src-tauri; cargo check --locked
```

## 提交要求

阶段完成后更新 `plans/stage-context.md`，然后中文提交：

```text
产品化AI设置持久化

实现 provider 设置读写、Windows Credential Manager 密钥存储、前端设置表单和测试。
```
