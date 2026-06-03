use crate::types::CallerKind;

/// Normalize a vendor-specific event name to the canonical polyhook event name.
///
/// If no mapping exists the original name is returned unchanged.
pub fn normalize_event(vendor: &str, caller: &CallerKind) -> String {
    let canonical = match caller {
        CallerKind::ClaudeCode => normalize_claude_code_event(vendor),
        CallerKind::Cursor => normalize_cursor_event(vendor),
        CallerKind::Windsurf => normalize_windsurf_event(vendor),
        CallerKind::Cline => normalize_cline_event(vendor),
        CallerKind::Amp => normalize_amp_event(vendor),
        CallerKind::GeminiCli => normalize_gemini_cli_event(vendor),
        CallerKind::Unknown => None,
    };

    canonical
        .map(|s| s.to_owned())
        .unwrap_or_else(|| vendor.to_owned())
}

fn normalize_claude_code_event(vendor: &str) -> Option<&'static str> {
    match vendor {
        "PreToolUse" => Some("tool:before"),
        "PostToolUse" => Some("tool:after"),
        "Startup" => Some("session:start"),
        "Stop" => Some("session:stop"),
        "SubagentStop" => Some("agent:stop"),
        "Notification" => Some("notification"),
        _ => None,
    }
}

fn normalize_cursor_event(vendor: &str) -> Option<&'static str> {
    match vendor {
        "BeforeToolCall" => Some("tool:before"),
        "AfterToolCall" => Some("tool:after"),
        "SessionStart" => Some("session:start"),
        "SessionEnd" => Some("session:stop"),
        "Notification" => Some("notification"),
        _ => None,
    }
}

fn normalize_windsurf_event(vendor: &str) -> Option<&'static str> {
    match vendor {
        "pre_tool" => Some("tool:before"),
        "post_tool" => Some("tool:after"),
        "session_start" => Some("session:start"),
        "session_end" => Some("session:stop"),
        "notification" => Some("notification"),
        _ => None,
    }
}

fn normalize_cline_event(vendor: &str) -> Option<&'static str> {
    match vendor {
        "beforeToolUse" => Some("tool:before"),
        "afterToolUse" => Some("tool:after"),
        "onStart" => Some("session:start"),
        "onStop" => Some("session:stop"),
        _ => None,
    }
}

fn normalize_amp_event(vendor: &str) -> Option<&'static str> {
    match vendor {
        "tool.before" => Some("tool:before"),
        "tool.after" => Some("tool:after"),
        "session.start" => Some("session:start"),
        "session.stop" => Some("session:stop"),
        "agent.stop" => Some("agent:stop"),
        _ => None,
    }
}

fn normalize_gemini_cli_event(vendor: &str) -> Option<&'static str> {
    match vendor {
        "BeforeTool" => Some("tool:before"),
        "AfterTool" => Some("tool:after"),
        "SessionStart" => Some("session:start"),
        "SessionEnd" => Some("session:stop"),
        "AfterAgent" => Some("agent:stop"),
        "Notification" => Some("notification"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
