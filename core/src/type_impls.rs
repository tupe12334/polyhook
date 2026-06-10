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
    use serde_json::json;

    #[test]
    fn default_caller_kind_is_unknown() {
        assert_eq!(CallerKind::default(), CallerKind::Unknown);
    }

    #[test]
    fn modify_with_object_preserves_fields() {
        let value = serde_json::to_value(HookResponse::modify(json!({"key": "value", "num": 42})))
            .expect("modify response should serialize");

        assert_eq!(
            value,
            json!({"action": "modify", "input": {"key": "value", "num": 42}})
        );
    }

    #[test]
    fn modify_with_string_produces_empty_input() {
        let value = serde_json::to_value(HookResponse::modify(serde_json::Value::String(
            "not an object".into(),
        )))
        .expect("modify response should serialize");

        assert_eq!(value, json!({"action": "modify", "input": {}}));
    }

    #[test]
    fn modify_with_array_produces_empty_input() {
        let value = serde_json::to_value(HookResponse::modify(json!([1, 2, 3])))
            .expect("modify response should serialize");

        assert_eq!(value, json!({"action": "modify", "input": {}}));
    }

    #[test]
    fn modify_with_null_produces_empty_input() {
        let value = serde_json::to_value(HookResponse::modify(serde_json::Value::Null))
            .expect("modify response should serialize");

        assert_eq!(value, json!({"action": "modify", "input": {}}));
    }
}
