# ameya

一个自用世界观整理工具，名字含义为雨夜。

## 开发

Windows 依赖：

- Microsoft C++ Build Tools
- WebView2 Runtime
- Rust MSVC toolchain
- Node.js LTS 或更新版本
- pnpm

常用命令：

```powershell
pnpm install
pnpm tauri dev
pnpm check
```

## 阶段能力

- 本地项目、词条、角色、事件、公理和关系资料库。
- 搜索、图谱、时间线和 JSON 导入导出。
- 可选 AI Provider、Prompt 模板、文本切片、向量检索和 RAG 上下文包。
- 逻辑审计、角色成长、模拟、诊断和帮助页的基础工作流。

## Windows 打包

```powershell
pnpm tauri build
```

当前为未签名构建，Windows SmartScreen 可能提示未知发布者；正式分发前需要配置代码签名证书。
