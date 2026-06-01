# polyhook — Rust SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

The Rust SDK links `polyhook-core` natively — no WASM overhead.

## Install

```bash
cargo add polyhook
```

## Quick Start

```rust
use polyhook::{read, respond, HookResponse};

fn main() {
    let event = match read() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("polyhook: failed to read event: {e}");
            std::process::exit(1);
        }
    };

    let response = match event.tool.as_deref() {
        Some("bash") => {
            let cmd = event.input
                .as_ref()
                .and_then(|i| i.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if cmd.contains("rm -rf /") {
                HookResponse::block("Refusing to delete from root")
            } else {
                HookResponse::approve()
            }
        }
        _ => HookResponse::approve(),
    };

    if let Err(e) = respond(&response) {
        eprintln!("polyhook: failed to write response: {e}");
        std::process::exit(1);
    }
}
```

More examples: [examples/](examples/)

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

## Documentation

Full docs and API reference: <https://github.com/tupe12334/polyhook>

## License

MIT
