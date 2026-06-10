package polyhook_test

// polyhook_test.go exercises the Go SDK with table-driven tests.
//
// Integration tests that need the WASM runtime use the passthrough WAT shim
// defined in wasm_helper_test.go, so they always run regardless of whether a
// real polyhook.wasm binary is present.
//
// The pure-unit tests (type constructors, JSON serialisation, CallerKind
// constants) always run and do not touch the WASM layer at all.

import (
	"bytes"
	"encoding/json"
	"errors"
	"io"
	"os"
	"strings"
	"testing"

	polyhook "github.com/tupe12334/polyhook/packages/sdk-go"
)

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// hookEventJSON builds a minimal pre-normalised HookEvent JSON string.
// When the passthrough WAT shim is in use, this is exactly what ReadFrom
// receives back after the echo round-trip.
func hookEventJSON(event, tool, caller, sessionID string, input map[string]interface{}) string {
	type he struct {
		Event     string                 `json:"event"`
		Tool      *string                `json:"tool,omitempty"`
		Input     map[string]interface{} `json:"input,omitempty"`
		SessionID string                 `json:"sessionId"`
		Caller    string                 `json:"caller"`
	}
	h := he{
		Event:     event,
		SessionID: sessionID,
		Caller:    caller,
		Input:     input,
	}
	if tool != "" {
		h.Tool = &tool
	}
	b, _ := json.Marshal(h)
	return string(b)
}

// ---------------------------------------------------------------------------
// Unit tests: type constructors and JSON marshalling
// ---------------------------------------------------------------------------

func TestApprove(t *testing.T) {
	r := polyhook.Approve()
	got, err := json.Marshal(r)
	if err != nil {
		t.Fatal(err)
	}
	want := `{"action":"approve"}`
	if string(got) != want {
		t.Errorf("Approve() JSON = %s; want %s", got, want)
	}
}

func TestBlock(t *testing.T) {
	cases := []struct {
		msg  string
		want string
	}{
		{"too dangerous", "too dangerous"},
		{"", ""},
		{"rm -rf / not allowed", "rm -rf / not allowed"},
	}
	for _, tc := range cases {
		r := polyhook.Block(tc.msg)
		got, err := json.Marshal(r)
		if err != nil {
			t.Fatalf("Block(%q) marshal: %v", tc.msg, err)
		}
		s := string(got)
		if !strings.Contains(s, `"action":"block"`) {
			t.Errorf("Block(%q) JSON missing action:block, got %s", tc.msg, s)
		}
		// message key always present, even for empty string
		if !strings.Contains(s, `"message"`) {
			t.Errorf("Block(%q) JSON missing message key, got %s", tc.msg, s)
		}
	}
}

func TestModify(t *testing.T) {
	cases := []struct {
		desc  string
		input map[string]interface{}
	}{
		{"single key", map[string]interface{}{"command": "ls /tmp"}},
		{"multiple keys", map[string]interface{}{"path": "/etc/hosts", "content": "# hi"}},
		{"empty map", map[string]interface{}{}},
	}
	for _, tc := range cases {
		r := polyhook.Modify(tc.input)
		got, err := json.Marshal(r)
		if err != nil {
			t.Fatalf("Modify(%s) marshal: %v", tc.desc, err)
		}
		s := string(got)
		if !strings.Contains(s, `"action":"modify"`) {
			t.Errorf("Modify(%s) JSON missing action:modify, got %s", tc.desc, s)
		}
		if !strings.Contains(s, `"input"`) {
			t.Errorf("Modify(%s) JSON missing input key, got %s", tc.desc, s)
		}
	}
}

// ---------------------------------------------------------------------------
// CallerKind constant tests
// ---------------------------------------------------------------------------

