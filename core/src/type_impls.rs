use crate::types::{ApproveResponse, BlockResponse, CallerKind, HookResponse, ModifyResponse};

#[allow(clippy::derivable_impls)] // CallerKind is generated; Unknown is not the first variant
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::{CallerKind, HookResponse};

    #[test]
    fn default_caller_kind_is_unknown() {
        assert_eq!(CallerKind::default(), CallerKind::Unknown);
    }

    #[test]
    fn modify_with_object_preserves_fields() {
        let obj = serde_json::json!({"key": "value", "num": 42});
        let HookResponse::ModifyResponse(m) = HookResponse::modify(obj) else { unreachable!() };
        assert_eq!(m.input.len(), 2);
        assert_eq!(m.input["key"], "value");
        assert_eq!(m.input["num"], 42);
    }

    #[test]
    fn modify_with_string_produces_empty_input() {
        let HookResponse::ModifyResponse(m) = HookResponse::modify(serde_json::Value::String("not an object".into())) else { unreachable!() };
        assert!(m.input.is_empty());
    }

    #[test]
    fn modify_with_array_produces_empty_input() {
        let HookResponse::ModifyResponse(m) = HookResponse::modify(serde_json::json!([1, 2, 3])) else { unreachable!() };
        assert!(m.input.is_empty());
    }

    #[test]
    fn modify_with_null_produces_empty_input() {
        let HookResponse::ModifyResponse(m) = HookResponse::modify(serde_json::Value::Null) else { unreachable!() };
        assert!(m.input.is_empty());
    }
}
