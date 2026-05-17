# 产品化 P3：CLI 共享执行器方案

## 对照路线图

- 路线图任务：T019 CLI Provider 抽象与进程执行器。
- 本阶段只实现 Claude/Codex 共享能力，不实现 Codex 或 Claude 的具体 Provider 适配。
- 依赖：T017 Provider 设置已完成，CLI command template 已可持久化。

## 目标

- 将命令模板渲染为明确的 `program + args` 参数数组，后续执行不经由 shell 拼接。
- 提供共享进程执行器，统一处理超时、stdout/stderr 捕获、退出码记录和结构化错误。
- 保持 AI 能力可降级：CLI 缺失、超时或执行失败时返回结构化结果，不影响本地资料库。

## 设计

- 扩展 `src-tauri/src/ai/cli_provider.rs`：
  - 保留现有模板变量：`prompt`、`workspace`、`max_turns`、`output_format`。
  - 新增 `CliInvocation`，表达解析后的程序名和参数数组。
  - 新增空命令模板错误，避免运行空程序。
- 新增 `src-tauri/src/ai/process_runner.rs`：
  - `ProcessRunSpec` 描述程序、参数、可选工作目录和超时时间。
  - `run_process` 使用 `std::process::Command` 直接传入参数数组。
  - 使用独立线程读取 stdout/stderr，避免输出管道阻塞进程退出。
  - 超时后 kill 子进程并返回 `ProcessRunErrorCode::TimedOut`。
  - 非零退出码不直接作为执行器错误；调用方可基于 `exit_code` 分类为执行失败。

## 测试

- Rust 测试覆盖模板变量渲染为安全参数数组。
- Rust 测试覆盖 stdout/stderr/exit code 捕获。
- Rust 测试覆盖超时错误分类和子进程终止。

## 阶段验收

- `pnpm typecheck`
- `pnpm test:unit`
- `cd src-tauri; cargo test --locked`
- `cd src-tauri; cargo check --locked`
- 如阶段仅涉及 Rust 后端共享逻辑，不强制运行前端 build 或 Tauri installer。
