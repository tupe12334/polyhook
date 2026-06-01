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
