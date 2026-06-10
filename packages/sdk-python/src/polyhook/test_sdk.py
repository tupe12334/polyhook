"""
Unit tests for the polyhook Python SDK.

The WASM module is mocked out so tests run without a real polyhook.wasm.
"""

from __future__ import annotations

import builtins
import io
import json
import struct
import sys
import types
from dataclasses import dataclass
from typing import Any
from unittest.mock import MagicMock, patch

import pytest


# ---------------------------------------------------------------------------
# Helpers for building mock WASM memory payloads
# ---------------------------------------------------------------------------


def _length_prefix(payload: bytes) -> bytes:
    """Produce a 4-byte LE length prefix followed by *payload*."""
    return struct.pack("<I", len(payload)) + payload


def _json_payload(obj: Any) -> bytes:
    return json.dumps(obj, separators=(",", ":")).encode()


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(autouse=True)
def reset_wasm_state():
    """Reset module-level WASM singletons before every test."""
    import polyhook.sdk as sdk

    sdk._store = None
    sdk._instance = None
    sdk._memory = None
    sdk._last_caller = "unknown"
    yield
    sdk._store = None
    sdk._instance = None
    sdk._memory = None
    sdk._last_caller = "unknown"


def _make_wasm_mock(
    parse_response: Any, serialize_response: bytes | None = None
) -> tuple[MagicMock, MagicMock]:
    """
    Build a (store_mock, instance_mock) pair whose exports behave like a real
    WASM instance for a single round-trip.

    *parse_response* — Python object to return from ``parse`` (will be JSON-
                       serialised and length-prefixed into a fake memory region).
    *serialize_response* — raw bytes the fake ``serialize`` call returns
                           (defaults to b'{"action":"approve"}').
    """
    if serialize_response is None:
        serialize_response = b'{"action":"approve"}'

    parse_blob = _length_prefix(_json_payload(parse_response))
    serialize_blob = _length_prefix(serialize_response)

    # Shared fake memory: we place parse result at offset 0x1000
    # and serialize result at offset 0x2000 so they don't overlap.
    PARSE_PTR = 0x1000
    SERIALIZE_PTR = 0x2000

    fake_mem: dict[tuple[int, int], bytes] = {
        (PARSE_PTR, PARSE_PTR + len(parse_blob)): parse_blob,
        (SERIALIZE_PTR, SERIALIZE_PTR + len(serialize_blob)): serialize_blob,
    }

    class FakeMemory:
        def write(self, store, data: bytes, offset: int) -> None:
            pass  # accept input writes

        def read(self, store, start: int, end: int) -> bytes:
            key = (start, end)
            if key in fake_mem:
                return fake_mem[key]
            # Allow reading individual bytes for length prefix
            for (s, e), blob in fake_mem.items():
                if s <= start and end <= e:
                    off = start - s
                    return blob[off : off + (end - start)]
            return b"\x00" * (end - start)

    alloc_counter = {"next": 0x100}

    def fake_alloc(store, n):
        ptr = alloc_counter["next"]
        alloc_counter["next"] += n
        return ptr

    def fake_dealloc(store, ptr, n):
        pass

    call_count = {"parse": 0, "serialize": 0}

    def fake_parse(store, ptr, n):
        call_count["parse"] += 1
        return PARSE_PTR

    def fake_serialize(store, ptr, n):
        call_count["serialize"] += 1
        return SERIALIZE_PTR

    exports_map = {
        "memory": FakeMemory(),
        "alloc": fake_alloc,
        "dealloc": fake_dealloc,
        "parse": fake_parse,
        "serialize": fake_serialize,
    }

    instance_mock = MagicMock()
    instance_mock.exports.return_value = exports_map

    store_mock = MagicMock()

    return store_mock, instance_mock, call_count


