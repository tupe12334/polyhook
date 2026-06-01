# Architecture

## Overview

The core logic — tool detection, event normalization, response serialization — is written once in Rust and compiled to WebAssembly. Every language binding is a thin shim over that same WASM module.

```
┌─────────────────────────────────────────┐
│           polyhook-core (Rust)          │
│  detection · normalization · serde      │
└──────────────────┬──────────────────────┘
                   │ wasm-pack / wasm-bindgen
                   ▼
           polyhook.wasm
          ╱      │      ╲
    TypeScript   Go    C#   Python  ...
    (thin shim)  (thin shim)  (thin shim)
```

No logic is re-implemented per language. All SDKs call the same WASM binary, so behavior is identical across every runtime.

---

## Repository Structure

```
polyhook/
├── crates/
│   └── polyhook-core/     # Rust: detection, normalization, serde — compiled to WASM + native
├── packages/
│   ├── sdk-ts/            # TypeScript bindings (wasm-bindgen)
│   ├── sdk-go/            # Go bindings (Wazero)
│   ├── sdk-dotnet/        # C# bindings (Wasmtime)
│   └── sdk-python/        # Python bindings (wasmtime-py)
└── polyhook.wasm          # built artifact, bundled into each SDK package
```

---

## polyhook-core

`crates/polyhook-core` is the single source of truth. It handles:

- **Caller detection** — identifies which AI tool invoked the binary from stdin shape and environment variables
- **Event normalization** — maps vendor-specific event/tool names to the canonical polyhook schema
- **Deserialization** — parses the caller's stdin JSON into a `HookEvent`
- **Serialization** — encodes a `HookResponse` into the format the caller expects on stdout

Compiled targets:
- `wasm32-unknown-unknown` — bundled into every language SDK
- Native (lib) — used directly by the Rust SDK, zero WASM overhead

---

## Language SDKs

Each SDK is a thin host binding. It:

1. Loads `polyhook.wasm` into a WASM runtime
2. Passes raw stdin bytes into the WASM module
3. Gets back a normalized `HookEvent` struct
4. After user logic runs, encodes the `HookResponse` via the WASM module
5. Writes the result to stdout

| SDK | WASM runtime |
|---|---|
| TypeScript (`@polyhook/sdk`) | wasm-bindgen (browser / Node.js) |
| Go (`polyhook-go`) | Wazero |
| C# (`Polyhook.Sdk`) | Wasmtime |
| Python (`polyhook`) | wasmtime-py |
| Rust (`polyhook`) | native — links `polyhook-core` directly |

Any language with a WASM runtime can bind polyhook. See [BINDINGS.md](BINDINGS.md) for the raw WASM host API.

---

## Caller Detection

Detection runs in priority order:

1. `POLYHOOK_CALLER` env var — explicit override, highest priority
2. Environment variables set by the invoking tool (e.g. `CLAUDE_CODE_VERSION`, `CURSOR_SESSION_ID`)
3. Stdin schema heuristics — shape of the top-level JSON object
4. Falls back to `caller: "unknown"` with best-effort parse

---

## Tool Name Normalization

Each AI tool uses different names for the same operation. The mapping table in `polyhook-core` translates vendor names to canonical polyhook names at parse time.

| polyhook name | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `bash` | `Bash` | `run_terminal_cmd` | `run_command` | `execute_command` | `shell` |
| `read_file` | `Read` | `read_file` | `read_file` | `read_file` | `file.read` |
| `write_file` | `Write` | `edit_file` | `write_file` | `write_to_file` | `file.write` |
| `edit_file` | `Edit` | `apply_edit` | `edit_file` | `apply_diff` | `file.edit` |
| `web_search` | `WebSearch` | `web_search` | `search_web` | `search` | `web.search` |
| `web_fetch` | `WebFetch` | `fetch_url` | `fetch_page` | `fetch` | `web.fetch` |

Full table: [docs/tool-names.md](docs/tool-names.md)

---

## Event Mapping

| polyhook event | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `tool:before` | `PreToolUse` | `BeforeToolCall` | `pre_tool` | `beforeToolUse` | `tool.before` |
| `tool:after` | `PostToolUse` | `AfterToolCall` | `post_tool` | `afterToolUse` | `tool.after` |
| `session:start` | `Startup` | `SessionStart` | `session_start` | `onStart` | `session.start` |
| `session:stop` | `Stop` | `SessionEnd` | `session_end` | `onStop` | `session.stop` |
| `agent:stop` | `SubagentStop` | — | — | — | `agent.stop` |
| `notification` | `Notification` | `Notification` | `notification` | — | — |

---

## Adding a New Tool

All changes go in `crates/polyhook-core`:

1. Add detection heuristics to `src/detect.rs`
2. Add tool name mappings to `src/tools.rs`
3. Add event mappings to `src/events.rs`
4. Add response serialization to `src/response.rs`
5. Rebuild WASM — all language SDKs pick up the changes automatically
