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
            "gemini-cli" | "geminicli" => return CallerKind::GeminiCli,
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
    if std::env::var("GEMINI_PROJECT_DIR").is_ok() {
        return CallerKind::GeminiCli;
    }

    // 3. JSON shape heuristics
    if let Some(obj) = stdin.as_object() {
        let has = |key: &str| obj.contains_key(key);
        let str_val = |key: &str| obj.get(key).and_then(|v| v.as_str()).unwrap_or("");

        // Gemini CLI: hook_event_name with Gemini-specific values.
        // Checked before the Claude Code heuristic because both send tool_name + tool_input.
        match str_val("hook_event_name") {
            "BeforeTool" | "AfterTool" | "BeforeAgent" | "AfterAgent" | "BeforeModel"
            | "AfterModel" | "BeforeToolSelection" | "PreCompress" | "SessionStart"
            | "SessionEnd" => return CallerKind::GeminiCli,
            _ => {}
        }

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
#[path = "detect_tests.rs"]
mod tests;
