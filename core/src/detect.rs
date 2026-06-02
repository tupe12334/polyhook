use crate::types::CallerKind;

/// Detect which agent is calling the hook.
///
/// Priority:
/// 1. `POLYHOOK_CALLER` env var (explicit override)
/// 2. Agent-specific env vars
/// 3. Heuristics on the raw stdin JSON shape
/// 4. `Unknown`
pub fn detect_caller(stdin: &serde_json::Value) -> CallerKind {
    // 1. Explicit override via env var
    if let Ok(val) = std::env::var("POLYHOOK_CALLER") {
        match val.to_lowercase().as_str() {
            "claude-code" | "claudecode" => return CallerKind::ClaudeCode,
            "cursor" => return CallerKind::Cursor,
            "windsurf" => return CallerKind::Windsurf,
            "cline" => return CallerKind::Cline,
            "amp" => return CallerKind::Amp,
            _ => {}
        }
    }

    // 2. Agent-specific env vars
    if std::env::var("CLAUDE_CODE_VERSION").is_ok() {
        return CallerKind::ClaudeCode;
    }
    if std::env::var("CURSOR_SESSION_ID").is_ok() {
        return CallerKind::Cursor;
    }
    if std::env::var("WINDSURF_SESSION_ID").is_ok() {
        return CallerKind::Windsurf;
    }
    if std::env::var("CLINE_SESSION_ID").is_ok() {
        return CallerKind::Cline;
    }
    if std::env::var("AMP_SESSION_ID").is_ok() {
        return CallerKind::Amp;
    }

    // 3. JSON shape heuristics
    if let Some(obj) = stdin.as_object() {
        let has = |key: &str| obj.contains_key(key);

        if has("tool_name") && has("tool_input") {
            return CallerKind::ClaudeCode;
        }
        if has("type") && has("toolCall") {
            return CallerKind::Cursor;
        }
        if has("event") && has("parameters") {
            return CallerKind::Windsurf;
        }
        // Cline uses toolName (not toolCall)
        if has("type") && has("toolName") && !has("toolCall") {
            return CallerKind::Cline;
        }
        if has("kind") {
            return CallerKind::Amp;
        }
    }

    CallerKind::Unknown
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::detect_caller;
    use crate::CallerKind;

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
    fn claude_code_version_env_var_detected() {
        let val = serde_json::json!({});
        with_clean_env(|| {
            temp_env::with_var("CLAUDE_CODE_VERSION", Some("1.0.0"), || {
                assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
            });
        });
    }

    #[test]
    fn cursor_session_id_env_var_detected() {
        let val = serde_json::json!({});
        with_clean_env(|| {
            temp_env::with_var("CURSOR_SESSION_ID", Some("cursor-sess-abc"), || {
                assert_eq!(detect_caller(&val), CallerKind::Cursor);
            });
        });
    }

    #[test]
    fn windsurf_session_id_env_var_detected() {
        let val = serde_json::json!({});
        with_clean_env(|| {
            temp_env::with_var("WINDSURF_SESSION_ID", Some("ws-sess-xyz"), || {
                assert_eq!(detect_caller(&val), CallerKind::Windsurf);
            });
        });
    }

    #[test]
    fn cline_session_id_env_var_detected() {
        let val = serde_json::json!({});
        with_clean_env(|| {
            temp_env::with_var("CLINE_SESSION_ID", Some("cline-sess-999"), || {
                assert_eq!(detect_caller(&val), CallerKind::Cline);
            });
        });
    }

    #[test]
    fn amp_session_id_env_var_detected() {
        let val = serde_json::json!({});
        with_clean_env(|| {
            temp_env::with_var("AMP_SESSION_ID", Some("amp-sess-000"), || {
                assert_eq!(detect_caller(&val), CallerKind::Amp);
            });
        });
    }

    #[test]
    fn polyhook_caller_garbage_falls_through_to_heuristics() {
        let val = serde_json::json!({"tool_name": "Bash", "tool_input": {}, "session_id": "s1"});
        with_clean_env(|| {
            temp_env::with_var("POLYHOOK_CALLER", Some("garbage_value_xyz"), || {
                assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
            });
        });
    }

    #[test]
    fn polyhook_caller_garbage_with_no_heuristic_match_returns_unknown() {
        let val = serde_json::json!({"some_random_key": "some_value"});
        with_clean_env(|| {
            temp_env::with_var("POLYHOOK_CALLER", Some("not_a_known_caller"), || {
                assert_eq!(detect_caller(&val), CallerKind::Unknown);
            });
        });
    }

    #[test]
    fn unknown_json_shape_returns_unknown() {
        let val = serde_json::json!({"foo": "bar"});
        with_clean_env(|| {
            assert_eq!(detect_caller(&val), CallerKind::Unknown);
        });
    }

    #[test]
    fn empty_object_returns_unknown() {
        let val = serde_json::json!({});
        with_clean_env(|| {
            assert_eq!(detect_caller(&val), CallerKind::Unknown);
        });
    }

    #[test]
    fn non_object_json_returns_unknown() {
        let val = serde_json::json!(["tool_name", "tool_input"]);
        with_clean_env(|| {
            assert_eq!(detect_caller(&val), CallerKind::Unknown);
        });
    }

    #[test]
    fn polyhook_caller_claudecode_alias_detected() {
        let val = serde_json::json!({});
        with_clean_env(|| {
            temp_env::with_var("POLYHOOK_CALLER", Some("claudecode"), || {
                assert_eq!(detect_caller(&val), CallerKind::ClaudeCode);
            });
        });
    }
}
