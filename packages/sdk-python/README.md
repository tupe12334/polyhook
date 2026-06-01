# polyhook — Python SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

## Install

```bash
pip install polyhook
```

## Quick Start

```python
import sys
import re
import polyhook

event = polyhook.read()

if (
    event.tool == "bash"
    and re.search(r"rm\s+-rf\s+/", event.input.get("command", "") if event.input else "")
):
    polyhook.respond(polyhook.block("Refusing to delete from root"))
else:
    polyhook.respond(polyhook.approve())
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