def _patch_wasm(parse_response: Any, serialize_response: bytes | None = None):
    """Context manager that replaces WASM internals with mock objects."""
    import polyhook.sdk as sdk

    store_mock, instance_mock, call_count = _make_wasm_mock(
        parse_response, serialize_response
    )

    sdk._store = store_mock
    sdk._instance = instance_mock
    sdk._memory = instance_mock.exports(store_mock)["memory"]
    return call_count


# ---------------------------------------------------------------------------
# HookEvent / convenience constructor tests (no WASM needed)
# ---------------------------------------------------------------------------


class TestConvenienceConstructors:
    def test_approve(self):
        from polyhook import approve

        r = approve()
        assert r == {"action": "approve"}

    def test_block(self):
        from polyhook import block

        r = block("dangerous command")
        assert r == {"action": "block", "message": "dangerous command"}

    def test_modify(self):
        from polyhook import modify

        new_input = {"command": "ls -la /tmp"}
        r = modify(new_input)
        assert r == {"action": "modify", "input": new_input}

    def test_block_empty_message(self):
        from polyhook import block

        r = block("")
        assert r["action"] == "block"
        assert r["message"] == ""

    def test_modify_nested(self):
        from polyhook import modify

        r = modify({"key": {"nested": [1, 2, 3]}})
        assert r["input"]["key"]["nested"] == [1, 2, 3]


# ---------------------------------------------------------------------------
# HookEvent dataclass tests
# ---------------------------------------------------------------------------


class TestHookEvent:
    def test_fields_present(self):
        from polyhook.sdk import HookEvent

        ev = HookEvent(
            event="tool:before",
            tool="bash",
            input={"command": "ls"},
            output=None,
            session_id="sess_abc",
            agent_id="agent_xyz",
            caller="claude-code",
        )
        assert ev.event == "tool:before"
        assert ev.tool == "bash"
        assert ev.input == {"command": "ls"}
        assert ev.output is None
        assert ev.session_id == "sess_abc"
        assert ev.agent_id == "agent_xyz"
        assert ev.caller == "claude-code"

    def test_optional_fields_none(self):
        from polyhook.sdk import HookEvent

        ev = HookEvent(
            event="session:start",
            tool=None,
            input=None,
            output=None,
            session_id="sess_001",
            agent_id=None,
            caller="cursor",
        )
        assert ev.tool is None
        assert ev.input is None
        assert ev.agent_id is None


# ---------------------------------------------------------------------------
# read() tests
# ---------------------------------------------------------------------------


class TestRead:
    def _event_dict(self, **overrides):
        base = {
            "event": "tool:before",
            "tool": "bash",
            "input": {"command": "echo hi"},
            "output": None,
            "sessionId": "sess_123",
            "agentId": "agent_abc",
            "caller": "claude-code",
        }
        base.update(overrides)
        return base

    def test_basic_parse(self):
        from polyhook import read

        _patch_wasm(self._event_dict())
        fake_stdin = b'{"hook_event_type": "PreToolUse"}'
        with patch("sys.stdin", io.TextIOWrapper(io.BytesIO(fake_stdin))):
            event = read()
        assert event.event == "tool:before"
        assert event.tool == "bash"
        assert event.input == {"command": "echo hi"}
        assert event.session_id == "sess_123"
        assert event.agent_id == "agent_abc"
        assert event.caller == "claude-code"

    def test_optional_fields_absent(self):
        from polyhook import read

        d = self._event_dict(tool=None, input=None, agentId=None)
        del d["tool"]
        del d["input"]
        del d["agentId"]
        _patch_wasm(d)
        with patch("sys.stdin", io.TextIOWrapper(io.BytesIO(b"{}"))):
            event = read()
        assert event.tool is None
        assert event.input is None
        assert event.agent_id is None

    def test_parse_error_raises_value_error(self):
        from polyhook import read

        _patch_wasm({"error": "unknown caller", "raw": "{}"})
        with patch("sys.stdin", io.TextIOWrapper(io.BytesIO(b"{}"))):
            with pytest.raises(
                ValueError, match="polyhook.wasm parse error: unknown caller"
            ):
                read()

    def test_caller_stored(self):
        from polyhook import read
        import polyhook.sdk as sdk

        _patch_wasm(self._event_dict(caller="windsurf"))
        with patch("sys.stdin", io.TextIOWrapper(io.BytesIO(b"{}"))):
            read()
        assert sdk._last_caller == "windsurf"

    def test_tool_after_event(self):
        from polyhook import read

        d = self._event_dict(
            event="tool:after",
            tool="write_file",
            input=None,
            output={"success": True},
        )
        _patch_wasm(d)
        with patch("sys.stdin", io.TextIOWrapper(io.BytesIO(b"{}"))):
            event = read()
        assert event.event == "tool:after"
        assert event.output == {"success": True}

    def test_session_start_event(self):
        from polyhook import read

        d = {
            "event": "session:start",
            "tool": None,
            "input": None,
            "output": None,
            "sessionId": "sess_new",
            "caller": "amp",
        }
        _patch_wasm(d)
        with patch("sys.stdin", io.TextIOWrapper(io.BytesIO(b"{}"))):
            event = read()
        assert event.event == "session:start"
        assert event.tool is None


