# NanaUI native 验证

`validation/` 提供离屏几何与内容验收。`artifacts/native-snapshots/` 下的 `.snapshot` 由真实 hosted NanaUI 布局导出（`native/src/snapshot_export.rs`：离屏 GPU 上下文 + `HeadlessRuntime` + 无障碍树几何投影），验证分组、布局、边距、控件包含与核心操作可用性。

## 最小运行

默认构建使用 headless view builder，桌面发行版启用 `nana-runtime`（真实 NanaUI hosted runtime）。在仓库根目录执行：

```bash
cmake -S native/validation -B build/native-validation -G "Visual Studio 17 2022" -A x64
cmake --build build/native-validation --config Debug
ctest --test-dir build/native-validation -C Debug --output-on-failure
cargo test --manifest-path native/Cargo.toml
cargo check --manifest-path native/Cargo.toml --features nana-runtime
```

重新生成快照（Windows/Git Bash 下把 TMP/TEMP 指到 Windows 路径，避免 tempfile 权限问题）：

```bash
mkdir -p native/target/tmp
TMP=native/target/tmp TEMP=native/target/tmp \
  cargo run -p ameya-nana --features nana-runtime -- \
  --render-pages artifacts/native-snapshots
```

该 smoke 不创建窗口、不依赖显示服务器，适合 CI 和 Windows 无桌面会话。成功输出类似：

```text
rendered 16 routes × 2 modes to artifacts/native-snapshots
```

验证目录不要求修改共享构建配置。

## Native command host

`src/host.rs` exposes `NativeCommandHost`, the command boundary consumed by the frontend Nana bridge. It initializes the existing SQLite migrations and preserves camelCase JSON contracts for health checks, project CRUD, library snapshots, and diagnostics. The domain, repository, search, RAG, jobs, simulation, and logic modules are compiled into this host from the framework-neutral root `ameya-core` crate.

## 快照契约

每行一个记录，字段以 `|` 分隔，注释以 `#` 开头：

```text
window|x=0|y=0|w=800|h=600
group|id=main|x=16|y=16|w=768|h=568|left=16|top=16|right=16|bottom=16
control|id=run-check|group=main|x=32|y=80|w=120|h=32|visible=1|enabled=1
action|id=run-check|supported=1
```

验证规则：

- `window`、至少一个 `group`、`control`、`action` 必须存在，尺寸必须为正数。
- 分组必须位于窗口内，边距非负且不能吞掉整个分组。
- 控件必须引用已有分组、可见且启用，必须位于分组内容区（扣除四边边距）内；同组控件不得重叠。
- `action|supported=1` 表示核心功能（例如运行校验、保存、打开项目）在当前快照中可用；缺失或不支持会使 CTest 失败。

NanaUI 侧建议使用布局计算完成后的坐标（而不是请求尺寸）导出快照。导出时统一使用 client-area 坐标系，并在窗口 resize、主题切换和核心操作前后各采样一次。后续若要做像素级离屏图像比较，再单独增加图像基线；几何 smoke 不应依赖字体、DPI 或 GPU。
