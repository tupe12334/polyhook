package polyhook_test

// polyhook_test.go exercises the Go SDK with table-driven tests.
//
// Because a real polyhook.wasm binary may not be present (it is compiled
// from the Rust crate separately), the integration tests that need the WASM
// runtime are gated by a probe: if the runtime cannot be initialised the
// tests are skipped.
//
// The pure-unit tests (type constructors, JSON serialisation, CallerKind
// constants) always run.

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"

	polyhook "github.com/polyhook/polyhook-go"
)

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// hookEventJSON builds a minimal pre-normalised HookEvent JSON string.
// When polyhook.wasm is absent and a passthrough shim is in use, this is
// exactly what ReadFrom receives.
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

// probeWASM returns a non-nil error when the WASM runtime cannot be
// initialised (e.g. polyhook.wasm is absent).
func probeWASM() error {
	sample := hookEventJSON("session:start", "", "unknown", "probe", nil)
	_, err := polyhook.ReadFrom(strings.NewReader(sample))
	return err
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
		{"ClaudeCode", polyhook.CallerClaudeCode, "claude-code"},
		{"Cursor", polyhook.CallerCursor, "cursor"},
		{"Windsurf", polyhook.CallerWindsurf, "windsurf"},
		{"Cline", polyhook.CallerCline, "cline"},
		{"Amp", polyhook.CallerAmp, "amp"},
		{"Unknown", polyhook.CallerUnknown, "unknown"},
	}
	for _, tc := range cases {
		if tc.got != tc.want {
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
	if ev.SessionID != "sess_abc" {
		t.Errorf("SessionID = %q; want sess_abc", ev.SessionID)
	}
	if ev.AgentID == nil || *ev.AgentID != agentID {
		t.Errorf("AgentID = %v; want %q", ev.AgentID, agentID)
	}
	if ev.Caller != polyhook.CallerClaudeCode {
		t.Errorf("Caller = %q; want %q", ev.Caller, polyhook.CallerClaudeCode)
	}
	if cmd, ok := ev.Input["command"]; !ok || cmd != "ls -la" {
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
	if ev.AgentID != nil {
		t.Errorf("AgentID should be nil; got %q", *ev.AgentID)
	}
	if ev.Input != nil {
		t.Errorf("Input should be nil; got %v", ev.Input)
	}
}

// ---------------------------------------------------------------------------
// Integration tests: ReadFrom / RespondTo
// These run only when the WASM runtime can be initialised.
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
	if err := probeWASM(); err != nil {
		t.Skipf("skipping: WASM runtime unavailable: %v", err)
	}

	for _, tc := range readTests {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			event, err := polyhook.ReadFrom(strings.NewReader(tc.inputJSON))
			if err != nil {
				t.Fatalf("ReadFrom: %v", err)
			}
			if event.Event != tc.wantEvent {
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
			if event.Caller != tc.wantCaller {
				t.Errorf("Caller = %q; want %q", event.Caller, tc.wantCaller)
			}
			if tc.wantInputKey != "" {
				if event.Input == nil || event.Input[tc.wantInputKey] == nil {
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
	if err := probeWASM(); err != nil {
		t.Skipf("skipping: WASM runtime unavailable: %v", err)
	}

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
