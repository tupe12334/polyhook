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
