# 开发启动

## 项目结构

```text
ameya/
├── src/                 # Vue 3 前端
│   ├── layouts/         # AppShell / SecondaryPanel / SettingsSidebar
│   ├── components/      # TitleBar / ViewTabs / SidebarFooter 等
│   ├── pages/           # Home / Settings / Plugins
│   ├── composables/     # useTheme / useResizablePane 等
│   ├── router.ts
│   └── styles.css
├── src-tauri/           # Tauri 2 Rust 端
├── tests/               # Vitest + Testing Library
└── scripts/             # 本地开发脚本
```

## 本地运行

本仓库固定使用 Node.js 26.5.0、Corepack 0.35.0 和 pnpm 4.17.1。Node.js 26 不再内置 Corepack,首次使用需显式安装。建议从仓库根目录运行贡献命令。

```bash
npm install --global corepack@0.35.0
corepack enable pnpm pnpm install
pnpm dev
pnpm tauri:dev
```

`pnpm tauri:dev` 会自动寻找可用本地端口,再把对应 `devUrl` 传给 Tauri。

## 验证

```bash
pnpm test
pnpm build
cargo check --manifest-path src-tauri/Cargo.toml
pnpm verify
```

按影响范围运行最小必要验证。涉及构建配置、壳层布局、路由或 Tauri 端改动时,优先运行 `pnpm verify`。

## 图标

Tauri 图标位于 `src-tauri/icons/`。如需替换图标,先更新源图,再使用 Tauri CLI 重新生成平台图标。
