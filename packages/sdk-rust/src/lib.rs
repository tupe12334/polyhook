// Re-export everything from polyhook-core
pub use polyhook_core::*;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use polyhook_core::detect::detect_caller;
    use polyhook_core::events::normalize_event;
    use polyhook_core::parse::parse_event;
    use polyhook_core::response::serialize_response;
    use polyhook_core::tools::normalize_tool;
    use polyhook_core::types::{CallerKind, HookResponse};

    const CLAUDE_PRE_TOOL_USE: &str = r#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la"},"session_id":"sess_123"}"#;
    const CURSOR_BEFORE_TOOL_CALL: &str = r#"{"type":"BeforeToolCall","toolCall":{"name":"run_terminal_cmd","args":{"command":"ls -la"}},"sessionId":"sess_456"}"#;
    const WINDSURF_PRE_TOOL: &str = r#"{"event":"pre_tool","tool":"run_command","parameters":{"command":"ls -la"},"session":"sess_789"}"#;
    const CLINE_BEFORE_TOOL_USE: &str = r#"{"type":"beforeToolUse","toolName":"execute_command","input":{"command":"ls -la"},"sessionId":"sess_abc"}"#;
    const AMP_TOOL_BEFORE: &str = r#"{"kind":"tool.before","name":"shell","input":{"command":"ls -la"},"sessionId":"sess_def"}"#;

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

    #[test]
    fn parse_claude_code() {
        let event = parse_event(CLAUDE_PRE_TOOL_USE.as_bytes()).expect("parse failed");
        assert_eq!(event.caller, CallerKind::ClaudeCode);
        assert_eq!(event.event.to_string(), "tool:before");
        assert_eq!(event.tool.as_deref(), Some("bash"));
        assert_eq!(event.session_id, "sess_123");
        assert_eq!(event.input.expect("input")["command"], "ls -la");
    }

    #[test]
    fn parse_cursor() {
        let event = parse_event(CURSOR_BEFORE_TOOL_CALL.as_bytes()).expect("parse failed");
        assert_eq!(event.caller, CallerKind::Cursor);
        assert_eq!(event.event.to_string(), "tool:before");
        assert_eq!(event.tool.as_deref(), Some("bash"));
    }

    #[test]
    fn parse_windsurf() {
        let event = parse_event(WINDSURF_PRE_TOOL.as_bytes()).expect("parse failed");
        assert_eq!(event.caller, CallerKind::Windsurf);
        assert_eq!(event.tool.as_deref(), Some("bash"));
    }

    #[test]
    fn parse_cline() {
        let event = parse_event(CLINE_BEFORE_TOOL_USE.as_bytes()).expect("parse failed");
        assert_eq!(event.caller, CallerKind::Cline);
        assert_eq!(event.tool.as_deref(), Some("bash"));
    }

    #[test]
    fn parse_amp() {
        let event = parse_event(AMP_TOOL_BEFORE.as_bytes()).expect("parse failed");
        assert_eq!(event.caller, CallerKind::Amp);
        assert_eq!(event.tool.as_deref(), Some("bash"));
    }

    #[test]
    fn serialize_claude_code_approve() {
        let val = serialize_response(&HookResponse::approve(), &CallerKind::ClaudeCode);
        assert!(val.as_object().unwrap().is_empty());
    }

    #[test]
    fn serialize_claude_code_block() {
        let val = serialize_response(
            &HookResponse::block("dangerous command"),
            &CallerKind::ClaudeCode,
        );
        assert_eq!(val["decision"], "block");
        assert_eq!(val["reason"], "dangerous command");
    }

    #[test]
    fn serialize_cursor_approve() {
        assert_eq!(
            serialize_response(&HookResponse::approve(), &CallerKind::Cursor)["action"],
            "allow"
        );
    }
    #[test]
    fn serialize_cursor_block() {
        assert_eq!(
            serialize_response(&HookResponse::block("x"), &CallerKind::Cursor)["action"],
            "deny"
        );
    }
    #[test]
    fn serialize_windsurf_approve() {
        assert_eq!(
            serialize_response(&HookResponse::approve(), &CallerKind::Windsurf)["allow"],
            true
        );
    }
    #[test]
    fn serialize_cline_approve() {
        assert_eq!(
            serialize_response(&HookResponse::approve(), &CallerKind::Cline)["approved"],
            true
        );
    }
    #[test]
    fn serialize_amp_approve() {
        assert_eq!(
            serialize_response(&HookResponse::approve(), &CallerKind::Amp)["result"],
            "allow"
        );
    }

    #[test]
    fn detect_env_var_claude_code() {
        with_clean_env(|| {
            temp_env::with_var("POLYHOOK_CALLER", Some("claude-code"), || {
                assert_eq!(
                    detect_caller(&serde_json::json!({})),
                    CallerKind::ClaudeCode
                );
            });
        });
    }

    #[test]
    fn detect_heuristic_cursor() {
        let val: serde_json::Value = serde_json::from_str(CURSOR_BEFORE_TOOL_CALL).unwrap();
        with_clean_env(|| {
            assert_eq!(detect_caller(&val), CallerKind::Cursor);
        });
    }

    #[test]
    fn normalize_tool_claude_code_basic() {
        assert_eq!(normalize_tool("Bash", &CallerKind::ClaudeCode), "bash");
        assert_eq!(normalize_tool("Read", &CallerKind::ClaudeCode), "read_file");
    }

    #[test]
    fn normalize_event_claude_code_basic() {
        assert_eq!(
            normalize_event("PreToolUse", &CallerKind::ClaudeCode),
            "tool:before"
        );
        assert_eq!(
            normalize_event("PostToolUse", &CallerKind::ClaudeCode),
            "tool:after"
        );
    }
}
