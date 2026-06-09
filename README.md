# Ameya

Ameya is a Windows-first worldbuilding and character logic tool.

## Development

Windows dependencies:

- Microsoft C++ Build Tools
- WebView2 Runtime
- Rust MSVC toolchain
- Node.js LTS or newer
- Corepack + Yarn 4

Common commands:

```powershell
corepack enable
corepack prepare yarn@4.14.1 --activate
yarn install
yarn tauri:dev
yarn verify
```

## Current Capabilities

- Local projects, entries, characters, events, axioms, and relation libraries.
- Search, graph, timeline, and JSON import/export workflows.
- Optional AI providers, prompt templates, text chunking, vector retrieval, and RAG context packs.
- Logic audit, character growth, simulation, diagnostics, and help workflows.

## Windows Packaging

```powershell
yarn tauri:build
```

Builds are unsigned for now, so Windows SmartScreen may warn about an unknown publisher.
