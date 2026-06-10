use super::parse_event;
use crate::CallerKind;
use serde_json::json;

fn fixture(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).expect("fixture file should be readable")
}

#[test]
fn claude_code_pre_tool() {
    let raw = fixture("claude-code-pre-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::ClaudeCode);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_cc_123");
    assert_eq!(evt.agent_id.as_deref(), Some("agent_001"));
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn claude_code_post_tool() {
    let raw = fixture("claude-code-post-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::ClaudeCode);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_cc_123");
    assert!(evt.agent_id.is_none());
    let input = evt.input.expect("input should be present");
    assert_eq!(input["file_path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello world"));
}

#[test]
fn claude_code_startup() {
    let raw = fixture("claude-code-startup.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Unknown);
    assert_eq!(evt.session_id, "sess_cc_123");
    assert!(evt.tool.is_none());
    assert_eq!(evt.event.to_string(), "notification");
}

#[test]
fn cursor_before_tool() {
    let raw = fixture("cursor-before-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Cursor);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_cur_456");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn cursor_after_tool() {
    let raw = fixture("cursor-after-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Cursor);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_cur_456");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello"));
}

#[test]
fn cursor_session_start() {
    let raw = fixture("cursor-session-start.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Unknown);
    assert_eq!(evt.session_id, "sess_cur_456");
    assert!(evt.tool.is_none());
    assert_eq!(evt.event.to_string(), "notification");
}

#[test]
fn windsurf_pre_tool() {
    let raw = fixture("windsurf-pre-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Windsurf);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_ws_789");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn windsurf_post_tool() {
    let raw = fixture("windsurf-post-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Windsurf);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_ws_789");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello"));
}

#[test]
fn cline_before_tool() {
    let raw = fixture("cline-before-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Cline);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_cl_abc");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn cline_after_tool() {
    let raw = fixture("cline-after-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Cline);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_cl_abc");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello"));
}

#[test]
fn amp_tool_before() {
    let raw = fixture("amp-tool-before.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Amp);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_amp_def");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn unknown_caller_tool_found_no_session() {
    let raw = br#"{"tool_name": "bash"}"#;
    let evt = parse_event(raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Unknown);
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "");
}

#[test]
fn claude_code_pre_tool_hook_event_name() {
    let raw = fixture("claude-code-pre-tool-hook-event-name.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::ClaudeCode);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_cc_real");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn claude_code_post_tool_hook_event_name() {
    let raw = fixture("claude-code-post-tool-hook-event-name.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::ClaudeCode);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_cc_real");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["file_path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello world"));
}

#[test]
fn non_object_tool_input_returns_none() {
    let raw =
        br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":"not-an-object","session_id":"s1"}"#;
    let evt = parse_event(raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::ClaudeCode);
    assert!(evt.input.is_none());
}

#[test]
fn unknown_caller_input_found_via_tool_input_key() {
    let raw = br#"{"tool_input": {"cmd": "ls"}, "session_id": "s1"}"#;
    let evt = parse_event(raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Unknown);
    let input = evt.input.expect("input should be present");
    assert_eq!(input["cmd"], json!("ls"));
}

#[test]
fn unknown_caller_output_found_via_tool_output_key() {
    let raw = br#"{"tool_output": {"result": "ok"}, "session_id": "s1"}"#;
    let evt = parse_event(raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Unknown);
    let output = evt.output.expect("output should be present");
    assert_eq!(output["result"], json!("ok"));
}

#[test]
fn amp_tool_after() {
    let raw = fixture("amp-tool-after.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Amp);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_amp_def");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello"));
}

#[test]
fn gemini_cli_before_tool() {
    let raw = fixture("gemini-cli-before-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::GeminiCli);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_gc_001");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn gemini_cli_after_tool() {
    let raw = fixture("gemini-cli-after-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::GeminiCli);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_gc_001");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello world"));
}

#[test]
fn gemini_cli_session_start() {
    let raw = fixture("gemini-cli-session-start.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::GeminiCli);
    assert_eq!(evt.event.to_string(), "session:start");
    assert_eq!(evt.session_id, "sess_gc_001");
    assert!(evt.tool.is_none());
    assert!(evt.input.is_none());
    assert!(evt.output.is_none());
}

#[test]
fn hermes_pre_tool() {
    let raw = fixture("hermes-pre-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Hermes);
    assert_eq!(evt.event.to_string(), "tool:before");
    assert_eq!(evt.tool.as_deref(), Some("bash"));
    assert_eq!(evt.session_id, "sess_hermes_001");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["command"], json!("ls -la"));
    assert!(evt.output.is_none());
}

#[test]
fn hermes_post_tool() {
    let raw = fixture("hermes-post-tool.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Hermes);
    assert_eq!(evt.event.to_string(), "tool:after");
    assert_eq!(evt.tool.as_deref(), Some("read_file"));
    assert_eq!(evt.session_id, "sess_hermes_001");
    let input = evt.input.expect("input should be present");
    assert_eq!(input["path"], json!("/tmp/foo.txt"));
    let output = evt.output.expect("output should be present");
    assert_eq!(output["content"], json!("hello world"));
}

#[test]
fn hermes_session_start() {
    let raw = fixture("hermes-session-start.json");
    let evt = parse_event(&raw).expect("parse failed");
    assert_eq!(evt.caller, CallerKind::Hermes);
    assert_eq!(evt.event.to_string(), "session:start");
    assert_eq!(evt.session_id, "sess_hermes_001");
    assert!(evt.tool.is_none());
    assert!(evt.input.is_none());
    assert!(evt.output.is_none());
}
