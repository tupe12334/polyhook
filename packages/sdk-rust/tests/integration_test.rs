use polyhook_core::detect::detect_caller;
use polyhook_core::events::normalize_event;
use polyhook_core::parse::parse_event;
use polyhook_core::response::serialize_response;
use polyhook_core::tools::normalize_tool;
use polyhook_core::types::{CallerKind, HookResponse};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const CLAUDE_PRE_TOOL_USE: &str =
    r#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la"},"session_id":"sess_123"}"#;

const CURSOR_BEFORE_TOOL_CALL: &str =
    r#"{"type":"BeforeToolCall","toolCall":{"name":"run_terminal_cmd","args":{"command":"ls -la"}},"sessionId":"sess_456"}"#;

const WINDSURF_PRE_TOOL: &str =
    r#"{"event":"pre_tool","tool":"run_command","parameters":{"command":"ls -la"},"session":"sess_789"}"#;

const CLINE_BEFORE_TOOL_USE: &str =
    r#"{"type":"beforeToolUse","toolName":"execute_command","input":{"command":"ls -la"},"sessionId":"sess_abc"}"#;

const AMP_TOOL_BEFORE: &str =
    r#"{"kind":"tool.before","name":"shell","input":{"command":"ls -la"},"sessionId":"sess_def"}"#;

// ---------------------------------------------------------------------------
// parse_event tests
// ---------------------------------------------------------------------------

#[test]
fn test_parse_event_claude_code_pre_tool_use() {
    let event = parse_event(CLAUDE_PRE_TOOL_USE.as_bytes()).expect("parse failed");
    assert_eq!(event.caller, CallerKind::ClaudeCode);
    assert_eq!(event.event.to_string(), "tool:before");
    assert_eq!(event.tool.as_deref(), Some("bash"));
    assert_eq!(event.session_id, "sess_123");
    let input = event.input.expect("input should be present");
    assert_eq!(input["command"], "ls -la");
}

#[test]
fn test_parse_event_cursor_before_tool_call() {
    let event = parse_event(CURSOR_BEFORE_TOOL_CALL.as_bytes()).expect("parse failed");
    assert_eq!(event.caller, CallerKind::Cursor);
    assert_eq!(event.event.to_string(), "tool:before");
    assert_eq!(event.tool.as_deref(), Some("bash"));
    assert_eq!(event.session_id, "sess_456");
    let input = event.input.expect("input should be present");
    assert_eq!(input["command"], "ls -la");
}

#[test]
fn test_parse_event_windsurf_pre_tool() {
    let event = parse_event(WINDSURF_PRE_TOOL.as_bytes()).expect("parse failed");
    assert_eq!(event.caller, CallerKind::Windsurf);
    assert_eq!(event.event.to_string(), "tool:before");
    assert_eq!(event.tool.as_deref(), Some("bash"));
    assert_eq!(event.session_id, "sess_789");
    let input = event.input.expect("input should be present");
    assert_eq!(input["command"], "ls -la");
}

#[test]
fn test_parse_event_cline_before_tool_use() {
    let event = parse_event(CLINE_BEFORE_TOOL_USE.as_bytes()).expect("parse failed");
    assert_eq!(event.caller, CallerKind::Cline);
    assert_eq!(event.event.to_string(), "tool:before");
    assert_eq!(event.tool.as_deref(), Some("bash"));
    assert_eq!(event.session_id, "sess_abc");
    let input = event.input.expect("input should be present");
    assert_eq!(input["command"], "ls -la");
}

#[test]
fn test_parse_event_amp_tool_before() {
    let event = parse_event(AMP_TOOL_BEFORE.as_bytes()).expect("parse failed");
    assert_eq!(event.caller, CallerKind::Amp);
    assert_eq!(event.event.to_string(), "tool:before");
    assert_eq!(event.tool.as_deref(), Some("bash"));
    assert_eq!(event.session_id, "sess_def");
    let input = event.input.expect("input should be present");
    assert_eq!(input["command"], "ls -la");
}

// ---------------------------------------------------------------------------
// serialize_response tests
// ---------------------------------------------------------------------------

#[test]
fn test_serialize_response_claude_code_approve() {
    let resp = HookResponse::approve();
    let val = serialize_response(&resp, &CallerKind::ClaudeCode);
    // Claude Code approve → empty object {}
    assert!(val.as_object().unwrap().is_empty());
}

#[test]
fn test_serialize_response_claude_code_block() {
    let resp = HookResponse::block("dangerous command");
    let val = serialize_response(&resp, &CallerKind::ClaudeCode);
    assert_eq!(val["decision"], "block");
    assert_eq!(val["reason"], "dangerous command");
}

