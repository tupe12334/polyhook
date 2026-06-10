use super::normalize_event;
use crate::CallerKind;

#[test]
fn unknown_caller_returns_original_string() {
    assert_eq!(
        normalize_event("SomeVendorEvent", &CallerKind::Unknown),
        "SomeVendorEvent"
    );
    assert_eq!(
        normalize_event("PreToolUse", &CallerKind::Unknown),
        "PreToolUse"
    );
    assert_eq!(normalize_event("", &CallerKind::Unknown), "");
}

// ClaudeCode — all arms
#[test]
fn cc_startup() {
    assert_eq!(
        normalize_event("Startup", &CallerKind::ClaudeCode),
        "session:start"
    );
}
#[test]
fn cc_stop() {
    assert_eq!(
        normalize_event("Stop", &CallerKind::ClaudeCode),
        "session:stop"
    );
}
#[test]
fn cc_subagent_stop() {
    assert_eq!(
        normalize_event("SubagentStop", &CallerKind::ClaudeCode),
        "agent:stop"
    );
}
#[test]
fn cc_notification() {
    assert_eq!(
        normalize_event("Notification", &CallerKind::ClaudeCode),
        "notification"
    );
}
#[test]
fn cc_unknown_falls_through() {
    assert_eq!(normalize_event("Bogus", &CallerKind::ClaudeCode), "Bogus");
}

// Cursor — all arms
#[test]
fn cur_session_start() {
    assert_eq!(
        normalize_event("SessionStart", &CallerKind::Cursor),
        "session:start"
    );
}
#[test]
fn cur_session_end() {
    assert_eq!(
        normalize_event("SessionEnd", &CallerKind::Cursor),
        "session:stop"
    );
}
#[test]
fn cur_notification() {
    assert_eq!(
        normalize_event("Notification", &CallerKind::Cursor),
        "notification"
    );
}
#[test]
fn cur_unknown_falls_through() {
    assert_eq!(normalize_event("Bogus", &CallerKind::Cursor), "Bogus");
}

// Windsurf — all arms
#[test]
fn ws_session_start() {
    assert_eq!(
        normalize_event("session_start", &CallerKind::Windsurf),
        "session:start"
    );
}
#[test]
fn ws_session_end() {
    assert_eq!(
        normalize_event("session_end", &CallerKind::Windsurf),
        "session:stop"
    );
}
#[test]
fn ws_notification() {
    assert_eq!(
        normalize_event("notification", &CallerKind::Windsurf),
        "notification"
    );
}
#[test]
fn ws_unknown_falls_through() {
    assert_eq!(normalize_event("bogus", &CallerKind::Windsurf), "bogus");
}

// Cline — all arms
#[test]
fn cl_on_start() {
    assert_eq!(
        normalize_event("onStart", &CallerKind::Cline),
        "session:start"
    );
}
#[test]
fn cl_on_stop() {
    assert_eq!(
        normalize_event("onStop", &CallerKind::Cline),
        "session:stop"
    );
}
#[test]
fn cl_unknown_falls_through() {
    assert_eq!(normalize_event("bogus", &CallerKind::Cline), "bogus");
}

// Amp — all arms
#[test]
fn amp_session_start() {
    assert_eq!(
        normalize_event("session.start", &CallerKind::Amp),
        "session:start"
    );
}
#[test]
fn amp_session_stop() {
    assert_eq!(
        normalize_event("session.stop", &CallerKind::Amp),
        "session:stop"
    );
}
#[test]
fn amp_agent_stop() {
    assert_eq!(
        normalize_event("agent.stop", &CallerKind::Amp),
        "agent:stop"
    );
}
#[test]
fn amp_unknown_falls_through() {
    assert_eq!(
        normalize_event("bogus.bogus", &CallerKind::Amp),
        "bogus.bogus"
    );
}

// GeminiCli — all arms
#[test]
fn gc_before_tool() {
    assert_eq!(
        normalize_event("BeforeTool", &CallerKind::GeminiCli),
        "tool:before"
    );
}
#[test]
fn gc_after_tool() {
    assert_eq!(
        normalize_event("AfterTool", &CallerKind::GeminiCli),
        "tool:after"
    );
}
#[test]
fn gc_session_start() {
    assert_eq!(
        normalize_event("SessionStart", &CallerKind::GeminiCli),
        "session:start"
    );
}
#[test]
fn gc_session_end() {
    assert_eq!(
        normalize_event("SessionEnd", &CallerKind::GeminiCli),
        "session:stop"
    );
}
#[test]
fn gc_after_agent() {
    assert_eq!(
        normalize_event("AfterAgent", &CallerKind::GeminiCli),
        "agent:stop"
    );
}
#[test]
fn gc_notification() {
    assert_eq!(
        normalize_event("Notification", &CallerKind::GeminiCli),
        "notification"
    );
}
#[test]
fn gc_before_agent_falls_through() {
    assert_eq!(
        normalize_event("BeforeAgent", &CallerKind::GeminiCli),
        "BeforeAgent"
    );
}
#[test]
fn gc_before_model_falls_through() {
    assert_eq!(
        normalize_event("BeforeModel", &CallerKind::GeminiCli),
        "BeforeModel"
    );
}
#[test]
fn gc_after_model_falls_through() {
    assert_eq!(
        normalize_event("AfterModel", &CallerKind::GeminiCli),
        "AfterModel"
    );
}
#[test]
fn gc_tool_selection_falls_through() {
    assert_eq!(
        normalize_event("BeforeToolSelection", &CallerKind::GeminiCli),
        "BeforeToolSelection"
    );
}
#[test]
fn gc_pre_compress_falls_through() {
    assert_eq!(
        normalize_event("PreCompress", &CallerKind::GeminiCli),
        "PreCompress"
    );
}
#[test]
fn gc_unknown_falls_through() {
    assert_eq!(normalize_event("Bogus", &CallerKind::GeminiCli), "Bogus");
}

// Hermes — all arms
#[test]
fn hermes_before_tool() {
    assert_eq!(
        normalize_event("pre_tool_call", &CallerKind::Hermes),
        "tool:before"
    );
}
#[test]
fn hermes_after_tool() {
    assert_eq!(
        normalize_event("post_tool_call", &CallerKind::Hermes),
        "tool:after"
    );
}
#[test]
fn hermes_pre_llm_call() {
    assert_eq!(
        normalize_event("pre_llm_call", &CallerKind::Hermes),
        "notification"
    );
}
#[test]
fn hermes_session_start() {
    assert_eq!(
        normalize_event("on_session_start", &CallerKind::Hermes),
        "session:start"
    );
}
#[test]
fn hermes_session_end() {
    assert_eq!(
        normalize_event("on_session_end", &CallerKind::Hermes),
        "session:stop"
    );
}
#[test]
fn hermes_session_finalize() {
    assert_eq!(
        normalize_event("on_session_finalize", &CallerKind::Hermes),
        "session:stop"
    );
}
#[test]
fn hermes_subagent_stop() {
    assert_eq!(
        normalize_event("subagent_stop", &CallerKind::Hermes),
        "agent:stop"
    );
}
#[test]
fn hermes_unknown_falls_through() {
    assert_eq!(normalize_event("bogus", &CallerKind::Hermes), "bogus");
}
