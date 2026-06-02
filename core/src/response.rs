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
#[path = "response_tests.rs"]
mod tests;