#[test]
fn test_serialize_response_claude_code_modify() {
    let input = serde_json::json!({"command": "echo safe"});
    let resp = HookResponse::modify(input.clone());
    let val = serialize_response(&resp, &CallerKind::ClaudeCode);
    assert_eq!(val["decision"], "approve");
    assert_eq!(val["tool_input"], input);
}

#[test]
fn test_serialize_response_cursor_approve() {
    let resp = HookResponse::approve();
    let val = serialize_response(&resp, &CallerKind::Cursor);
    assert_eq!(val["action"], "allow");
}

#[test]
fn test_serialize_response_cursor_block() {
    let resp = HookResponse::block("not allowed");
    let val = serialize_response(&resp, &CallerKind::Cursor);
    assert_eq!(val["action"], "deny");
    assert_eq!(val["message"], "not allowed");
}

#[test]
fn test_serialize_response_cursor_modify() {
    let input = serde_json::json!({"command": "echo safe"});
    let resp = HookResponse::modify(input.clone());
    let val = serialize_response(&resp, &CallerKind::Cursor);
    assert_eq!(val["action"], "modify");
    assert_eq!(val["args"], input);
}

#[test]
fn test_serialize_response_windsurf_approve() {
    let resp = HookResponse::approve();
    let val = serialize_response(&resp, &CallerKind::Windsurf);
    assert_eq!(val["allow"], true);
}

#[test]
fn test_serialize_response_windsurf_block() {
    let resp = HookResponse::block("blocked by policy");
    let val = serialize_response(&resp, &CallerKind::Windsurf);
    assert_eq!(val["allow"], false);
    assert_eq!(val["reason"], "blocked by policy");
}

#[test]
fn test_serialize_response_windsurf_modify() {
    let input = serde_json::json!({"command": "echo safe"});
    let resp = HookResponse::modify(input.clone());
    let val = serialize_response(&resp, &CallerKind::Windsurf);
    assert_eq!(val["allow"], true);
    assert_eq!(val["modified_parameters"], input);
}

#[test]
fn test_serialize_response_cline_approve() {
    let resp = HookResponse::approve();
    let val = serialize_response(&resp, &CallerKind::Cline);
    assert_eq!(val["approved"], true);
}

#[test]
fn test_serialize_response_cline_block() {
    let resp = HookResponse::block("cline blocked");
    let val = serialize_response(&resp, &CallerKind::Cline);
    assert_eq!(val["approved"], false);
    assert_eq!(val["reason"], "cline blocked");
}

#[test]
fn test_serialize_response_cline_modify() {
    let input = serde_json::json!({"command": "echo safe"});
    let resp = HookResponse::modify(input.clone());
    let val = serialize_response(&resp, &CallerKind::Cline);
    assert_eq!(val["approved"], true);
    assert_eq!(val["modifiedInput"], input);
}

#[test]
fn test_serialize_response_amp_approve() {
    let resp = HookResponse::approve();
    let val = serialize_response(&resp, &CallerKind::Amp);
    assert_eq!(val["result"], "allow");
}

#[test]
fn test_serialize_response_amp_block() {
    let resp = HookResponse::block("amp blocked");
    let val = serialize_response(&resp, &CallerKind::Amp);
    assert_eq!(val["result"], "deny");
    assert_eq!(val["reason"], "amp blocked");
}

#[test]
fn test_serialize_response_amp_modify() {
    let input = serde_json::json!({"command": "echo safe"});
    let resp = HookResponse::modify(input.clone());
    let val = serialize_response(&resp, &CallerKind::Amp);
    assert_eq!(val["result"], "allow");
    assert_eq!(val["modified"], input);
}

// ---------------------------------------------------------------------------
// Helpers for env-var isolation
// ---------------------------------------------------------------------------

/// All env vars that detect_caller inspects. We unset them all in heuristic
/// tests so that concurrent env-var-override tests cannot interfere.
const AGENT_ENV_VARS: &[&str] = &[
    "POLYHOOK_CALLER",
    "CLAUDE_CODE_VERSION",
    "CURSOR_SESSION_ID",
    "WINDSURF_SESSION_ID",
    "CLINE_SESSION_ID",
    "AMP_SESSION_ID",
];

fn with_clean_env<F: FnOnce()>(f: F) {
    let vars: Vec<(&str, Option<&str>)> = AGENT_ENV_VARS.iter().map(|k| (*k, None)).collect();
    temp_env::with_vars(vars, f);
}

