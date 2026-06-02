pub mod detect;
pub mod events;
pub mod parse;
pub mod response;
pub mod tools;
pub mod types;
mod type_impls;
pub mod wasm;
mod stdin_wrappers;

pub use types::*;
pub use stdin_wrappers::{read, respond};

use std::cell::RefCell;
use std::io::{Read, Write};

use parse::parse_event;
use response::serialize_response;

// Store the caller from the most recently parsed event so that `respond` can
// serialise the response in the correct format without the caller needing to
// thread the CallerKind through their code.
thread_local! {
    static LAST_CALLER: RefCell<CallerKind> = RefCell::new(CallerKind::Unknown);
}

/// Read a [`HookEvent`] from an arbitrary reader.
///
/// Reads until EOF, then parses the JSON payload.  The detected [`CallerKind`]
/// is stored in a thread-local so that a subsequent [`respond_to`] call can
/// serialise the response in the correct format.
pub fn read_from(r: &mut impl Read) -> Result<HookEvent, String> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)
        .map_err(|e| format!("read error: {e}"))?;

    let event = parse_event(&buf)?;

    // Persist caller so `respond_to` / `respond` can use it.
    LAST_CALLER.with(|c| {
        *c.borrow_mut() = event.caller.clone();
    });

    Ok(event)
}

/// Write a [`HookResponse`] to an arbitrary writer in the format expected by
/// the agent that was detected during the most recent [`read_from`] call.
pub fn respond_to(w: &mut impl Write, response: &HookResponse) -> Result<(), String> {
    let caller = LAST_CALLER.with(|c| c.borrow().clone());
    let value = serialize_response(response, &caller);
    // serde_json::Value is always serializable; expect is safe here.
    let json = serde_json::to_string(&value).expect("serde_json::Value is always serializable");

    w.write_all(json.as_bytes())
        .map_err(|e| format!("write error: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
        // First set the LAST_CALLER via read_from so respond_to uses ClaudeCode format.
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("read_from should succeed");

        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::approve()).expect("respond_to should succeed");

        let json: serde_json::Value =
            serde_json::from_slice(&output).expect("output should be valid JSON");
        // ClaudeCode approve → empty object
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn respond_to_block_uses_detected_caller() {
        // Parse a Cursor event so LAST_CALLER becomes Cursor.
        let mut cursor = Cursor::new(CURSOR_BEFORE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("read_from should succeed");

        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::block("stop")).expect("respond_to should succeed");

        let json: serde_json::Value =
            serde_json::from_slice(&output).expect("output should be valid JSON");
        // Cursor block → {"action": "deny", "message": "..."}
        assert_eq!(json["action"], "deny");
        assert_eq!(json["message"], "stop");
    }

    #[test]
    fn respond_to_modify_uses_detected_caller() {
        // Parse a Claude Code event.
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("read_from should succeed");

        let new_input = serde_json::json!({"command": "echo safe"});
        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::modify(new_input.clone()))
            .expect("respond_to should succeed");

        let json: serde_json::Value =
            serde_json::from_slice(&output).expect("output should be valid JSON");
        // ClaudeCode modify → {"decision": "approve", "tool_input": {...}}
        assert_eq!(json["decision"], "approve");
        assert_eq!(json["tool_input"], new_input);
    }

    #[test]
    fn last_caller_thread_local_is_updated_by_read_from() {
        // Parse ClaudeCode event → LAST_CALLER should be ClaudeCode.
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let event = read_from(&mut cursor).expect("read_from should succeed");
        assert_eq!(event.caller, CallerKind::ClaudeCode);

        // respond_to should use ClaudeCode format.
        let mut output: Vec<u8> = Vec::new();
        respond_to(&mut output, &HookResponse::approve()).expect("respond_to should succeed");
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert!(json.as_object().unwrap().is_empty());

        // Now parse Cursor event → LAST_CALLER should switch to Cursor.
        let mut cursor2 = Cursor::new(CURSOR_BEFORE_TOOL.as_bytes());
        let event2 = read_from(&mut cursor2).expect("read_from should succeed");
        assert_eq!(event2.caller, CallerKind::Cursor);

        let mut output2: Vec<u8> = Vec::new();
        respond_to(&mut output2, &HookResponse::approve()).expect("respond_to should succeed");
        let json2: serde_json::Value = serde_json::from_slice(&output2).unwrap();
        // Cursor approve → {"action": "allow"}
        assert_eq!(json2["action"], "allow");
    }

    // Test the respond() thin wrapper (writes to stdout — captured by test harness).
    #[test]
    fn respond_delegates_to_stdout() {
        // Prime LAST_CALLER.
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("prime");
        // respond() writes to stdout; in tests the harness captures it.
        let result = respond(&HookResponse::approve());
        assert!(result.is_ok());
    }

    // Test the read() thin wrapper via an OS pipe (Unix only).
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

    // Cover the error branch in read_from where the reader fails.
    #[test]
    fn read_from_io_error_returns_err() {
        struct FailReader;
        impl std::io::Read for FailReader {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken"))
            }
        }
        let result = read_from(&mut FailReader);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("read error"));
    }

    // Cover the error branch in respond_to where the writer fails.
    #[test]
    fn respond_to_write_error_returns_err() {
        struct FailWriter;
        impl std::io::Write for FailWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken"))
            }
            fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
        }
        assert!(FailWriter.flush().is_ok());
        let mut cursor = Cursor::new(CLAUDE_PRE_TOOL.as_bytes());
        let _ = read_from(&mut cursor).expect("prime");
        let result = respond_to(&mut FailWriter, &HookResponse::approve());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("write error"));
    }
}
