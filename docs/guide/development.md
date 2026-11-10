# 开发启动

## 项目结构

```text
ameya/
├── src/                 # Rust application core
│   ├── domain/          # 领域模型与仓储
│   ├── services/        # 搜索、RAG、任务与 AI 服务
│   ├── logic/           # 冲突检测与修复
│   └── lib.rs
├── migrations/          # SQLite schema migrations
├── native/              # Nana 原生宿主与 view/builder
│   └── validation/       # 离屏布局 smoke tests
└── tests/               # Rust core integration tests
```

## 本地验证

仓库只包含 Rust 核心和 NanaUI 原生宿主，不需要 Node、浏览器或兼容层。

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo check -p ameya-nana --features nana-runtime
cargo test -p ameya-nana --features nana-runtime
cmake -S native/validation -B build/native-validation -G "Visual Studio 17 2022" -A x64
cmake --build build/native-validation --config Debug
ctest --test-dir build/native-validation -C Debug --output-on-failure
```

离屏快照（`artifacts/native-snapshots/`）由真实 hosted NanaUI 布局导出：`snapshot_export.rs` 用离屏 GPU 上下文构造 `HeadlessRuntime`，逐路由读取无障碍树几何（client-area 坐标），CMake 验证窗口、分组、控件边距及同组重叠。重新生成：

```bash
mkdir -p native/target/tmp
TMP=native/target/tmp TEMP=native/target/tmp \
  cargo run -p ameya-nana --features nana-runtime -- \
  --render-pages artifacts/native-snapshots
```

## 命令边界

NanaUI 宿主通过 `native/src/host.rs` 将 typed command dispatch 到根 Rust core。所有命令保持 camelCase 数据契约；没有宿主或浏览器 fallback。桌面应用（`nana-runtime`）在 `native/src/{app,shell,state,commands,views}` 组织：`ApplicationState` 驱动 `Store<AppState>`，网络类命令（AI 调用、嵌入构建、语义检索）经后台任务与 `AppMessage` 回流。