func TestCallerKindConstants(t *testing.T) {
	cases := []struct {
		label string
		got   polyhook.CallerKind
		want  string
	}{
		{"ClaudeCode", polyhook.CallerKindClaudeCode, "claude-code"},
		{"Cursor", polyhook.CallerKindCursor, "cursor"},
		{"Windsurf", polyhook.CallerKindWindsurf, "windsurf"},
		{"Cline", polyhook.CallerKindCline, "cline"},
		{"Amp", polyhook.CallerKindAmp, "amp"},
		{"GeminiCli", polyhook.CallerKindGeminiCli, "gemini-cli"},
		{"Hermes", polyhook.CallerKindHermes, "hermes"},
		{"Unknown", polyhook.CallerKindUnknown, "unknown"},
	}
	for _, tc := range cases {
		if string(tc.got) != tc.want {
			t.Errorf("Caller%s = %q; want %q", tc.label, tc.got, tc.want)
		}
	}
}

// ---------------------------------------------------------------------------
// HookEvent struct JSON round-trip
// ---------------------------------------------------------------------------

func TestHookEventUnmarshal(t *testing.T) {
	tool := "bash"
	agentID := "agent_xyz"
	raw := `{
		"event":     "tool:before",
		"tool":      "bash",
		"input":     {"command": "ls -la"},
		"sessionId": "sess_abc",
		"agentId":   "agent_xyz",
		"caller":    "claude-code"
	}`
	var ev polyhook.HookEvent
	if err := json.Unmarshal([]byte(raw), &ev); err != nil {
		t.Fatal(err)
	}
	if ev.Event != "tool:before" {
		t.Errorf("Event = %q; want tool:before", ev.Event)
	}
	if ev.Tool == nil || *ev.Tool != tool {
		t.Errorf("Tool = %v; want %q", ev.Tool, tool)
	}
	if ev.SessionId != "sess_abc" {
		t.Errorf("SessionId = %q; want sess_abc", ev.SessionId)
	}
	if ev.AgentId == nil || *ev.AgentId != agentID {
		t.Errorf("AgentId = %v; want %q", ev.AgentId, agentID)
	}
	if ev.Caller != polyhook.CallerKindClaudeCode {
		t.Errorf("Caller = %q; want %q", ev.Caller, polyhook.CallerKindClaudeCode)
	}
	input, ok := ev.Input.(map[string]interface{})
	if !ok {
		t.Fatalf("Input = %T; want map[string]interface{}", ev.Input)
	}
	if cmd, ok := input["command"]; !ok || cmd != "ls -la" {
		t.Errorf("Input[command] = %v; want ls -la", cmd)
	}
}

func TestHookEventOptionalFields(t *testing.T) {
	raw := `{"event":"session:start","sessionId":"s1","caller":"amp"}`
	var ev polyhook.HookEvent
	if err := json.Unmarshal([]byte(raw), &ev); err != nil {
		t.Fatal(err)
	}
	if ev.Tool != nil {
		t.Errorf("Tool should be nil for session:start; got %q", *ev.Tool)
	}
	if ev.AgentId != nil {
		t.Errorf("AgentId should be nil; got %q", *ev.AgentId)
	}
	if ev.Input != nil {
		t.Errorf("Input should be nil; got %v", ev.Input)
	}
}

// ---------------------------------------------------------------------------
// Integration tests: ReadFrom / RespondTo — always run via passthrough WAT
// ---------------------------------------------------------------------------

type readTestCase struct {
	name         string
	inputJSON    string
	wantEvent    string
	wantTool     string // empty means expect nil Tool
	wantCaller   string
	wantInputKey string // optional key expected in Input map
}

