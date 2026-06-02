# polyhook — Go SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

## Install

```bash
go get github.com/tupe12334/polyhook/packages/sdk-go
```

## Quick Start

```go
package main

import (
	"fmt"
	"os"
	"strings"

	polyhook "github.com/tupe12334/polyhook/packages/sdk-go"
)

func main() {
	event, err := polyhook.Read()
	if err != nil {
		fmt.Fprintf(os.Stderr, "polyhook: %v\n", err)
		os.Exit(1)
	}

	if event.Tool != nil && *event.Tool == "bash" {
		if cmd, ok := event.Input["command"].(string); ok && strings.Contains(cmd, "rm -rf /") {
			polyhook.Respond(polyhook.Block("Refusing to delete from root"))
			return
		}
	}

	polyhook.Respond(polyhook.Approve())
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
