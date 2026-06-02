use crate::detect::detect_caller;
use crate::events::normalize_event;
use crate::tools::normalize_tool;
use crate::types::{CallerKind, HookEvent, HookEventEvent};

/// Parse raw stdin bytes into a normalized [`HookEvent`].
pub fn parse_event(raw: &[u8]) -> Result<HookEvent, String> {
    let val: serde_json::Value =
        serde_json::from_slice(raw).map_err(|e| format!("JSON parse error: {e}"))?;

    let caller = detect_caller(&val);

    // --- event name ---
    let raw_event = extract_event_field(&val, &caller);
    let event_str = normalize_event(&raw_event, &caller);
    let event = event_str
        .parse::<HookEventEvent>()
        .unwrap_or(HookEventEvent::Notification);

    // --- tool name ---
    let raw_tool = extract_tool_field(&val, &caller);
    let tool = raw_tool.map(|t| normalize_tool(&t, &caller));

    // --- input / output ---
    let input = extract_input(&val, &caller);
    let output = extract_output(&val, &caller);

    // --- session / agent ids ---
    let session_id = extract_session_id(&val);
    let agent_id = extract_agent_id(&val);

    Ok(HookEvent {
        event,
        tool,
        input,
        output,
        session_id,
        agent_id,
        caller,
    })
}

// ---------------------------------------------------------------------------
// Field extraction helpers
// ---------------------------------------------------------------------------

fn str_field<'a>(val: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    val.get(key).and_then(|v| v.as_str())
}

fn extract_event_field(val: &serde_json::Value, caller: &CallerKind) -> String {
    let candidates: &[&str] = match caller {
        CallerKind::ClaudeCode => &["event", "hookEvent", "hook_event", "type"],
        CallerKind::Cursor => &["type", "event"],
        CallerKind::Windsurf => &["event", "type"],
        CallerKind::Cline => &["type", "event"],
        CallerKind::Amp => &["kind", "event", "type"],
        CallerKind::Unknown => &["event", "type", "kind", "hookEvent"],
    };

    for key in candidates {
        if let Some(s) = str_field(val, key) {
            return s.to_owned();
        }
    }
    String::new()
}

fn extract_tool_field(val: &serde_json::Value, caller: &CallerKind) -> Option<String> {
    match caller {
        CallerKind::ClaudeCode => str_field(val, "tool_name").map(|s| s.to_owned()),
        CallerKind::Cursor => val
            .get("toolCall")
            .and_then(|tc| tc.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_owned()),
        CallerKind::Windsurf => str_field(val, "tool").map(|s| s.to_owned()),
        CallerKind::Cline => str_field(val, "toolName").map(|s| s.to_owned()),
        CallerKind::Amp => str_field(val, "name").map(|s| s.to_owned()),
        CallerKind::Unknown => {
            for key in &["tool_name", "toolName", "tool", "name"] {
                if let Some(s) = str_field(val, key) {
                    return Some(s.to_owned());
                }
            }
            None
        }
    }
}

fn into_map(v: serde_json::Value) -> Option<serde_json::Map<String, serde_json::Value>> {
    match v {
        serde_json::Value::Object(m) => Some(m),
        _ => None,
    }
}

fn extract_input(
    val: &serde_json::Value,
    caller: &CallerKind,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let raw = match caller {
        CallerKind::ClaudeCode => val.get("tool_input").cloned(),
        CallerKind::Cursor => val.get("toolCall").and_then(|tc| tc.get("args")).cloned(),
        CallerKind::Windsurf => val.get("parameters").cloned(),
        CallerKind::Cline => val.get("args").cloned().or_else(|| val.get("input").cloned()),
        CallerKind::Amp => val.get("args").cloned().or_else(|| val.get("input").cloned()),
        CallerKind::Unknown => {
            for key in &["tool_input", "args", "parameters", "input"] {
                if let Some(v) = val.get(key) {
                    return into_map(v.clone());
                }
            }
            None
        }
    };
    raw.and_then(into_map)
}

fn extract_output(
    val: &serde_json::Value,
    caller: &CallerKind,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let raw = match caller {
        CallerKind::ClaudeCode => val.get("tool_output").cloned(),
        CallerKind::Cursor => val
            .get("toolCall")
            .and_then(|tc| tc.get("result"))
            .cloned(),
        CallerKind::Windsurf => val.get("result").cloned(),
        CallerKind::Cline => val.get("result").cloned().or_else(|| val.get("output").cloned()),
        CallerKind::Amp => val.get("result").cloned().or_else(|| val.get("output").cloned()),
        CallerKind::Unknown => {
            for key in &["tool_output", "result", "output"] {
                if let Some(v) = val.get(key) {
                    return into_map(v.clone());
                }
            }
            None
        }
    };
    raw.and_then(into_map)
}

fn extract_session_id(val: &serde_json::Value) -> String {
    for key in &["session_id", "sessionId", "session"] {
        if let Some(s) = str_field(val, key) {
            return s.to_owned();
        }
    }
    String::new()
}

fn extract_agent_id(val: &serde_json::Value) -> Option<String> {
    for key in &["agent_id", "agentId", "agent"] {
        if let Some(s) = str_field(val, key) {
            return Some(s.to_owned());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
        // No env vars + JSON shape → Unknown; tool_name present → hits extract_tool_field Unknown branch
        // No session_id/sessionId/session → hits String::new() in extract_session_id
        // No event/type/kind/hookEvent → hits String::new() in extract_event_field
        let raw = br#"{"tool_name": "bash"}"#;
        let evt = parse_event(raw).expect("parse failed");
        assert_eq!(evt.caller, CallerKind::Unknown);
        assert_eq!(evt.tool.as_deref(), Some("bash"));
        assert_eq!(evt.session_id, "");
    }

    #[test]
    fn non_object_tool_input_returns_none() {
        // ClaudeCode shape but tool_input is a string → into_map returns None for non-Object
        let raw =
            br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":"not-an-object","session_id":"s1"}"#;
        let evt = parse_event(raw).expect("parse failed");
        assert_eq!(evt.caller, CallerKind::ClaudeCode);
        assert!(evt.input.is_none());
    }

    #[test]
    fn unknown_caller_input_found_via_tool_input_key() {
        // tool_input present but no tool_name → Unknown caller
        // → extract_input Unknown branch hits return into_map(v.clone())
        let raw = br#"{"tool_input": {"cmd": "ls"}, "session_id": "s1"}"#;
        let evt = parse_event(raw).expect("parse failed");
        assert_eq!(evt.caller, CallerKind::Unknown);
        let input = evt.input.expect("input should be present");
        assert_eq!(input["cmd"], json!("ls"));
    }

    #[test]
    fn unknown_caller_output_found_via_tool_output_key() {
        // tool_output present, no tool_name → Unknown caller
        // → extract_output Unknown branch hits return into_map(v.clone())
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
}