# ---------------------------------------------------------------------------
# respond() tests
# ---------------------------------------------------------------------------


class TestRespond:
    def _run_respond(
        self, response_obj: Any, wasm_output: bytes = b'{"ok":true}'
    ) -> bytes:
        """Run respond() and return what was written to stdout."""
        from polyhook import respond

        _patch_wasm(
            parse_response={"event": "tool:before", "sessionId": "s", "caller": "c"},
            serialize_response=wasm_output,
        )
        # respond() writes to sys.stdout.buffer, so patch that directly.
        buf = io.BytesIO()
        mock_stdout = MagicMock()
        mock_stdout.buffer = buf
        with patch("sys.stdout", mock_stdout):
            respond(response_obj)
        return buf.getvalue()

    def test_approve_written(self):
        from polyhook import approve

        output = self._run_respond(approve(), b'{"action":"approve"}')
        assert output == b'{"action":"approve"}'

    def test_block_written(self):
        from polyhook import block

        output = self._run_respond(
            block("stop it"), b'{"decision":"block","reason":"stop it"}'
        )
        assert output == b'{"decision":"block","reason":"stop it"}'

    def test_modify_written(self):
        from polyhook import modify

        output = self._run_respond(
            modify({"command": "ls /tmp"}),
            b'{"decision":"modify","input":{"command":"ls /tmp"}}',
        )
        assert b"modify" in output

    def test_serialize_called_once(self):
        """Verify that the serialize WASM export is called exactly once."""
        import polyhook.sdk as sdk
        from polyhook import approve, respond

        store_mock, instance_mock, call_count = _make_wasm_mock(
            {"event": "tool:before", "sessionId": "s", "caller": "c"},
            b'{"action":"approve"}',
        )
        sdk._store = store_mock
        sdk._instance = instance_mock
        sdk._memory = instance_mock.exports(store_mock)["memory"]

        buf = io.BytesIO()
        mock_stdout = MagicMock()
        mock_stdout.buffer = buf
        with patch("sys.stdout", mock_stdout):
            respond(approve())

        assert call_count["serialize"] == 1

    def test_respond_json_encoded(self):
        """Ensure respond() JSON-encodes the dict before passing to WASM."""
        import polyhook.sdk as sdk

        captured_input: list[bytes] = []

        original_write = sdk._write_to_wasm

        def spy_write(data: bytes):
            captured_input.append(data)
            return original_write(data)

        _patch_wasm(
            {"event": "tool:before", "sessionId": "s", "caller": "c"},
            b'{"action":"approve"}',
        )

        buf = io.BytesIO()
        mock_stdout = MagicMock()
        mock_stdout.buffer = buf
        with patch.object(sdk, "_write_to_wasm", side_effect=spy_write):
            with patch("sys.stdout", mock_stdout):
                from polyhook import approve, respond

                respond(approve())

        # At least one write should be valid JSON
        assert any(json.loads(d) is not None for d in captured_input)


