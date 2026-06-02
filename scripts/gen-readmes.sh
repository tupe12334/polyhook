#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

TOOLS_TABLE="| Tool | Status |
|---|---|
| [Claude Code](https://claude.ai/code) | ✅ Supported |
| [Cursor](https://cursor.com) | ✅ Supported |
| [Windsurf](https://windsurf.ai) | ✅ Supported |
| [Cline](https://github.com/cline/cline) | ✅ Supported |
| [Amp](https://ampcode.com) | ✅ Supported |
| [Continue](https://continue.dev) | 🚧 In progress |
| [Aider](https://aider.chat) | 🚧 In progress |
| [Copilot](https://github.com/features/copilot) | 📋 Planned |"

write_readme() {
  local path="$1"
  local content="$2"
  echo "$content" > "$path"
  echo "  wrote $path"
}

# ── TypeScript ────────────────────────────────────────────────────────────────
write_readme "$ROOT/packages/sdk-ts/README.md" "# polyhook — TypeScript / JavaScript SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

Ships as dual ESM + CJS — works with \`import\` (ESM) and \`require\` (CJS) out of the box.

## Install

\`\`\`bash
npm install @polyhook/sdk
# or
pnpm add @polyhook/sdk
# or
yarn add @polyhook/sdk
\`\`\`

## Quick Start

\`\`\`typescript
// ESM (\"type\": \"module\" or .mts)
import { read, respond, block, approve } from \"@polyhook/sdk\";

const event = await read();

if (
  event.tool === \"bash\" &&
  /rm\\s+-rf\\s+\\//.test((event.input?.command as string) ?? \"\")
) {
  await respond(block(\"Refusing to delete from root\"));
} else {
  await respond(approve());
}
\`\`\`

\`\`\`javascript
// CJS (require)
const { read, respond, block, approve } = require(\"@polyhook/sdk\");

async function main() {
  const event = await read();
  await respond(approve());
}
main();
\`\`\`

More examples: [examples/](examples/)

## API

| Function | Description |
|---|---|
| \`read()\` | Read and parse the hook event from stdin |
| \`respond(r)\` | Serialize and write a response to stdout |
| \`approve()\` | Build an approve response |
| \`block(message)\` | Build a block response with a message |
| \`modify(input)\` | Build a modify response with replacement fields |

## Supported Tools

$TOOLS_TABLE

## Documentation

Full docs and API reference: <https://github.com/tupe12334/polyhook>

## License

MIT"

# ── Rust ──────────────────────────────────────────────────────────────────────
write_readme "$ROOT/packages/sdk-rust/README.md" "# polyhook — Rust SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

The Rust SDK links \`polyhook-core\` natively — no WASM overhead.

## Install

\`\`\`bash
cargo add polyhook
\`\`\`

## Quick Start

\`\`\`rust
use polyhook::{read, respond, HookResponse};

fn main() {
    let event = match read() {
        Ok(e) => e,
        Err(e) => {
            eprintln!(\"polyhook: failed to read event: {e}\");
            std::process::exit(1);
        }
    };

    let response = match event.tool.as_deref() {
        Some(\"bash\") => {
            let cmd = event.input
                .as_ref()
                .and_then(|i| i.get(\"command\"))
                .and_then(|v| v.as_str())
                .unwrap_or(\"\");

            if cmd.contains(\"rm -rf /\") {
                HookResponse::block(\"Refusing to delete from root\")
            } else {
                HookResponse::approve()
            }
        }
        _ => HookResponse::approve(),
    };

    if let Err(e) = respond(&response) {
        eprintln!(\"polyhook: failed to write response: {e}\");
        std::process::exit(1);
    }
}
\`\`\`

More examples: [examples/](examples/)

## Supported Tools

$TOOLS_TABLE

## Documentation

Full docs and API reference: <https://github.com/tupe12334/polyhook>

## License

MIT"

# ── Go ────────────────────────────────────────────────────────────────────────
write_readme "$ROOT/packages/sdk-go/README.md" "# polyhook — Go SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

## Install

\`\`\`bash
go get github.com/tupe12334/polyhook/packages/sdk-go
\`\`\`

## Quick Start

\`\`\`go
package main

import (
	\"fmt\"
	\"os\"
	\"strings\"

	polyhook \"github.com/tupe12334/polyhook/packages/sdk-go\"
)

func main() {
	event, err := polyhook.Read()
	if err != nil {
		fmt.Fprintf(os.Stderr, \"polyhook: %v\n\", err)
		os.Exit(1)
	}

	if event.Tool != nil && *event.Tool == \"bash\" {
		if cmd, ok := event.Input[\"command\"].(string); ok && strings.Contains(cmd, \"rm -rf /\") {
			polyhook.Respond(polyhook.Block(\"Refusing to delete from root\"))
			return
		}
	}

	polyhook.Respond(polyhook.Approve())
}
\`\`\`

More examples: [examples/](examples/)

## Supported Tools

$TOOLS_TABLE

## Documentation

Full docs and API reference: <https://github.com/tupe12334/polyhook>

## License

MIT"

# ── .NET ──────────────────────────────────────────────────────────────────────
write_readme "$ROOT/packages/sdk-dotnet/README.md" "# polyhook — C# / .NET SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

## Install

\`\`\`bash
dotnet add package Polyhook.Sdk
\`\`\`

## Quick Start

\`\`\`csharp
using Polyhook.Sdk;
using System.Text.RegularExpressions;

var evt = await Polyhook.ReadAsync();

if (evt.Tool == \"bash\" &&
    evt.Input?.TryGetValue(\"command\", out var cmdEl) == true &&
    Regex.IsMatch(cmdEl.ToString()!, @\"rm\s+-rf\s+/\"))
{
    await Polyhook.RespondAsync(Polyhook.Block(\"Refusing to delete from root\"));
}
else
{
    await Polyhook.RespondAsync(Polyhook.Approve());
}
\`\`\`

More examples: [examples/](examples/)

## Supported Tools

$TOOLS_TABLE

## Documentation

Full docs and API reference: <https://github.com/tupe12334/polyhook>

## License

MIT"

# ── Python ────────────────────────────────────────────────────────────────────
write_readme "$ROOT/packages/sdk-python/README.md" "# polyhook — Python SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

## Install

\`\`\`bash
pip install polyhook
\`\`\`

## Quick Start

\`\`\`python
import sys
import re
import polyhook

event = polyhook.read()

if (
    event.tool == \"bash\"
    and re.search(r\"rm\\s+-rf\\s+/\", event.input.get(\"command\", \"\") if event.input else \"\")
):
    polyhook.respond(polyhook.block(\"Refusing to delete from root\"))
else:
    polyhook.respond(polyhook.approve())
\`\`\`

More examples: [examples/](examples/)

## Supported Tools

$TOOLS_TABLE

## Documentation

Full docs and API reference: <https://github.com/tupe12334/polyhook>

## License

MIT"

# ── Core ──────────────────────────────────────────────────────────────────────
write_readme "$ROOT/core/README.md" "# polyhook-core

Core types and detection logic for [polyhook](https://github.com/tupe12334/polyhook).

This crate is an internal dependency of the \`polyhook\` Rust SDK. **Use \`polyhook\` directly** — not this crate.

\`\`\`bash
cargo add polyhook
\`\`\`

## License

MIT"

echo "All READMEs generated."