// ---------------------------------------------------------------------------
// detect_caller with POLYHOOK_CALLER env var override
// ---------------------------------------------------------------------------

#[test]
fn test_detect_caller_env_var_override_claude_code() {
    // Use a dummy empty JSON object; the env var should take priority.
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("claude-code"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::ClaudeCode);
        });
    });
}

#[test]
fn test_detect_caller_env_var_override_cursor() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("cursor"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Cursor);
        });
    });
}

#[test]
fn test_detect_caller_env_var_override_windsurf() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("windsurf"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Windsurf);
        });
    });
}

#[test]
fn test_detect_caller_env_var_override_cline() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("cline"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Cline);
        });
    });
}

#[test]
fn test_detect_caller_env_var_override_amp() {
    let val = serde_json::json!({});
    with_clean_env(|| {
        temp_env::with_var("POLYHOOK_CALLER", Some("amp"), || {
            let caller = detect_caller(&val);
            assert_eq!(caller, CallerKind::Amp);
        });
    });
}

#[test]
fn test_detect_caller_heuristic_claude_code() {
    let val: serde_json::Value = serde_json::from_str(CLAUDE_PRE_TOOL_USE).unwrap();
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::ClaudeCode);
    });
}

#[test]
fn test_detect_caller_heuristic_cursor() {
    let val: serde_json::Value = serde_json::from_str(CURSOR_BEFORE_TOOL_CALL).unwrap();
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::Cursor);
    });
}

#[test]
fn test_detect_caller_heuristic_windsurf() {
    let val: serde_json::Value = serde_json::from_str(WINDSURF_PRE_TOOL).unwrap();
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::Windsurf);
    });
}

#[test]
fn test_detect_caller_heuristic_cline() {
    let val: serde_json::Value = serde_json::from_str(CLINE_BEFORE_TOOL_USE).unwrap();
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::Cline);
    });
}

#[test]
fn test_detect_caller_heuristic_amp() {
    let val: serde_json::Value = serde_json::from_str(AMP_TOOL_BEFORE).unwrap();
    with_clean_env(|| {
        let caller = detect_caller(&val);
        assert_eq!(caller, CallerKind::Amp);
    });
}

// ---------------------------------------------------------------------------
// normalize_tool tests
// ---------------------------------------------------------------------------

#[test]
fn test_normalize_tool_claude_code() {
    assert_eq!(normalize_tool("Bash", &CallerKind::ClaudeCode), "bash");
    assert_eq!(normalize_tool("Read", &CallerKind::ClaudeCode), "read_file");
    assert_eq!(normalize_tool("Write", &CallerKind::ClaudeCode), "write_file");
    assert_eq!(normalize_tool("Edit", &CallerKind::ClaudeCode), "edit_file");
    assert_eq!(normalize_tool("Task", &CallerKind::ClaudeCode), "spawn_agent");
    // Unknown tool should pass through unchanged
    assert_eq!(normalize_tool("MyCustomTool", &CallerKind::ClaudeCode), "MyCustomTool");
}

#[test]
fn test_normalize_tool_cursor() {
    assert_eq!(normalize_tool("run_terminal_cmd", &CallerKind::Cursor), "bash");
    assert_eq!(normalize_tool("read_file", &CallerKind::Cursor), "read_file");
    assert_eq!(normalize_tool("edit_file", &CallerKind::Cursor), "write_file");
    assert_eq!(normalize_tool("grep_search", &CallerKind::Cursor), "grep");
    assert_eq!(normalize_tool("web_search", &CallerKind::Cursor), "web_search");
    assert_eq!(normalize_tool("unknown_cursor_tool", &CallerKind::Cursor), "unknown_cursor_tool");
}

#[test]
fn test_normalize_tool_windsurf() {
    assert_eq!(normalize_tool("run_command", &CallerKind::Windsurf), "bash");
    assert_eq!(normalize_tool("read_file", &CallerKind::Windsurf), "read_file");
    assert_eq!(normalize_tool("write_file", &CallerKind::Windsurf), "write_file");
    assert_eq!(normalize_tool("list_directory", &CallerKind::Windsurf), "list_dir");
    assert_eq!(normalize_tool("search_web", &CallerKind::Windsurf), "web_search");
    assert_eq!(normalize_tool("unknown_windsurf_tool", &CallerKind::Windsurf), "unknown_windsurf_tool");
}