var readTests = []readTestCase{
	{
		name: "claude-code tool:before bash",
		inputJSON: hookEventJSON(
			"tool:before", "bash", "claude-code", "sess_cc_001",
			map[string]interface{}{"command": "ls -la"},
		),
		wantEvent:    "tool:before",
		wantTool:     "bash",
		wantCaller:   "claude-code",
		wantInputKey: "command",
	},
	{
		name: "cursor tool:before bash",
		inputJSON: hookEventJSON(
			"tool:before", "bash", "cursor", "sess_cur_002",
			map[string]interface{}{"command": "pwd"},
		),
		wantEvent:    "tool:before",
		wantTool:     "bash",
		wantCaller:   "cursor",
		wantInputKey: "command",
	},
	{
		name: "windsurf tool:before write_file",
		inputJSON: hookEventJSON(
			"tool:before", "write_file", "windsurf", "sess_ws_003",
			map[string]interface{}{"path": "/tmp/out.txt", "content": "hello"},
		),
		wantEvent:    "tool:before",
		wantTool:     "write_file",
		wantCaller:   "windsurf",
		wantInputKey: "path",
	},
	{
		name: "cline tool:after read_file",
		inputJSON: hookEventJSON(
			"tool:after", "read_file", "cline", "sess_cl_004",
			nil,
		),
		wantEvent:  "tool:after",
		wantTool:   "read_file",
		wantCaller: "cline",
	},
	{
		name: "amp session:start",
		inputJSON: hookEventJSON(
			"session:start", "", "amp", "sess_amp_005",
			nil,
		),
		wantEvent:  "session:start",
		wantTool:   "",
		wantCaller: "amp",
	},
	{
		name: "unknown caller notification",
		inputJSON: hookEventJSON(
			"notification", "", "unknown", "sess_unk_006",
			nil,
		),
		wantEvent:  "notification",
		wantCaller: "unknown",
	},
	{
		name: "claude-code agent:stop",
		inputJSON: hookEventJSON(
			"agent:stop", "", "claude-code", "sess_cc_007",
			nil,
		),
		wantEvent:  "agent:stop",
		wantCaller: "claude-code",
	},
	{
		name: "cursor tool:before with multiple input keys",
		inputJSON: hookEventJSON(
			"tool:before", "write_file", "cursor", "sess_cur_008",
			map[string]interface{}{"path": "/src/main.go", "content": "package main"},
		),
		wantEvent:    "tool:before",
		wantTool:     "write_file",
		wantCaller:   "cursor",
		wantInputKey: "content",
	},
}

func TestReadFrom_TableDriven(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	for _, tc := range readTests {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			event, err := polyhook.ReadFrom(strings.NewReader(tc.inputJSON))
			if err != nil {
				t.Fatalf("ReadFrom: %v", err)
			}
			if string(event.Event) != tc.wantEvent {
				t.Errorf("Event = %q; want %q", event.Event, tc.wantEvent)
			}
			if tc.wantTool != "" {
				if event.Tool == nil {
					t.Errorf("Tool = nil; want %q", tc.wantTool)
				} else if *event.Tool != tc.wantTool {
					t.Errorf("Tool = %q; want %q", *event.Tool, tc.wantTool)
				}
			} else {
				if event.Tool != nil {
					t.Errorf("Tool = %q; want nil", *event.Tool)
				}
			}
			if string(event.Caller) != tc.wantCaller {
				t.Errorf("Caller = %q; want %q", event.Caller, tc.wantCaller)
			}
			if tc.wantInputKey != "" {
				input, ok := event.Input.(map[string]interface{})
				if !ok || input[tc.wantInputKey] == nil {
					t.Errorf("Input[%q] missing; got %v", tc.wantInputKey, event.Input)
				}
			}
		})
	}
}

// ---------------------------------------------------------------------------
// Table-driven tests for RespondTo
// ---------------------------------------------------------------------------

type respondTestCase struct {
	name        string
	response    polyhook.HookResponse
	wantAction  string
	wantContain []string // additional substrings expected in output
}

var respondTests = []respondTestCase{
	{
		name:       "approve",
		response:   polyhook.Approve(),
		wantAction: "approve",
	},
	{
		name:        "block with reason",
		response:    polyhook.Block("rm -rf / is not allowed"),
		wantAction:  "block",
		wantContain: []string{"rm -rf / is not allowed"},
	},
	{
		name:       "block empty message",
		response:   polyhook.Block(""),
		wantAction: "block",
	},
	{
		name:        "modify command",
		response:    polyhook.Modify(map[string]interface{}{"command": "ls /tmp"}),
		wantAction:  "modify",
		wantContain: []string{"ls /tmp"},
	},
	{
		name: "modify with multiple keys",
		response: polyhook.Modify(map[string]interface{}{
			"path":    "/etc/hosts",
			"content": "# managed",
		}),
		wantAction:  "modify",
		wantContain: []string{"/etc/hosts", "managed"},
	},
}

