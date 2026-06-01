# WASM Host Bindings

This document describes the raw API exposed by `polyhook.wasm`. Use this if you are writing a language binding that isn't covered by an existing SDK.

For end-user documentation see the [README](README.md).

---

## Exports

`polyhook.wasm` exports two functions:

### `parse(ptr: i32, len: i32) -> i32`

Parses raw stdin bytes into a normalized `HookEvent`.

- **`ptr`** — pointer to a UTF-8 JSON byte slice in WASM linear memory
- **`len`** — byte length of the slice
- **Returns** — pointer to a length-prefixed JSON byte slice containing the normalized `HookEvent`, or a JSON error object on failure

### `serialize(ptr: i32, len: i32) -> i32`

Serializes a normalized `HookResponse` into the format expected by the detected caller.

Must be called after `parse` — detection state from the previous `parse` call is used to select the correct output format.

- **`ptr`** — pointer to a UTF-8 JSON byte slice containing a `HookResponse`
- **`len`** — byte length of the slice
- **Returns** — pointer to a length-prefixed JSON byte slice ready to write to stdout

---

## Memory

`polyhook.wasm` exports a `memory` object (standard WASM linear memory).

All pointers passed to and returned from exports point into this memory. Returned pointers use a **length prefix**: the first 4 bytes (little-endian `i32`) are the byte length of the payload, followed immediately by the payload bytes.

```
offset 0                    4         4+len
  ┌──────────────────────────┬─────────────────────┐
  │  len (i32, little-endian)│  payload bytes (UTF-8 JSON)  │
  └──────────────────────────┴─────────────────────┘
```

The host is responsible for:
1. Allocating input bytes into WASM memory before calling an export
2. Reading the length-prefixed result out of WASM memory after the call
3. Freeing allocations via the exported `alloc` / `dealloc` functions

### `alloc(len: i32) -> i32`

Allocates `len` bytes in WASM linear memory. Returns the pointer. The host writes input bytes here before calling `parse` or `serialize`.

### `dealloc(ptr: i32, len: i32)`

Frees a previously allocated region. Call after reading a result pointer returned by `parse` or `serialize`.

---

## Example Host Sequence

```
1. stdin_bytes = read_all_stdin()

2. ptr = alloc(len(stdin_bytes))
   write_to_wasm_memory(ptr, stdin_bytes)

3. result_ptr = parse(ptr, len(stdin_bytes))
   dealloc(ptr, len(stdin_bytes))

4. result_len = read_i32_le(wasm_memory, result_ptr)
   event_json  = read_bytes(wasm_memory, result_ptr + 4, result_len)
   dealloc(result_ptr, 4 + result_len)

5. event = json_decode(event_json)   // -> HookEvent

6. response = your_hook_logic(event) // -> HookResponse

7. response_json = json_encode(response)
   ptr2 = alloc(len(response_json))
   write_to_wasm_memory(ptr2, response_json)

8. out_ptr = serialize(ptr2, len(response_json))
   dealloc(ptr2, len(response_json))

9. out_len   = read_i32_le(wasm_memory, out_ptr)
   out_bytes = read_bytes(wasm_memory, out_ptr + 4, out_len)
   dealloc(out_ptr, 4 + out_len)

10. write_all_stdout(out_bytes)
```

---

## Error Handling

Both `parse` and `serialize` return a valid length-prefixed payload on error. The payload is a JSON object with an `"error"` key:

```json
{ "error": "unknown caller", "raw": "<original stdin>" }
```

On parse error, `caller` falls back to `"unknown"` and polyhook attempts a best-effort parse. On serialize error, an empty `approve` response is emitted to avoid blocking the caller.

---

## HookEvent JSON Schema

```json
{
  "event":     "tool:before",
  "tool":      "bash",
  "input":     { "command": "ls -la" },
  "output":    null,
  "sessionId": "sess_abc123",
  "agentId":   "agent_xyz",
  "caller":    "claude-code"
}
```

## HookResponse JSON Schema

```json
{ "action": "approve" }
{ "action": "block",  "message": "reason string" }
{ "action": "modify", "input": { "command": "ls -la /tmp" } }
```
