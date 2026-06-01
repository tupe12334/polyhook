# polyhook — TypeScript / JavaScript SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

## Install

```bash
npm install @polyhook/sdk
```

## Quick Start

```typescript
import { read, respond, block, approve } from "@polyhook/sdk";

const event = await read();

if (
  event.tool === "bash" &&
  /rm\s+-rf\s+\//.test((event.input?.command as string) ?? "")
) {
  await respond(block("Refusing to delete from root"));
} else {
  await respond(approve());
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