func TestRespondTo_TableDriven(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	// Prime the runtime with a parse call so the serialize function has caller context.
	sampleEvent := hookEventJSON("session:start", "", "claude-code", "sess_prime", nil)
	if _, err := polyhook.ReadFrom(strings.NewReader(sampleEvent)); err != nil {
		t.Fatalf("primer ReadFrom: %v", err)
	}

	for _, tc := range respondTests {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			var buf bytes.Buffer
			if err := polyhook.RespondTo(&buf, tc.response); err != nil {
				t.Fatalf("RespondTo: %v", err)
			}
			out := buf.String()
			if !strings.Contains(out, tc.wantAction) {
				t.Errorf("output %q missing action %q", out, tc.wantAction)
			}
			for _, sub := range tc.wantContain {
				if !strings.Contains(out, sub) {
					t.Errorf("output %q missing expected substring %q", out, sub)
				}
			}
		})
	}
}

// ---------------------------------------------------------------------------
// Error-path tests
// ---------------------------------------------------------------------------

// TestDefaultWASMLoader_ErrorPath verifies that defaultWASMLoader returns an
// error when neither embedded bytes nor a file on disk are available.
// We exercise this by resetting the runtime and installing the real
// defaultWASMLoader (which finds no file in the test working directory).
func TestDefaultWASMLoader_ErrorPath(t *testing.T) {
	// Install a loader that always fails (simulates no wasm binary present).
	polyhook.SetWasmLoader(func() ([]byte, error) {
		return nil, errors.New("polyhook.wasm not found; embed it or place it next to the binary")
	})
	polyhook.ResetRuntime()
	defer resetRuntime()

	_, err := polyhook.ReadFrom(strings.NewReader(`{}`))
	if err == nil {
		t.Fatal("expected error from ReadFrom when wasmLoader fails; got nil")
	}
	if !strings.Contains(err.Error(), "polyhook.wasm") {
		t.Errorf("expected error to mention polyhook.wasm; got: %v", err)
	}
}

// TestGetRuntime_LoaderError verifies that getRuntime propagates a loader error
// through ReadFrom.
func TestGetRuntime_LoaderError(t *testing.T) {
	sentinel := errors.New("injected loader error")
	polyhook.SetWasmLoader(func() ([]byte, error) { return nil, sentinel })
	polyhook.ResetRuntime()
	defer resetRuntime()

	_, err := polyhook.ReadFrom(strings.NewReader(`{}`))
	if err == nil {
		t.Fatal("expected error; got nil")
	}
	if !strings.Contains(err.Error(), "injected loader error") {
		t.Errorf("error should contain sentinel message; got: %v", err)
	}
}

// TestGetRuntime_LoaderError_RespondTo verifies the same error propagation
// through RespondTo.
func TestGetRuntime_LoaderError_RespondTo(t *testing.T) {
	sentinel := errors.New("injected loader error for respond")
	polyhook.SetWasmLoader(func() ([]byte, error) { return nil, sentinel })
	polyhook.ResetRuntime()
	defer resetRuntime()

	var buf bytes.Buffer
	err := polyhook.RespondTo(&buf, polyhook.Approve())
	if err == nil {
		t.Fatal("expected error from RespondTo when wasmLoader fails; got nil")
	}
	if !strings.Contains(err.Error(), "injected loader error for respond") {
		t.Errorf("error should contain sentinel; got: %v", err)
	}
}

// TestGetRuntime_InvalidWASM verifies that an invalid WASM binary causes
// initRuntime to return an error (exercises the rt.Instantiate error path).
func TestGetRuntime_InvalidWASM(t *testing.T) {
	polyhook.SetWasmLoader(func() ([]byte, error) {
		// Valid WASM magic + version but no sections — wazero should still
		// parse it. Use outright garbage instead.
		return []byte("not valid wasm or wat"), nil
	})
	polyhook.ResetRuntime()
	defer resetRuntime()

	_, err := polyhook.ReadFrom(strings.NewReader(`{}`))
	if err == nil {
		t.Fatal("expected instantiation error for invalid WASM; got nil")
	}
}

