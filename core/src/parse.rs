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
        CallerKind::ClaudeCode => &["hook_event_name", "event", "hookEvent", "hook_event", "type"],
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
#[path = "parse_tests.rs"]
mod tests;
