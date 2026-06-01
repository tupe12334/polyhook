use crate::types::{ApproveResponse, BlockResponse, CallerKind, HookResponse, ModifyResponse};

impl Default for CallerKind {
    fn default() -> Self {
        CallerKind::Unknown
    }
}

impl HookResponse {
    pub fn approve() -> Self {
        HookResponse::ApproveResponse(ApproveResponse {
            action: "approve".to_string(),
        })
    }

    pub fn block(msg: &str) -> Self {
        HookResponse::BlockResponse(BlockResponse {
            action: "block".to_string(),
            message: msg.to_owned(),
        })
    }

    /// `input` must be a JSON object; non-object values are silently treated as empty.
    pub fn modify(input: serde_json::Value) -> Self {
        HookResponse::ModifyResponse(ModifyResponse {
            action: "modify".to_string(),
            input: input.as_object().cloned().unwrap_or_default(),
        })
    }
}