#[test]
fn test_normalize_tool_cline() {
    assert_eq!(normalize_tool("execute_command", &CallerKind::Cline), "bash");
    assert_eq!(normalize_tool("read_file", &CallerKind::Cline), "read_file");
    assert_eq!(normalize_tool("write_to_file", &CallerKind::Cline), "write_file");
    assert_eq!(normalize_tool("apply_diff", &CallerKind::Cline), "edit_file");
    assert_eq!(normalize_tool("list_files", &CallerKind::Cline), "list_dir");
    assert_eq!(normalize_tool("unknown_cline_tool", &CallerKind::Cline), "unknown_cline_tool");
}

#[test]
fn test_normalize_tool_amp() {
    assert_eq!(normalize_tool("shell", &CallerKind::Amp), "bash");
    assert_eq!(normalize_tool("file.read", &CallerKind::Amp), "read_file");
    assert_eq!(normalize_tool("file.write", &CallerKind::Amp), "write_file");
    assert_eq!(normalize_tool("file.edit", &CallerKind::Amp), "edit_file");
    assert_eq!(normalize_tool("fs.list", &CallerKind::Amp), "list_dir");
    assert_eq!(normalize_tool("web.search", &CallerKind::Amp), "web_search");
    assert_eq!(normalize_tool("unknown_amp_tool", &CallerKind::Amp), "unknown_amp_tool");
}

// ---------------------------------------------------------------------------
// normalize_event tests
// ---------------------------------------------------------------------------

#[test]
fn test_normalize_event_claude_code() {
    assert_eq!(normalize_event("PreToolUse", &CallerKind::ClaudeCode), "tool:before");
    assert_eq!(normalize_event("PostToolUse", &CallerKind::ClaudeCode), "tool:after");
    assert_eq!(normalize_event("Startup", &CallerKind::ClaudeCode), "session:start");
    assert_eq!(normalize_event("Stop", &CallerKind::ClaudeCode), "session:stop");
    assert_eq!(normalize_event("SubagentStop", &CallerKind::ClaudeCode), "agent:stop");
    assert_eq!(normalize_event("Notification", &CallerKind::ClaudeCode), "notification");
    assert_eq!(normalize_event("UnknownEvent", &CallerKind::ClaudeCode), "UnknownEvent");
}

#[test]
fn test_normalize_event_cursor() {
    assert_eq!(normalize_event("BeforeToolCall", &CallerKind::Cursor), "tool:before");
    assert_eq!(normalize_event("AfterToolCall", &CallerKind::Cursor), "tool:after");
    assert_eq!(normalize_event("SessionStart", &CallerKind::Cursor), "session:start");
    assert_eq!(normalize_event("SessionEnd", &CallerKind::Cursor), "session:stop");
    assert_eq!(normalize_event("Notification", &CallerKind::Cursor), "notification");
    assert_eq!(normalize_event("UnknownEvent", &CallerKind::Cursor), "UnknownEvent");
}

#[test]
fn test_normalize_event_windsurf() {
    assert_eq!(normalize_event("pre_tool", &CallerKind::Windsurf), "tool:before");
    assert_eq!(normalize_event("post_tool", &CallerKind::Windsurf), "tool:after");
    assert_eq!(normalize_event("session_start", &CallerKind::Windsurf), "session:start");
    assert_eq!(normalize_event("session_end", &CallerKind::Windsurf), "session:stop");
    assert_eq!(normalize_event("notification", &CallerKind::Windsurf), "notification");
    assert_eq!(normalize_event("unknown_event", &CallerKind::Windsurf), "unknown_event");
}

#[test]
fn test_normalize_event_cline() {
    assert_eq!(normalize_event("beforeToolUse", &CallerKind::Cline), "tool:before");
    assert_eq!(normalize_event("afterToolUse", &CallerKind::Cline), "tool:after");
    assert_eq!(normalize_event("onStart", &CallerKind::Cline), "session:start");
    assert_eq!(normalize_event("onStop", &CallerKind::Cline), "session:stop");
    assert_eq!(normalize_event("unknownEvent", &CallerKind::Cline), "unknownEvent");
}

#[test]
fn test_normalize_event_amp() {
    assert_eq!(normalize_event("tool.before", &CallerKind::Amp), "tool:before");
    assert_eq!(normalize_event("tool.after", &CallerKind::Amp), "tool:after");
    assert_eq!(normalize_event("session.start", &CallerKind::Amp), "session:start");
    assert_eq!(normalize_event("session.stop", &CallerKind::Amp), "session:stop");
    assert_eq!(normalize_event("agent.stop", &CallerKind::Amp), "agent:stop");
    assert_eq!(normalize_event("unknown.event", &CallerKind::Amp), "unknown.event");
}