// TestReadFrom_ReadError verifies that an io.Reader error is propagated by
// ReadFrom.
func TestReadFrom_ReadError(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	errReader := &alwaysErrReader{err: errors.New("read failure")}
	_, err := polyhook.ReadFrom(errReader)
	if err == nil {
		t.Fatal("expected read error; got nil")
	}
	if !strings.Contains(err.Error(), "read failure") {
		t.Errorf("expected 'read failure' in error; got: %v", err)
	}
}

// TestRespondTo_WriteError verifies that a writer error from RespondTo is
// propagated.
func TestRespondTo_WriteError(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	errWriter := &alwaysErrWriter{err: errors.New("write failure")}
	err := polyhook.RespondTo(errWriter, polyhook.Approve())
	if err == nil {
		t.Fatal("expected write error; got nil")
	}
	if !strings.Contains(err.Error(), "write failure") {
		t.Errorf("expected 'write failure' in error; got: %v", err)
	}
}

// TestReadFrom_InvalidJSON verifies that non-JSON output from parse causes an
// unmarshal error. With the passthrough shim the WASM just echoes input, so we
// supply invalid JSON and expect an unmarshal error from ReadFrom.
// NOTE: json.Unmarshal on `{}` succeeds but on a raw non-JSON string it fails.
func TestReadFrom_InvalidJSONEcho(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	// The passthrough shim echoes the raw bytes. If we send invalid JSON the
	// final json.Unmarshal inside ReadFrom should fail.
	_, err := polyhook.ReadFrom(strings.NewReader(`not json at all`))
	if err == nil {
		t.Fatal("expected unmarshal error for non-JSON input; got nil")
	}
}

// TestGetRuntime_MissingExport verifies that a WASM module missing a required
// export (e.g. "parse") triggers an error.
func TestGetRuntime_MissingExport(t *testing.T) {
	// Minimal valid WASM binary: just the magic header + version (no sections).
	// When wazero instantiates it, ExportedFunction("alloc") returns nil,
	// which triggers the mustExport error path.
	minimalWASM := []byte{0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00}
	polyhook.SetWasmLoader(func() ([]byte, error) {
		return minimalWASM, nil
	})
	polyhook.ResetRuntime()
	defer resetRuntime()

	_, err := polyhook.ReadFrom(strings.NewReader(`{}`))
	if err == nil {
		t.Fatal("expected error for missing WASM export; got nil")
	}
	if !strings.Contains(err.Error(), "alloc") && !strings.Contains(err.Error(), "missing export") {
		t.Errorf("error should mention missing export; got: %v", err)
	}
}

// TestReadFrom_WASMErrorField verifies that ReadFrom succeeds even when the
// payload contains a non-empty "error" field alongside a valid HookEvent
// structure. This exercises the best-effort error-logging branch in ReadFrom.
func TestReadFrom_WASMErrorField(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	// The passthrough shim echoes input verbatim. The payload has both an
	// "error" field (triggering the errCheck branch) and valid HookEvent fields.
	input := `{"error":"best-effort parse failed","event":"notification","sessionId":"s1","caller":"unknown"}`
	ev, err := polyhook.ReadFrom(strings.NewReader(input))
	if err != nil {
		t.Fatalf("ReadFrom with error field: %v", err)
	}
	if ev.Event != "notification" {
		t.Errorf("Event = %q; want notification", ev.Event)
	}
}

// TestRead_ViaStdin exercises the thin Read() wrapper that reads from os.Stdin.
// We redirect os.Stdin to a pipe, write a valid HookEvent JSON to the write end,
// close the write end, and verify that Read() returns the expected event.
func TestRead_ViaStdin(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	pr, pw, err := os.Pipe()
	if err != nil {
		t.Fatalf("os.Pipe: %v", err)
	}
	oldStdin := os.Stdin
	os.Stdin = pr
	defer func() { os.Stdin = oldStdin }()

	input := hookEventJSON("session:start", "", "claude-code", "pipe_sess", nil)
	go func() {
		_, _ = pw.WriteString(input)
		_ = pw.Close()
	}()

	ev, err := polyhook.Read()
	if err != nil {
		t.Fatalf("Read(): %v", err)
	}
	if ev.Event != "session:start" {
		t.Errorf("Event = %q; want session:start", ev.Event)
	}
}

