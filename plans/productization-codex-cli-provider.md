# 产品化 P4：Codex CLI Provider 方案

## 对照路线图

- 路线图任务：T020 Codex CLI Provider。
- 本阶段依赖 T019 的 CLI 模板和共享进程执行器。
- 本阶段只实现 Codex CLI，不实现 Claude CLI。

## 目标

- 使用默认 `codex exec` 命令模板执行无头 prompt。
- 支持用户在设置中覆盖 Codex 命令模板。
- 通过 `codex --help` 检测 Codex CLI 是否可用。
- 对用户触发的测试调用返回文本结果或结构化错误。

## 设计

- 扩展 `src-tauri/src/ai/cli_provider.rs`：
  - 增加共享 CLI Provider 错误码：`missingCli`、`authFailed`、`executionFailed`、`timedOut`。
  - 这些错误码供 Codex 和后续 Claude 复用。
- 扩展 `src-tauri/src/ai/process_runner.rs`：
  - 增加 `ProcessRunner` trait 和 `StdProcessRunner`，生产环境使用真实进程，测试使用 fake runner。
- 新增 `src-tauri/src/ai/codex_cli.rs`：
  - `CodexCliConfig` 保存命令模板、workspace、max_turns、output_format 和 timeout。
  - `check_available` 运行 `codex --help`。
  - `run_prompt` 渲染模板并通过共享执行器执行。
  - 非零退出码按输出内容分类：登录、鉴权、401、unauthorized 等归为 `authFailed`，其他归为 `executionFailed`。
- 新增 `src-tauri/src/commands/cli_providers.rs`：
  - `test_codex_cli_provider` 从设置读取 Codex command template。
  - workspace 使用应用数据目录，避免测试命令在源码目录写入内容。
- 前端设置页：
  - Codex CLI 区块增加“测试 Codex CLI”按钮。
  - 测试结果通过 `src/api/ai.ts` 和 `aiStore` 统一管理。

## 测试

- Rust 测试覆盖 `codex --help` 探测使用明确参数数组。
- Rust 测试覆盖 prompt 模板渲染和 stdout 文本返回。
- Rust 测试覆盖四类失败分类：缺失 CLI、鉴权失败、执行失败、超时。
- 前端单测覆盖 store 调用 `test_codex_cli_provider` 并保存测试结果。

## 阶段验收

- `pnpm typecheck`
- `pnpm test:unit`
- `cd src-tauri; cargo test --locked`
- `cd src-tauri; cargo check --locked`
- `pnpm build`
