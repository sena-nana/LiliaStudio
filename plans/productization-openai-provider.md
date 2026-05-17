# OpenAI-compatible Provider 产品化阶段方案

## 阶段目标

对照路线图 T018，把现有 OpenAI-compatible 响应解析升级为真实 HTTP Provider：

- 支持 Chat Completions 风格调用。
- 支持 Embeddings 风格调用。
- 支持 `base_url`、`api_key`、`model`、`temperature`、`messages`。
- 返回统一结构化错误码，区分配置缺失、鉴权失败、HTTP 失败、模型响应解析失败和网络失败。
- 暴露测试 command，供设置页或后续工作流验证 provider 是否可用。

## 架构设计

- `src-tauri/src/ai/openai_compatible.rs` 保持 provider 核心逻辑：请求模型、响应模型、错误分类、HTTP transport trait。
- 真实 HTTP transport 使用 blocking HTTP client；测试使用内存 transport，不打真实网络。
- `src-tauri/src/commands/ai.rs` 只负责从本机设置和 Windows Credential Manager 取配置，再调用 provider。
- 前端只新增 API/store 调用入口，不把 provider 逻辑放进组件。

## 任务拆分

### P2-T001 Provider 模型与错误分类

**文件：**

- 修改：`src-tauri/src/ai/openai_compatible.rs`

**验收：**

- `ProviderError` 可序列化，字段包含 `code`、`message`、`status`。
- 401/403 归类为 `authFailed`。
- 非 2xx HTTP 响应归类为 `httpError`。
- 响应 JSON 缺失有效内容归类为 `modelResponseInvalid`。

### P2-T002 HTTP 调用与测试

**文件：**

- 修改：`src-tauri/Cargo.toml`
- 修改：`src-tauri/src/ai/openai_compatible.rs`
- 修改：`src-tauri/tests/ai_vector.rs`

**验收：**

- Chat 调用发送 Bearer token、JSON body，并解析 assistant content。
- Embeddings 调用发送文本数组，并返回向量和维度。
- 单元测试使用 fake transport 覆盖成功和错误分类。

### P2-T003 Tauri 测试命令与前端入口

**文件：**

- 修改：`src-tauri/src/commands/ai.rs`
- 修改：`src/api/ai.ts`
- 修改：`src/stores/aiStore.ts`
- 修改：`src/types/ai.ts`

**验收：**

- `test_openai_provider` 从已保存设置读取配置与密钥。
- 配置缺失或密钥缺失时返回结构化错误。
- 前端 store 可保存最近一次 provider 测试结果。

## 验证命令

```powershell
pnpm typecheck
pnpm test:unit
cd src-tauri; cargo test --locked
pnpm build
pnpm test:e2e
```

必要时补：

```powershell
cd src-tauri; cargo check --locked
pnpm tauri build
```

## 提交要求

阶段完成后更新 `plans/stage-context.md`，然后中文提交：

```text
实现OpenAI兼容Provider调用

对照开发路线图 T018，新增 OpenAI-compatible HTTP Provider、结构化错误分类、测试命令和前端入口。
```