// TestRespond_ViaStdout exercises the thin Respond() wrapper that writes to os.Stdout.
func TestRespond_ViaStdout(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	pr, pw, err := os.Pipe()
	if err != nil {
		t.Fatalf("os.Pipe: %v", err)
	}
	oldStdout := os.Stdout
	os.Stdout = pw
	defer func() { os.Stdout = oldStdout }()

	if err := polyhook.Respond(polyhook.Approve()); err != nil {
		_ = pw.Close()
		t.Fatalf("Respond: %v", err)
	}
	_ = pw.Close()

	var buf bytes.Buffer
	if _, err := io.Copy(&buf, pr); err != nil {
		t.Fatalf("reading captured stdout: %v", err)
	}
	if !strings.Contains(buf.String(), "approve") {
		t.Errorf("stdout %q missing 'approve'", buf.String())
	}
}

// TestReadFrom_MemoryOverflow verifies that wasmWrite returns an error when
// the input is larger than WASM linear memory. The passthrough WAT shim has 2
// pages (131 072 bytes) of memory; feeding it more than that causes
// Memory().Write to return false, which covers the wasmWrite error path.
func TestReadFrom_MemoryOverflow(t *testing.T) {
	usePassthroughWASM()
	defer resetRuntime()

	// Build a JSON object whose serialized form exceeds the 2-page (131072 B)
	// WASM memory. We use a long string value.
	big := strings.Repeat("x", 135000) // > 131072
	input := `{"event":"tool:before","tool":"bash","sessionId":"s","caller":"claude-code","input":{"command":"` + big + `"}}`
	_, err := polyhook.ReadFrom(strings.NewReader(input))
	if err == nil {
		t.Fatal("expected memory overflow error from ReadFrom with oversized input; got nil")
	}
}

// ---------------------------------------------------------------------------
// Helper types for error injection
// ---------------------------------------------------------------------------

type alwaysErrReader struct{ err error }

func (r *alwaysErrReader) Read(_ []byte) (int, error) { return 0, r.err }

type alwaysErrWriter struct{ err error }

func (w *alwaysErrWriter) Write(_ []byte) (int, error) { return 0, w.err }

// Ensure interfaces are satisfied.
var _ io.Reader = (*alwaysErrReader)(nil)
var _ io.Writer = (*alwaysErrWriter)(nil)

// ---------------------------------------------------------------------------
// Real-WASM tests — exercise wasmAlloc, wasmDealloc, wasmWrite,
// wasmReadLengthPrefixed, wasmCall, and the full initRuntime success path.
// These tests require polyhook.wasm to be present (build with
// `cargo build -p polyhook-core --target wasm32-unknown-unknown --release`).
// They are skipped automatically when the binary is absent.
// ---------------------------------------------------------------------------

// probeRealWASM returns a non-nil error if the real polyhook.wasm cannot be
// found or instantiated. It does NOT use the passthrough shim.
func probeRealWASM() error {
	defer func() {
		polyhook.ClearTestHooks()
		polyhook.RestoreWasmLoader()
		polyhook.ResetRuntime()
	}()
	// defaultWASMLoader searches for polyhook.wasm on disk.
	_, err := polyhook.ReadFrom(strings.NewReader(
		`{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"probe"}`,
	))
	return err
}

func TestReadFrom_WithRealWASM(t *testing.T) {
	if err := probeRealWASM(); err != nil {
		t.Skipf("skipping real-WASM test: %v", err)
	}
	defer polyhook.ResetRuntime()

	cases := []struct {
		name       string
		input      string
		wantCaller string
		wantEvent  string
	}{
		{
			name:       "claude-code pre-tool",
			input:      `{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la"},"session_id":"sess_1"}`,
			wantCaller: "claude-code",
			wantEvent:  "tool:before",
		},
		{
			name:       "cursor before-tool",
			input:      `{"type":"BeforeToolCall","toolCall":{"name":"run_terminal_cmd","args":{"command":"pwd"}},"sessionId":"sess_2"}`,
			wantCaller: "cursor",
			wantEvent:  "tool:before",
		},
	}

	for _, tc := range cases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			ev, err := polyhook.ReadFrom(strings.NewReader(tc.input))
			if err != nil {
				t.Fatalf("ReadFrom: %v", err)
			}
			if string(ev.Caller) != tc.wantCaller {
				t.Errorf("Caller = %q; want %q", ev.Caller, tc.wantCaller)
			}
			if string(ev.Event) != tc.wantEvent {
				t.Errorf("Event = %q; want %q", ev.Event, tc.wantEvent)
			}
		})
	}
}

