# Ameya

Ameya is a Windows-first worldbuilding and character logic tool.

## Architecture

- `src/`, `migrations/`, and `tests/` form the root Rust application core.
- `native/` is the NanaUI view/builder host and desktop entry point.

The repository is a native NanaUI application. There is no browser compatibility layer, web frontend, or intermediate backend package boundary.

## Development

Requirements:

- Rust stable toolchain
- CMake (for headless layout validation)
- Windows NanaUI hosted dependencies for desktop builds

Common commands:

```powershell
cargo test --workspace
cargo check -p ameya-nana --features nana-runtime
cmake -S native/validation -B build/native-validation -G "Visual Studio 17 2022" -A x64
cmake --build build/native-validation --config Debug
ctest --test-dir build/native-validation -C Debug --output-on-failure
```

## Current Capabilities

- Local projects, entries, characters, events, axioms, and relation libraries.
- Search, graph, timeline, and JSON import/export workflows.
- Optional AI providers, prompt templates, text chunking, vector retrieval, and RAG context packs.
- Logic audit, character growth, simulation, diagnostics, and help workflows.

## Native host integration

NanaUI owns the application window, view tree, layout, and command boundary. The native host dispatches typed commands to the root Rust core and does not depend on a browser runtime.
