# Contributing

## Setup

```bash
git clone https://github.com/polyhook/polyhook
cd polyhook

# Rust toolchain + wasm target
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Node (for TypeScript SDK)
npm install
```

## Project Structure

```
crates/polyhook-core/   Rust core — the only place logic lives
packages/sdk-ts/        TypeScript bindings
packages/sdk-go/        Go bindings
packages/sdk-dotnet/    C# bindings
packages/sdk-python/    Python bindings
```

## Building

```bash
# Build WASM artifact (required before building any language SDK)
wasm-pack build crates/polyhook-core --target bundler --out-dir ../../polyhook.wasm

# Build all SDKs
cargo build                  # Rust (native)
npm run build -w sdk-ts      # TypeScript
go build ./...               # Go (from packages/sdk-go)
dotnet build packages/sdk-dotnet
```

## Running Tests

```bash
cargo test                   # Rust core + Rust SDK
npm test -w sdk-ts           # TypeScript SDK
go test ./...                # Go SDK (from packages/sdk-go)
dotnet test packages/sdk-dotnet
```

## Adding Support for a New AI Tool

All changes go in `crates/polyhook-core/src/`:

1. **`detect.rs`** — add detection heuristics (env vars, stdin shape) for the new tool
2. **`tools.rs`** — add vendor → canonical tool name mappings
3. **`events.rs`** — add vendor → canonical event name mappings
4. **`response.rs`** — add canonical `HookResponse` → vendor response serialization

Add the tool to the supported tools table in `README.md` and `ARCHITECTURE.md`.

Rebuild WASM after any core change — all language SDKs pick it up automatically:

```bash
wasm-pack build crates/polyhook-core --target bundler --out-dir ../../polyhook.wasm
```

## Adding a New Language Binding

1. Create `packages/sdk-<lang>/`
2. Load `polyhook.wasm` using your language's WASM runtime
3. Expose `read()` and `respond()` wrapping the WASM exports
4. Follow the host API in [BINDINGS.md](BINDINGS.md)
5. Add an entry to the SDK table in `ARCHITECTURE.md`

## Pull Requests

- One concern per PR
- Tests required for new tool support (add a fixture in `crates/polyhook-core/tests/fixtures/`)
- Update `docs/tool-names.md` for new tool name mappings
