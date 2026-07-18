# Ameya

Ameya is a Windows-first worldbuilding and character logic tool.

## Development

Windows dependencies:

- Microsoft C++ Build Tools
- WebView2 Runtime
- Rust MSVC toolchain
- Node.js 26.5.0
- Corepack 0.35.0 + pnpm 4.17.1

Common commands:

```powershell
npm install --global corepack@0.35.0
corepack enable pnpm pnpm install
pnpm tauri:dev
pnpm verify
```

## Current Capabilities

- Local projects, entries, characters, events, axioms, and relation libraries.
- Search, graph, timeline, and JSON import/export workflows.
- Optional AI providers, prompt templates, text chunking, vector retrieval, and RAG context packs.
- Logic audit, character growth, simulation, diagnostics, and help workflows.

## Windows Packaging

```powershell
pnpm tauri:build
```

Builds are unsigned for now, so Windows SmartScreen may warn about an unknown publisher.
