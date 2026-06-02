use crate::types::{CallerKind, HookResponse};
use serde_json::{json, Value};

/// Serialize a [`HookResponse`] into the JSON format expected by the detected caller.
pub fn serialize_response(resp: &HookResponse, caller: &CallerKind) -> Value {
    match caller {
        CallerKind::ClaudeCode | CallerKind::Unknown => serialize_claude_code(resp),
        CallerKind::Cursor => serialize_cursor(resp),
        CallerKind::Windsurf => serialize_windsurf(resp),
        CallerKind::Cline => serialize_cline(resp),
        CallerKind::Amp => serialize_amp(resp),
    }
}

// ---------------------------------------------------------------------------
// Per-caller serializers
// ---------------------------------------------------------------------------

fn serialize_claude_code(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({}),
        HookResponse::BlockResponse(b) => {
            json!({ "decision": "block", "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "decision": "approve", "tool_input": m.input })
        }
    }
}

fn serialize_cursor(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "action": "allow" }),
        HookResponse::BlockResponse(b) => {
            json!({ "action": "deny", "message": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "action": "modify", "args": m.input })
        }
    }
}

fn serialize_windsurf(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "allow": true }),
        HookResponse::BlockResponse(b) => {
            json!({ "allow": false, "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "allow": true, "modified_parameters": m.input })
        }
    }
}

fn serialize_cline(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "approved": true }),
        HookResponse::BlockResponse(b) => {
            json!({ "approved": false, "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "approved": true, "modifiedInput": m.input })
        }
    }
}

fn serialize_amp(resp: &HookResponse) -> Value {
    match resp {
        HookResponse::ApproveResponse(_) => json!({ "result": "allow" }),
        HookResponse::BlockResponse(b) => {
            json!({ "result": "deny", "reason": b.message })
        }
        HookResponse::ModifyResponse(m) => {
            json!({ "result": "allow", "modified": m.input })
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
        let val = serialize_response(&HookResponse::block("not allowed"), &CallerKind::ClaudeCode);
        assert_eq!(val["decision"], json!("block"));
        assert_eq!(val["reason"], json!("not allowed"));
    }

    #[test]
    fn claude_code_modify() {
        let new_input = json!({"command": "echo safe"});
        let val = serialize_response(&HookResponse::modify(new_input.clone()), &CallerKind::ClaudeCode);
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
        let val = serialize_response(&HookResponse::modify(new_input.clone()), &CallerKind::Unknown);
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
        let val = serialize_response(&HookResponse::modify(new_input.clone()), &CallerKind::Cursor);
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
        let val = serialize_response(&HookResponse::block("blocked by policy"), &CallerKind::Windsurf);
        assert_eq!(val["allow"], json!(false));
        assert_eq!(val["reason"], json!("blocked by policy"));
    }

    #[test]
    fn windsurf_modify() {
        let new_input = json!({"path": "/safe/dir"});
        let val = serialize_response(&HookResponse::modify(new_input.clone()), &CallerKind::Windsurf);
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
}
