# polyhook — C# / .NET SDK

**Write AI coding agent hooks once. Run them everywhere.**

polyhook detects which AI coding tool invoked your hook binary, deserializes the event into a normalized struct, and serializes your response back in the format that tool expects. Your hook runs unchanged whether Claude Code, Cursor, Windsurf, Cline, or Amp invoked it.

## Install

```bash
dotnet add package Polyhook.Sdk
```

## Quick Start

```csharp
using Polyhook.Sdk;
using System.Text.RegularExpressions;

var evt = await Polyhook.ReadAsync();

if (evt.Tool == "bash" &&
    evt.Input?.TryGetValue("command", out var cmdEl) == true &&
    Regex.IsMatch(cmdEl.ToString()!, @"rm\s+-rf\s+/"))
{
    await Polyhook.RespondAsync(Polyhook.Block("Refusing to delete from root"));
}
else
{
    await Polyhook.RespondAsync(Polyhook.Approve());
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
