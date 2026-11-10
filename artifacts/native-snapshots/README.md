# Native offline page snapshots

这些文件由 `cargo run -p ameya-nana --features nana-runtime -- --render-pages artifacts/native-snapshots` 一次性生成（16 个路由 × expanded/collapsed 两种侧栏状态，输出到 `expanded/`、`collapsed/`）。

- 16 个路由 × expanded/collapsed 两种侧栏状态。
- `.snapshot` 的 `window/group/control` 记录来自 **真实 hosted NanaUI 布局**：导出器用离屏 GPU 上下文构造 `HeadlessRuntime<RuntimeApplication<AmeyaApp>>`，逐路由 `Navigate` + `present` 后经 `project_accessibility_in_viewport` 读取无障碍树的真实几何（1280×800、scale 1.0）。页面数据由 `AMEYA_SNAPSHOT_SEED=1` 注入确定性样例（示例项目/月光金属/阿离等），保证输入控件非空且可复现。`section/field/list/status/empty/action` 业务记录仍由 `content_records(route)` 按内容契约生成。
- `.png` 是可读性审阅产物，由 `native/validation/render_snapshots.py` 基于上述 `.snapshot` 业务记录渲染（Microsoft YaHei，避免 CJK tofu）；它是离线验收的光栅化适配器，不是应用 UI 实现，也不是 GPU 像素基线。颜色统一来自 `native/validation/nana_theme.py`，其值严格镜像 NanaUI `nana_ui_core::SemanticPalette` 与 `AccentRamp`。
- `metadata.json` 登记 route、control→command 映射、每个快照与 PNG 的 SHA-256、viewport、字体与生成器信息；`contentAcceptance` 业务验收契约与 `content-acceptance.tsv` 对应。
- `content-acceptance.tsv` 是 16 个 route × expanded/collapsed 的可审计内容契约；每个快照必须包含 `section`（业务区块）、`field`（字段/值/状态）、`list`（列表/空态引用）、`status`（当前状态）和 `empty`（可见空态）记录，且字段控件的 action 映射必须与 `action` 记录一致。

布局与业务验收：`cmake -S native/validation -B build/native-validation -G "Visual Studio 17 2022" -A x64`，随后 `cmake --build build/native-validation --config Debug && ctest --test-dir build/native-validation -C Debug --output-on-failure`；其中 `native.offline_content_acceptance` 会逐一检查 32 个快照的五类业务 records 和 action 映射，per-snapshot layout 测试检查真实几何的包含/重叠约束。
