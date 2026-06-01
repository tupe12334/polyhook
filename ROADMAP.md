# Roadmap

## Now — Foundation ✅

Core architecture is stable. WASM module handles detection, normalization, and serialization for five tools. All five language SDKs ship.

- **Core:** Rust → WASM, caller detection, event + tool normalization, response serialization
- **Tools:** Claude Code, Cursor, Windsurf, Cline, Amp
- **SDKs:** TypeScript, Rust, Go, C#, Python
- **Types:** auto-generated from `schema.json` across every SDK

---

## Near-term

### Tool coverage

| Tool | Work |
|---|---|
| **Continue** | In progress — detection heuristics + event mapping |
| **Aider** | In progress — detection heuristics + event mapping |
| **GitHub Copilot** | Planned — blocked on hooks API availability |

### SDK polish

- Ergonomic helpers: `event.is_bash()`, `event.command()`, etc.
- Typed `input` fields per tool (vs `Record<string, unknown>`)
- First-class `modify` response support in all SDKs

### Developer experience

- `polyhook check` — CLI that prints the detected caller and parsed event for a given stdin payload; aids hook debugging
- Published examples per SDK + tool combination

---

## Mid-term

### More tool support

- Goose (Block)
- Roo Code
- Zed AI

### Schema evolution

- Structured `output` in `tool:after` events (currently opaque)
- `error` event type for hook runtime failures
- Versioned schema with migration path

### Testing

- Cross-tool fixture suite — same logical event, all vendor formats; verifies normalization is consistent
- SDK conformance tests generated from the fixture suite

---

## Long-term

### Hook registry / hub

Shareable, installable hook packages. Publish a hook once; install it in any tool.

```
polyhook install no-root-delete
polyhook install require-tests-before-push
```

### Browser / edge runtime support

`polyhook.wasm` already runs in any WASM host. Explicit support for Deno, Bun, Cloudflare Workers, and browser `<script type="module">`.

### Observability

Optional structured log output — every hook invocation emits a JSON trace: caller, event type, tool, decision, latency. Plug into any log aggregator.

---

## Out of scope

- Re-implementing any detection or serialization logic outside of `core` — all SDK logic stays in WASM
- Supporting tools that don't expose a hook mechanism
- A managed cloud runtime — polyhook runs inside your binary, not ours
