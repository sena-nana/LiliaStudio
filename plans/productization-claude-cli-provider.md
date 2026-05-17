# 产品化 P5：Claude CLI Provider 方案

## 对照路线图

- 路线图任务：T021 Claude CLI Provider。
- 本阶段依赖 T019 的共享进程执行器和 T020 的 CLI 错误分类。
- 本阶段只实现 Claude CLI，不改变 Codex CLI 行为。

## 目标

- 使用默认 `claude -p --output-format json` 命令模板执行无头 prompt。
- 支持用户在设置中覆盖 Claude 命令模板。
- 通过 `claude --help` 检测 Claude CLI 是否可用。
- 支持解析 JSON 输出中的文本；无法解析 JSON 时保留原始 stdout。
- 失败分类与 Codex 保持一致：`missingCli`、`authFailed`、`executionFailed`、`timedOut`。

## 设计

- 新增 `src-tauri/src/ai/claude_cli.rs`：
  - `ClaudeCliConfig` 保存命令模板、workspace、max_turns、output_format 和 timeout。
  - `check_available` 从模板解析程序路径并执行 `--help`。
  - `run_prompt` 渲染模板并通过共享执行器执行。
  - `parse_claude_output` 优先解析 JSON，支持 `result`、`text`、`content` 字段，content 数组支持 `{ "type": "text", "text": "..." }`。
  - JSON 解析失败时返回原始 stdout。
- 扩展 `src-tauri/src/commands/cli_providers.rs`：
  - 新增 `test_claude_cli_provider`，读取 Claude command template 并执行简单 prompt。
- 前端设置页：
  - Claude CLI 区块增加“测试 Claude CLI”按钮。
  - 测试结果通过 `src/api/ai.ts` 和 `aiStore` 保存。

## 测试

- Rust 测试覆盖默认模板包含 `claude -p` 和 `--output-format json`。
- Rust 测试覆盖 `claude --help` 探测和自定义程序路径。
- Rust 测试覆盖 JSON result/text/content 解析，以及 JSON 解析失败时保留 stdout。
- Rust 测试覆盖缺失 CLI、鉴权失败、执行失败、超时四类失败分类。
- 前端单测覆盖 store 调用 `test_claude_cli_provider` 并保存测试结果。

## 阶段验收

- `pnpm typecheck`
- `pnpm test:unit`
- `cd src-tauri; cargo test --locked`
- `cd src-tauri; cargo check --locked`
- `pnpm build`