# ---------------------------------------------------------------------------
# Missing WASM file test
# ---------------------------------------------------------------------------


class TestMissingWasm:
    def test_missing_wasm_raises_import_error(self):
        import polyhook.sdk as sdk

        # Ensure state is clean
        sdk._store = None
        sdk._instance = None
        sdk._memory = None

        # Build a fake path object whose .exists() returns False.
        fake_wasm_path = MagicMock()
        fake_wasm_path.exists.return_value = False
        fake_wasm_path.__str__ = MagicMock(return_value="/fake/path/polyhook.wasm")

        # The code does: Path(__file__).parent / "polyhook.wasm"
        # We need to intercept that division so it returns our fake path.
        fake_parent = MagicMock()
        fake_parent.__truediv__ = MagicMock(return_value=fake_wasm_path)

        fake_path_instance = MagicMock()
        fake_path_instance.parent = fake_parent

        def fake_path_constructor(*args):
            return fake_path_instance

        with patch("polyhook.sdk.Path", side_effect=fake_path_constructor):
            with pytest.raises(ImportError, match="polyhook.wasm not found"):
                sdk._init_wasm()


# ---------------------------------------------------------------------------
# _init_wasm() specific path tests
# ---------------------------------------------------------------------------


class TestInitWasm:
    def test_early_return_when_instance_already_set(self):
        """_init_wasm() must return immediately when _instance is already set."""
        import polyhook.sdk as sdk

        sentinel = object()
        sdk._instance = sentinel  # pre-set so _init_wasm should bail out

        # If _init_wasm does NOT early-return it will try to import wasmtime
        # or check the wasm file; both would fail/have side-effects.
        # Patching Path to guarantee a failure if the early-return is skipped.
        with patch("polyhook.sdk.Path") as mock_path:
            sdk._init_wasm()
            # Path should never have been called — we returned before reaching it.
            mock_path.assert_not_called()

        # _instance must still be the sentinel we set.
        assert sdk._instance is sentinel

    def test_happy_path_with_mocked_wasmtime(self):
        """_init_wasm() happy path: file exists + wasmtime importable."""
        import polyhook.sdk as sdk

        # Start from a clean state (autouse fixture already does this, but be explicit).
        sdk._store = None
        sdk._instance = None
        sdk._memory = None

        # --- Fake wasmtime objects ----------------------------------------
        fake_memory = MagicMock(name="memory")

        fake_exports = {"memory": fake_memory}

        fake_instance = MagicMock(name="Instance")
        fake_instance.exports.return_value = fake_exports

        fake_store = MagicMock(name="Store")

        fake_module = MagicMock(name="Module")

        FakeStore = MagicMock(return_value=fake_store)
        FakeModule = MagicMock(name="Module_class")
        FakeModule.from_file.return_value = fake_module
        FakeInstance = MagicMock(return_value=fake_instance)

        # --- Fake wasmtime module -----------------------------------------
        fake_wasmtime_module = types.ModuleType("wasmtime")
        fake_wasmtime_module.Store = FakeStore
        fake_wasmtime_module.Module = FakeModule
        fake_wasmtime_module.Instance = FakeInstance

        # --- Fake path that reports the file exists -----------------------
        fake_wasm_path = MagicMock()
        fake_wasm_path.exists.return_value = True
        fake_wasm_path.__str__ = MagicMock(return_value="/fake/polyhook.wasm")

        fake_parent = MagicMock()
        fake_parent.__truediv__ = MagicMock(return_value=fake_wasm_path)

        fake_path_instance = MagicMock()
        fake_path_instance.parent = fake_parent

        def fake_path_constructor(*args):
            return fake_path_instance

        with patch("polyhook.sdk.Path", side_effect=fake_path_constructor):
            with patch.dict(sys.modules, {"wasmtime": fake_wasmtime_module}):
                sdk._init_wasm()

        # After a successful _init_wasm the globals must be populated.
        assert sdk._store is fake_store
        assert sdk._instance is fake_instance
        assert sdk._memory is fake_memory

        # Verify the wasmtime objects were constructed correctly.
        FakeStore.assert_called_once_with()
        FakeModule.from_file.assert_called_once_with(
            fake_store.engine, "/fake/polyhook.wasm"
        )
        FakeInstance.assert_called_once_with(fake_store, fake_module, [])

    def test_wasmtime_import_error(self):
        """_init_wasm() raises ImportError with a helpful message when wasmtime is absent."""
        import polyhook.sdk as sdk

        sdk._store = None
        sdk._instance = None
        sdk._memory = None

        # Fake path that claims the file exists.
        fake_wasm_path = MagicMock()
        fake_wasm_path.exists.return_value = True
        fake_wasm_path.__str__ = MagicMock(return_value="/fake/polyhook.wasm")

        fake_parent = MagicMock()
        fake_parent.__truediv__ = MagicMock(return_value=fake_wasm_path)

        fake_path_instance = MagicMock()
        fake_path_instance.parent = fake_parent

        def fake_path_constructor(*args):
            return fake_path_instance

        # Remove wasmtime from sys.modules so "import wasmtime" inside
        # _init_wasm raises ImportError naturally.
        cleaned_modules = {k: v for k, v in sys.modules.items() if k != "wasmtime"}

        original_import = builtins.__import__

        def blocking_import(name, *args, **kwargs):
            if name == "wasmtime":
                raise ImportError("No module named 'wasmtime'")
            return original_import(name, *args, **kwargs)

        with patch("polyhook.sdk.Path", side_effect=fake_path_constructor):
            with patch.dict(sys.modules, cleaned_modules, clear=True):
                with patch("builtins.__import__", side_effect=blocking_import):
                    with pytest.raises(ImportError, match="wasmtime is required"):
                        sdk._init_wasm()


