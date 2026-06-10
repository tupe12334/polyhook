use super::*;
use std::io::Cursor;

const CLAUDE_PRE_TOOL: &str = r#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la"},"session_id":"sess_test_001"}"#;
const CURSOR_BEFORE_TOOL: &str = r#"{"type":"BeforeToolCall","toolCall":{"name":"run_terminal_cmd","args":{"command":"echo hi"}},"sessionId":"sess_test_002"}"#;

#[test]
fn read_from_parses_claude_code_event() {
    let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
    let event = read_from(&mut cursor).expect("read_from should succeed");
    assert_eq!(event.caller, CallerKind::ClaudeCode);
    assert_eq!(event.event.to_string(), "tool:before");
    assert_eq!(event.tool.as_deref(), Some("bash"));
    assert_eq!(event.session_id, "sess_test_001");
}

#[test]
fn read_from_returns_error_on_invalid_json() {
    let mut cursor = Cursor::new(b"not valid json" as &[u8]);
    let result = read_from(&mut cursor);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("JSON parse error") || msg.contains("parse"));
}

#[test]
fn respond_to_writes_json_to_writer() {
    let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
    let _ = read_from(&mut cursor).expect("read_from should succeed");

    let mut output: Vec<u8> = Vec::new();
    respond_to(&mut output, &HookResponse::approve()).expect("respond_to should succeed");

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");
    assert!(json.as_object().unwrap().is_empty());
}

#[test]
fn respond_to_block_uses_detected_caller() {
    let mut cursor = Cursor::new(CURSOR_BEFORE_TOOL.as_bytes());
    let _ = read_from(&mut cursor).expect("read_from should succeed");

    let mut output: Vec<u8> = Vec::new();
    respond_to(&mut output, &HookResponse::block("stop")).expect("respond_to should succeed");

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");
    assert_eq!(json["action"], "deny");
    assert_eq!(json["message"], "stop");
}

#[test]
fn respond_to_claude_pre_tool_use_block_uses_hook_specific_output() {
    let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
    let _ = read_from(&mut cursor).expect("read_from should succeed");

    let mut output: Vec<u8> = Vec::new();
    respond_to(&mut output, &HookResponse::block("blocked")).expect("respond_to should succeed");

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");
    assert_eq!(json["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(json["hookSpecificOutput"]["permissionDecision"], "deny");
    assert_eq!(
        json["hookSpecificOutput"]["permissionDecisionReason"],
        "blocked"
    );
    assert_eq!(
        json["hookSpecificOutput"]["additionalContext"],
        serde_json::Value::Null
    );
}

#[test]
fn respond_to_modify_uses_detected_caller() {
    let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
    let _ = read_from(&mut cursor).expect("read_from should succeed");

    let new_input = serde_json::json!({"command": "echo safe"});
    let mut output: Vec<u8> = Vec::new();
    respond_to(&mut output, &HookResponse::modify(new_input.clone()))
        .expect("respond_to should succeed");

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");
    assert_eq!(json["decision"], "approve");
    assert_eq!(json["tool_input"], new_input);
}

#[test]
fn last_caller_thread_local_is_updated_by_read_from() {
    let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
    let event = read_from(&mut cursor).expect("read_from should succeed");
    assert_eq!(event.caller, CallerKind::ClaudeCode);

    let mut output: Vec<u8> = Vec::new();
    respond_to(&mut output, &HookResponse::approve()).expect("respond_to should succeed");
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert!(json.as_object().unwrap().is_empty());

    let mut cursor2 = Cursor::new(CURSOR_BEFORE_TOOL.as_bytes());
    let event2 = read_from(&mut cursor2).expect("read_from should succeed");
    assert_eq!(event2.caller, CallerKind::Cursor);

    let mut output2: Vec<u8> = Vec::new();
    respond_to(&mut output2, &HookResponse::approve()).expect("respond_to should succeed");
    let json2: serde_json::Value = serde_json::from_slice(&output2).unwrap();
    assert_eq!(json2["action"], "allow");
}

#[test]
fn respond_delegates_to_stdout() {
    let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
    let _ = read_from(&mut cursor).expect("prime");
    let result = respond(&HookResponse::approve());
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn read_delegates_to_stdin() {
    extern "C" {
        fn pipe(fds: *mut i32) -> i32;
        fn dup(fd: i32) -> i32;
        fn dup2(oldfd: i32, newfd: i32) -> i32;
        fn close(fd: i32) -> i32;
        fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    }

    let json = CLAUDE_PRE_TOOL.as_bytes();

    unsafe {
        let mut fds = [0i32; 2];
        assert_eq!(pipe(fds.as_mut_ptr()), 0);
        let (read_fd, write_fd) = (fds[0], fds[1]);

        write(write_fd, json.as_ptr(), json.len());
        close(write_fd);

        let saved = dup(0);
        dup2(read_fd, 0);
        close(read_fd);

        let result = read();

        dup2(saved, 0);
        close(saved);

        result.expect("read() should succeed");
    }
}

#[test]
fn read_from_io_error_returns_err() {
    struct FailReader;
    impl std::io::Read for FailReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "pipe broken",
            ))
        }
    }
    let result = read_from(&mut FailReader);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("read error"));
}

#[test]
fn respond_to_write_error_returns_err() {
    struct FailWriter;
    impl std::io::Write for FailWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "pipe broken",
            ))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(FailWriter.flush().is_ok());
    let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
    let _ = read_from(&mut cursor).expect("prime");
    let result = respond_to(&mut FailWriter, &HookResponse::approve());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("write error"));
}
