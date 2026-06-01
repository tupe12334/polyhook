"""
polyhook Python SDK — wraps polyhook.wasm via wasmtime-py.

Typical usage::

    import polyhook

    event = polyhook.read()          # parse stdin → HookEvent
    if event.tool == "bash":
        polyhook.respond(polyhook.block("not allowed"))
    else:
        polyhook.respond(polyhook.approve())
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

from .generated_models import HookEvent, HookResponse, CallerKind, HookEventName

# Re-export for backwards compatibility
__all__ = ["HookEvent", "HookResponse", "CallerKind", "HookEventName"]

# ---------------------------------------------------------------------------
# Convenience constructors
# ---------------------------------------------------------------------------

def approve() -> HookResponse:
    """Return an *approve* response (allow the action unchanged)."""
    return {"action": "approve"}


def block(message: str) -> HookResponse:
    """Return a *block* response (reject the action with *message*)."""
    return {"action": "block", "message": message}


def modify(input: dict[str, Any]) -> HookResponse:
    """Return a *modify* response (replace tool input with *input*)."""
    return {"action": "modify", "input": input}


# ---------------------------------------------------------------------------
# Internal WASM state (lazy-initialised on first read() call)
# ---------------------------------------------------------------------------

_store: Any = None
_instance: Any = None
_memory: Any = None
_last_caller: str = "unknown"


def _init_wasm() -> None:
    """Lazily initialise the wasmtime Store / Instance from polyhook.wasm."""
    global _store, _instance, _memory

    if _instance is not None:
        return

    wasm_path = Path(__file__).parent / "polyhook.wasm"
    if not wasm_path.exists():
        raise ImportError(
            f"polyhook.wasm not found at {wasm_path}.\n"
            "Build the WASM module first (e.g. `cargo build --target wasm32-unknown-unknown --release`) "
            "and copy polyhook.wasm into the package directory alongside sdk.py."
        )

    try:
        from wasmtime import Instance, Module, Store
    except ImportError as exc:
        raise ImportError(
            "wasmtime is required to use the polyhook SDK.  "
            "Install it with: pip install wasmtime"
        ) from exc

    _store = Store()
    module = Module.from_file(_store.engine, str(wasm_path))
    _instance = Instance(_store, module, [])
    _memory = _instance.exports(_store)["memory"]


# ---------------------------------------------------------------------------
# Low-level WASM helpers
# ---------------------------------------------------------------------------

def _alloc(n: int) -> int:
    return _instance.exports(_store)["alloc"](_store, n)


def _dealloc(ptr: int, n: int) -> None:
    _instance.exports(_store)["dealloc"](_store, ptr, n)


def _parse(ptr: int, n: int) -> int:
    return _instance.exports(_store)["parse"](_store, ptr, n)


def _serialize(ptr: int, n: int) -> int:
    return _instance.exports(_store)["serialize"](_store, ptr, n)


def _write_to_wasm(data: bytes) -> tuple[int, int]:
    """Copy *data* into WASM linear memory and return ``(ptr, len)``."""
    ptr = _alloc(len(data))
    _memory.write(_store, data, ptr)
    return ptr, len(data)


def _read_from_wasm(ptr: int) -> bytes:
    """Read a length-prefixed payload from WASM linear memory.

    Memory layout::

        offset 0        4          4+len
          ┌─────────────┬──────────────────┐
          │ len (i32 LE)│ payload (UTF-8)   │
          └─────────────┴──────────────────┘
    """
    length_bytes = bytes(_memory.read(_store, ptr, ptr + 4))
    length = int.from_bytes(length_bytes, "little")
    payload = bytes(_memory.read(_store, ptr + 4, ptr + 4 + length))
    return payload


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def read() -> HookEvent:
    """Read the hook event from *stdin* and return a :class:`HookEvent`.

    This must be called exactly once per invocation, before :func:`respond`.
    The WASM module is initialised on the first call.
    """
    global _last_caller

    _init_wasm()

    stdin_bytes: bytes = sys.stdin.buffer.read()

    # Write stdin into WASM memory, call parse, then free the input buffer.
    in_ptr, in_len = _write_to_wasm(stdin_bytes)
    result_ptr = _parse(in_ptr, in_len)
    _dealloc(in_ptr, in_len)

    # Read the length-prefixed result, then free the result buffer.
    payload = _read_from_wasm(result_ptr)
    payload_len = int.from_bytes(bytes(_memory.read(_store, result_ptr, result_ptr + 4)), "little")
    _dealloc(result_ptr, 4 + payload_len)

    data: dict[str, Any] = json.loads(payload)

    # Surface WASM-level parse errors as Python exceptions.
    if "error" in data:
        raise ValueError(
            f"polyhook.wasm parse error: {data['error']}"
            + (f" (raw: {data.get('raw', '')})" if "raw" in data else "")
        )

    # Map camelCase JSON keys to snake_case dataclass fields.
    caller = data.get("caller", "unknown")
    _last_caller = caller

    return HookEvent(
        event=data["event"],
        tool=data.get("tool"),
        input=data.get("input"),
        output=data.get("output"),
        session_id=data["sessionId"],
        agent_id=data.get("agentId"),
        caller=caller,
    )


def respond(r: HookResponse) -> None:
    """Serialize *r* via the WASM module and write the result to *stdout*.

    Must be called after :func:`read` (the WASM module uses caller-detection
    state set during the preceding ``parse`` call to select the correct output
    format).
    """
    _init_wasm()

    response_json: bytes = json.dumps(r, separators=(",", ":")).encode("utf-8")

    # Write the response into WASM memory, call serialize, free the input.
    in_ptr, in_len = _write_to_wasm(response_json)
    out_ptr = _serialize(in_ptr, in_len)
    _dealloc(in_ptr, in_len)

    # Read the length-prefixed output, then free the result buffer.
    out_bytes = _read_from_wasm(out_ptr)
    out_len = int.from_bytes(bytes(_memory.read(_store, out_ptr, out_ptr + 4)), "little")
    _dealloc(out_ptr, 4 + out_len)

    sys.stdout.buffer.write(out_bytes)
    sys.stdout.buffer.flush()
