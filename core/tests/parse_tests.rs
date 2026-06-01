//! Integration tests for `parse_event` using the fixture JSON files.

use polyhook_core::{parse::parse_event, CallerKind};
use serde_json::json;

fn fixture(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
}

// ---------------------------------------------------------------------------
// Claude Code
// ---------------------------------------------------------------------------

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
    // Bare startup fixture has no tool_name/toolCall — heuristic detects Unknown.
    // Non-canonical event name "Startup" falls back to HookEventEvent::Notification.
    let raw = fixture("claude-code-startup.json");
    let evt = parse_event(&raw).expect("parse failed");

    assert_eq!(evt.caller, CallerKind::Unknown);
    assert_eq!(evt.session_id, "sess_cc_123");
    assert!(evt.tool.is_none());
    // "Startup" is not a canonical HookEventEvent variant; falls back to "notification".
    assert_eq!(evt.event.to_string(), "notification");
}

// ---------------------------------------------------------------------------
// Cursor
// ---------------------------------------------------------------------------

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
    // No toolCall field → Cursor heuristic doesn't fire → Unknown.
    let raw = fixture("cursor-session-start.json");
    let evt = parse_event(&raw).expect("parse failed");

    assert_eq!(evt.caller, CallerKind::Unknown);
    assert_eq!(evt.session_id, "sess_cur_456");
    assert!(evt.tool.is_none());
    // "SessionStart" is not canonical; falls back to "notification".
    assert_eq!(evt.event.to_string(), "notification");
}

// ---------------------------------------------------------------------------
// Windsurf
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Cline
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Amp
// ---------------------------------------------------------------------------

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
