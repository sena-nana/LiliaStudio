# 产品化 P6：AI 后台任务队列与日志方案

## 对照路线图

- 路线图任务：T022 AI 后台任务队列与日志。
- 依赖：T018 OpenAI-compatible、T020 Codex CLI、T021 Claude CLI。
- 当前仓库已有 `services/jobs.rs` 和 `commands/jobs.rs` 骨架，本阶段沿用现有 services/commands 分层，不引入平行的后端 jobs 目录。

## 目标

- 将 AI job 从单一 queued 记录扩展为完整状态机：queued、running、succeeded、failed、cancelled。
- 增加 job 日志表，只记录摘要、状态变化和错误，不记录 API Key 或完整 prompt。
- 支持取消和重试 job；取消 running job 时更新取消请求和终态，后续 CLI 运行器可根据取消信号 kill 子进程。
- 前端提供 jobs API、jobStore、底部状态栏和 Jobs 页面。

## 设计

- 新增迁移 `0002_ai_job_queue.sql`：
  - 扩展 `ai_jobs`：started_at、finished_at、cancel_requested_at、retry_of_job_id。
  - 新增 `ai_job_logs`：job_id、level、message、created_at。
- 扩展 `src-tauri/src/services/jobs.rs`：
  - `AiJobStatus` 枚举映射 DB 文本。
  - 状态转换函数：mark running、complete success、fail、cancel、retry。
  - `append_job_log` 统一脱敏 `sk-`、`api_key`、`Authorization` 相关内容。
- 扩展 `src-tauri/src/commands/jobs.rs`：
  - 新增 list logs、current running、cancel、retry commands；运行、成功、失败状态转换保留在 service 层供后续调度器使用。
- 前端新增：
  - `src/api/jobs.ts`
  - `src/stores/jobStore.ts`
  - `src/components/jobs/JobStatusBar.vue`
  - `src/views/JobsView.vue`
  - 路由和侧边栏入口。

## 测试

- Rust 测试覆盖：状态转换、日志脱敏、取消、重试、当前 running job。
- 前端单测覆盖：jobStore list/current/cancel/retry 调用。

## 阶段验收

- `pnpm typecheck`
- `pnpm test:unit`
- `cd src-tauri; cargo test --locked`
- `cd src-tauri; cargo check --locked`
- `pnpm build`
- `pnpm test:e2e`
- 说明：未连接真实调度器前，队列层先提供显式状态更新和日志能力，便于后续工作流接入。