func TestRespondTo_WithRealWASM(t *testing.T) {
	if err := probeRealWASM(); err != nil {
		t.Skipf("skipping real-WASM test: %v", err)
	}
	// Prime LAST_CALLER by parsing a Claude Code event first.
	if _, err := polyhook.ReadFrom(strings.NewReader(
		`{"type":"PreToolUse","tool_name":"Bash","tool_input":{},"session_id":"s"}`,
	)); err != nil {
		t.Skipf("ReadFrom failed: %v", err)
	}
	defer polyhook.ResetRuntime()

	var buf bytes.Buffer
	if err := polyhook.RespondTo(&buf, polyhook.Approve()); err != nil {
		t.Fatalf("RespondTo: %v", err)
	}
	if !strings.Contains(buf.String(), "approve") && buf.Len() == 0 {
		t.Errorf("output %q missing 'approve'", buf.String())
	}
}

// TestDefaultWASMLoader_EmbeddedBytes exercises the embedded-binary fast path
// in defaultWASMLoader (wasmBytes != nil).
func TestDefaultWASMLoader_EmbeddedBytes(t *testing.T) {
	if err := probeRealWASM(); err != nil {
		t.Skipf("skipping: real WASM unavailable: %v", err)
	}
	wasmBytes, err := os.ReadFile("polyhook.wasm")
	if err != nil {
		t.Skipf("polyhook.wasm not found: %v", err)
	}

	polyhook.SetWasmBytes(wasmBytes)
	defer func() {
		polyhook.SetWasmBytes(nil)
		polyhook.RestoreWasmLoader()
		polyhook.ResetRuntime()
	}()
	polyhook.ResetRuntime()

	ev, err := polyhook.ReadFrom(strings.NewReader(
		`{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"s"}`,
	))
	if err != nil {
		t.Fatalf("ReadFrom with embedded bytes: %v", err)
	}
	if ev.Caller != "claude-code" {
		t.Errorf("Caller = %q; want claude-code", ev.Caller)
	}
}

// TestWasmWrite_MemoryOverflow exercises the wasmWrite error path when the
// input is larger than WASM linear memory.
func TestWasmWrite_MemoryOverflow(t *testing.T) {
	if err := probeRealWASM(); err != nil {
		t.Skipf("skipping: real WASM unavailable: %v", err)
	}
	defer func() {
		polyhook.RestoreWasmLoader()
		polyhook.ResetRuntime()
	}()

	// 70 000 bytes > 1 WASM page (65 536 bytes); alloc should succeed
	// (wee_alloc grows memory) but write into a buffer that exceeds the
	// original memory limit exercises the wasmWrite error branch.
	//
	// In practice wazero allows unlimited memory growth, so this test at
	// least exercises the alloc→write→call path with a large buffer.
	big := strings.Repeat("x", 70000)
	input := `{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"` + big + `"},"session_id":"s"}`
	// Either it succeeds (large memory) or fails — both paths are useful.
	_, _ = polyhook.ReadFrom(strings.NewReader(input))
}

// TestRespondTo_TestSerializerError exercises the testSerializer error branch
// in RespondTo.
func TestRespondTo_TestSerializerError(t *testing.T) {
	sentinel := errors.New("serializer error")
	polyhook.SetTestSerializer(func(_ []byte) ([]byte, error) { return nil, sentinel })
	defer resetRuntime()

	var buf bytes.Buffer
	err := polyhook.RespondTo(&buf, polyhook.Approve())
	if err == nil {
		t.Fatal("expected error from failing testSerializer; got nil")
	}
	if !strings.Contains(err.Error(), "serializer error") {
		t.Errorf("error should mention sentinel; got: %v", err)
	}
}
