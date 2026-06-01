# polyhook

**Write AI coding agent hooks once. Run them everywhere.**

polyhook is a multi-language SDK for AI coding agent hooks. Instead of parsing each tool's bespoke stdin/stdout format, you call polyhook — it detects which tool invoked your binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects.

Your hook binary runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

---

## Architecture

Rust core compiled to WASM. Every language SDK is a thin shim over the same binary — no logic re-implemented per language. See [ARCHITECTURE.md](ARCHITECTURE.md).

---

## The Problem

Every AI coding tool triggers hooks differently. The same logical event — "bash command about to run" — arrives in a different shape and expects a different response format:

```
Claude Code  →  stdin: { "tool_name": "Bash", "tool_input": { "command": "..." } }
                stdout: { "decision": "block", "reason": "..." }

Cursor       →  stdin: { "type": "BeforeToolCall", "toolCall": { "name": "run_terminal_cmd", "args": {...} } }
                stdout: { "action": "deny", "message": "..." }

Windsurf     →  stdin: { "event": "pre_tool", "tool": "run_command", "parameters": {...} }
                stdout: { "allow": false, "reason": "..." }
```

Without polyhook you write a parser and serializer for each tool. With polyhook you call `read()` and `respond()`.

---

## How It Works

```
AI tool  ──stdin──▶  your binary
                         │
                    polyhook.read()        ← WASM: detects tool, parses format
                         │
                    your hook logic        ← tool-agnostic
                         │
                    polyhook.respond()     ← WASM: serializes to tool's format
                         │
your binary  ──stdout──▶  AI tool
```

---

## Normalized Types

Types are auto-generated in every SDK from `schema.json` — not hand-written. See [ARCHITECTURE.md](ARCHITECTURE.md) for the full generation pipeline.

### HookEvent

```typescript
interface HookEvent {
  event:     "tool:before" | "tool:after" | "session:start" | "session:stop" | "agent:stop" | "notification";
  tool?:     string;                        // normalized tool name, e.g. "bash", "write_file"
  input?:    Record<string, unknown>;       // tool input arguments
  output?:   Record<string, unknown>;       // tool output (tool:after only)
  sessionId: string;
  agentId?:  string;
  caller:    "claude-code" | "cursor" | "windsurf" | "cline" | "amp" | "unknown";
}
```

Normalized tool names: [docs/tool-names.md](docs/tool-names.md)

### HookResponse

```typescript
type HookResponse =
  | { action: "approve" }
  | { action: "block";  message: string }
  | { action: "modify"; input: Record<string, unknown> }
```

---

## Language SDKs

All SDKs expose the same two functions. The WASM module does all the work.

### TypeScript / JavaScript

```bash
npm install @polyhook/sdk
```

```typescript
import { read, respond } from "@polyhook/sdk";

const event = await read();

if (event.tool === "bash" && /rm\s+-rf\s+\//.test(event.input?.command as string)) {
  await respond({ action: "block", message: "Refusing to delete from root" });
} else {
  await respond({ action: "approve" });
}
```

### Rust

The only SDK that links polyhook-core natively — no WASM overhead.

```bash
cargo add polyhook
```

```rust
use polyhook::{read, respond, HookResponse};

fn main() {
    let event = read().unwrap();

    let response = match event.tool.as_deref() {
        Some("bash") => {
            let cmd = event.input["command"].as_str().unwrap_or("");
            if cmd.contains("rm -rf /") {
                HookResponse::block("Refusing to delete from root")
            } else {
                HookResponse::approve()
            }
        }
        _ => HookResponse::approve(),
    };

    respond(&response).unwrap();
}
```

### Go

```bash
go get github.com/polyhook/polyhook-go
```

```go
package main

import (
    "strings"
    "github.com/polyhook/polyhook-go"
)

func main() {
    event, _ := polyhook.Read()  // calls polyhook.wasm via Wazero

    if event.Tool == "bash" {
        if cmd, ok := event.Input["command"].(string); ok && strings.Contains(cmd, "rm -rf /") {
            polyhook.Respond(polyhook.Block("Refusing to delete from root"))
            return
        }
    }

    polyhook.Respond(polyhook.Approve())
}
```

### C# / .NET

```bash
dotnet add package Polyhook.Sdk
```

```csharp
using Polyhook.Sdk;  // calls polyhook.wasm via Wasmtime

var evt = await Polyhook.ReadAsync();

if (evt.Tool == "bash" &&
    evt.Input.TryGetValue("command", out var cmd) &&
    cmd.ToString()!.Contains("rm -rf /"))
{
    await Polyhook.RespondAsync(HookResponse.Block("Refusing to delete from root"));
}
else
{
    await Polyhook.RespondAsync(HookResponse.Approve());
}
```

### Python

```bash
pip install polyhook
```

```python
import polyhook  # calls polyhook.wasm via wasmtime-py

event = polyhook.read()

if event.tool == "bash" and "rm -rf /" in (event.input.get("command") or ""):
    polyhook.respond(polyhook.block("Refusing to delete from root"))
else:
    polyhook.respond(polyhook.approve())
```

> Any language with a WASM runtime can bind polyhook. See [BINDINGS.md](BINDINGS.md) for the raw WASM API.

---


## Supported Tools

| Tool | Status |
|---|---|
| [Claude Code](https://claude.ai/code) | ✅ Supported |
| [Cursor](https://cursor.com) | ✅ Supported |
| [Windsurf](https://windsurf.ai) | ✅ Supported |
| [Cline](https://github.com/cline/cline) | ✅ Supported |
| [Amp](https://ampcode.com) | ✅ Supported |
| [Continue](https://continue.dev) | 🚧 In progress |
| [Aider](https://aider.chat) | 🚧 In progress |
| [Copilot](https://github.com/features/copilot) | 📋 Planned |

Source of truth: [tools.toml](tools.toml) — hook documentation links: [docs/tool-names.md](docs/tool-names.md)

---


## Design Goals

- **One implementation.** All detection and serialization logic lives in `polyhook-core`. Language SDKs cannot diverge.
- **Runtime only.** No code generation. polyhook runs inside your binary at hook invocation time.
- **Transparent fallback.** Unknown caller → `caller: "unknown"`, best-effort parse. Your logic keeps running.
- **Bring your own runtime.** Each SDK ships `polyhook.wasm` and a thin host binding. Swap the WASM runtime if your platform requires it.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). To add tool support, add detection heuristics and mappings to `crates/polyhook-core` — all language SDKs pick it up automatically on the next WASM build.

---

## License

MIT