# ---------------------------------------------------------------------------
# Round-trip integration test (parse → respond) with mock WASM
# ---------------------------------------------------------------------------


class TestRoundTrip:
    def test_full_round_trip(self):
        """
        Simulate a complete hook invocation: read an event then respond.
        Asserts that stdout receives the serialized bytes from WASM.
        """
        import polyhook.sdk as sdk
        from polyhook import approve, block, read, respond

        event_dict = {
            "event": "tool:before",
            "tool": "bash",
            "input": {"command": "rm -rf /"},
            "output": None,
            "sessionId": "sess_danger",
            "agentId": None,
            "caller": "claude-code",
        }
        wasm_block_output = b'{"decision":"block","reason":"dangerous command"}'

        store_mock, instance_mock, call_count = _make_wasm_mock(
            event_dict, wasm_block_output
        )
        sdk._store = store_mock
        sdk._instance = instance_mock
        sdk._memory = instance_mock.exports(store_mock)["memory"]

        stdin_data = b'{"hook_event_type": "PreToolUse", "tool_name": "Bash"}'
        stdout_buf = io.BytesIO()

        with patch("sys.stdin", io.TextIOWrapper(io.BytesIO(stdin_data))):
            event = read()

        assert event.tool == "bash"
        assert event.input is not None and "rm -rf /" in event.input.get("command", "")

        response = block("dangerous command")
        mock_stdout = MagicMock()
        mock_stdout.buffer = stdout_buf
        with patch("sys.stdout", mock_stdout):
            respond(response)

        result = stdout_buf.getvalue()
        assert result == wasm_block_output
        assert call_count["parse"] == 1
        assert call_count["serialize"] == 1
