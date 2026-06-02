# polyhook — TypeScript / JavaScript SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

Ships as dual ESM + CJS — works with `import` (ESM) and `require` (CJS) out of the box.

## Install

```bash
npm install @polyhook/sdk
# or
pnpm add @polyhook/sdk
# or
yarn add @polyhook/sdk
```

## Quick Start

```typescript
// ESM ("type": "module" or .mts)
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

```javascript
// CJS (require)
const { read, respond, block, approve } = require("@polyhook/sdk");

async function main() {
  const event = await read();
  await respond(approve());
}
main();
```

More examples: [examples/](examples/)

## API

| Function | Description |
|---|---|
| `read()` | Read and parse the hook event from stdin |
| `respond(r)` | Serialize and write a response to stdout |
| `approve()` | Build an approve response |
| `block(message)` | Build a block response with a message |
| `modify(input)` | Build a modify response with replacement fields |

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
