use super::serialize_response;
use crate::{CallerKind, HookResponse};
use serde_json::json;

#[test]
fn claude_code_approve() {
    let val = serialize_response(&HookResponse::approve(), &CallerKind::ClaudeCode);
    assert_eq!(val, json!({}));
}

#[test]
fn claude_code_block() {
    // serialize_response has no event context → falls back to decision:block
    let val = serialize_response(&HookResponse::block("not allowed"), &CallerKind::ClaudeCode);
    assert_eq!(val["decision"], json!("block"));
    assert_eq!(val["reason"], json!("not allowed"));
}

#[test]
fn claude_code_pre_tool_use_block_uses_hook_specific_output() {
    use crate::response::serialize_response_with_event;
    use crate::types::HookEventEvent;
    let val = serialize_response_with_event(
        &HookResponse::block("not allowed"),
        &CallerKind::ClaudeCode,
        Some(HookEventEvent::ToolBefore),
    );
    assert_eq!(
        val["hookSpecificOutput"]["hookEventName"],
        json!("PreToolUse")
    );
    assert_eq!(
        val["hookSpecificOutput"]["permissionDecision"],
        json!("deny")
    );
    assert_eq!(
        val["hookSpecificOutput"]["permissionDecisionReason"],
        json!("not allowed")
    );
    assert_eq!(
        val["hookSpecificOutput"]["additionalContext"],
        serde_json::Value::Null
    );
}

#[test]
fn claude_code_post_tool_use_block_uses_decision() {
    use crate::response::serialize_response_with_event;
    use crate::types::HookEventEvent;
    let val = serialize_response_with_event(
        &HookResponse::block("not allowed"),
        &CallerKind::ClaudeCode,
        Some(HookEventEvent::ToolAfter),
    );
    assert_eq!(val["decision"], json!("block"));
    assert_eq!(val["reason"], json!("not allowed"));
}

#[test]
fn claude_code_modify() {
    let new_input = json!({"command": "echo safe"});
    let val = serialize_response(
        &HookResponse::modify(new_input.clone()),
        &CallerKind::ClaudeCode,
    );
    assert_eq!(val["decision"], json!("approve"));
    assert_eq!(val["tool_input"], new_input);
}

#[test]
fn unknown_approve() {
    let val = serialize_response(&HookResponse::approve(), &CallerKind::Unknown);
    assert_eq!(val, json!({}));
}

#[test]
fn unknown_block() {
    let val = serialize_response(&HookResponse::block("denied"), &CallerKind::Unknown);
    assert_eq!(val["decision"], json!("block"));
    assert_eq!(val["reason"], json!("denied"));
}

#[test]
fn unknown_modify() {
    let new_input = json!({"x": 1});
    let val = serialize_response(
        &HookResponse::modify(new_input.clone()),
        &CallerKind::Unknown,
    );
    assert_eq!(val["decision"], json!("approve"));
    assert_eq!(val["tool_input"], new_input);
}

#[test]
fn cursor_approve() {
    let val = serialize_response(&HookResponse::approve(), &CallerKind::Cursor);
    assert_eq!(val["action"], json!("allow"));
}

#[test]
fn cursor_block() {
    let val = serialize_response(&HookResponse::block("too risky"), &CallerKind::Cursor);
    assert_eq!(val["action"], json!("deny"));
    assert_eq!(val["message"], json!("too risky"));
}

#[test]
fn cursor_modify() {
    let new_input = json!({"command": "echo safe"});
    let val = serialize_response(
        &HookResponse::modify(new_input.clone()),
        &CallerKind::Cursor,
    );
    assert_eq!(val["action"], json!("modify"));
    assert_eq!(val["args"], new_input);
}

#[test]
fn windsurf_approve() {
    let val = serialize_response(&HookResponse::approve(), &CallerKind::Windsurf);
    assert_eq!(val["allow"], json!(true));
}

#[test]
fn windsurf_block() {
    let val = serialize_response(
        &HookResponse::block("blocked by policy"),
        &CallerKind::Windsurf,
    );
    assert_eq!(val["allow"], json!(false));
    assert_eq!(val["reason"], json!("blocked by policy"));
}

#[test]
fn windsurf_modify() {
    let new_input = json!({"path": "/safe/dir"});
    let val = serialize_response(
        &HookResponse::modify(new_input.clone()),
        &CallerKind::Windsurf,
    );
    assert_eq!(val["allow"], json!(true));
    assert_eq!(val["modified_parameters"], new_input);
}

#[test]
fn cline_approve() {
    let val = serialize_response(&HookResponse::approve(), &CallerKind::Cline);
    assert_eq!(val["approved"], json!(true));
}

#[test]
fn cline_block() {
    let val = serialize_response(&HookResponse::block("cline blocked"), &CallerKind::Cline);
    assert_eq!(val["approved"], json!(false));
    assert_eq!(val["reason"], json!("cline blocked"));
}

#[test]
fn cline_modify() {
    let new_input = json!({"command": "ls /safe"});
    let val = serialize_response(&HookResponse::modify(new_input.clone()), &CallerKind::Cline);
    assert_eq!(val["approved"], json!(true));
    assert_eq!(val["modifiedInput"], new_input);
}

#[test]
fn amp_approve() {
    let val = serialize_response(&HookResponse::approve(), &CallerKind::Amp);
    assert_eq!(val["result"], json!("allow"));
}

#[test]
fn amp_block() {
    let val = serialize_response(&HookResponse::block("amp denied"), &CallerKind::Amp);
    assert_eq!(val["result"], json!("deny"));
    assert_eq!(val["reason"], json!("amp denied"));
}

#[test]
fn amp_modify() {
    let new_input = json!({"command": "echo ok"});
    let val = serialize_response(&HookResponse::modify(new_input.clone()), &CallerKind::Amp);
    assert_eq!(val["result"], json!("allow"));
    assert_eq!(val["modified"], new_input);
}

#[test]
fn gemini_cli_approve() {
    let val = serialize_response(&HookResponse::approve(), &CallerKind::GeminiCli);
    assert_eq!(val["decision"], json!("allow"));
}

#[test]
fn gemini_cli_block() {
    let val = serialize_response(
        &HookResponse::block("secret detected"),
        &CallerKind::GeminiCli,
    );
    assert_eq!(val["decision"], json!("deny"));
    assert_eq!(val["reason"], json!("secret detected"));
}

#[test]
fn gemini_cli_modify() {
    let new_input = json!({"content": "safe content"});
    let val = serialize_response(
        &HookResponse::modify(new_input.clone()),
        &CallerKind::GeminiCli,
    );
    assert_eq!(val["decision"], json!("allow"));
    assert_eq!(val["tool_input"], new_input);
}
