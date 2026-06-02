/// WASM ABI layer.
///
/// Memory protocol
/// ---------------
/// All strings cross the WASM boundary as length-prefixed blobs:
///   [4 bytes LE i32 = payload length][payload bytes...]
///
/// Callers must:
///   1. Call `alloc(len)` to get a pointer.
///   2. Write their payload into WASM memory at that pointer.
///   3. Call `parse` or `serialize` with (ptr, len).
///   4. Read the 4-byte LE length from the returned pointer, then the payload.
///   5. Call `dealloc` on both the input buffer and the output buffer.
use std::cell::RefCell;

use crate::parse::parse_event;
use crate::response::serialize_response;
use crate::types::CallerKind;

// Use wee_alloc as the global allocator when targeting WASM to minimise
// binary size.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Store the caller detected during `parse` so that `serialize` can use it
// without requiring callers to pass it explicitly.
thread_local! {
    static LAST_CALLER: RefCell<CallerKind> = const { RefCell::new(CallerKind::Unknown) };
}

// ---------------------------------------------------------------------------
// Memory helpers
// ---------------------------------------------------------------------------

/// Allocate `len` bytes and return a raw pointer.
///
/// The allocation is managed by Rust's allocator; the caller is responsible
/// for calling `dealloc` with the same pointer and length.
///
/// # Safety
///
/// The caller must write exactly `len` bytes before passing the pointer back
/// to `parse` or `serialize`, and must call `dealloc(ptr, len)` exactly once
/// when done.
#[no_mangle]
pub unsafe extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Free memory previously allocated by `alloc` or returned by `parse`/`serialize`.
///
/// # Safety
///
/// `ptr` must have been returned by `alloc`, `parse`, or `serialize`, and
/// `len` must be the exact byte count that was originally allocated.  Must
/// not be called more than once for the same pointer.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    // Reconstruct the Vec so Rust can drop it.
    let _ = Vec::from_raw_parts(ptr, len, len);
}

// ---------------------------------------------------------------------------
// Core exports
// ---------------------------------------------------------------------------

/// Parse raw JSON bytes into a normalised `HookEvent` and return it as
/// length-prefixed JSON.
///
/// Side-effect: stores the detected `CallerKind` in the thread-local so that
/// the subsequent `serialize` call can use it.
///
/// # Safety
///
/// `ptr` must point to `len` consecutive readable bytes valid for the
/// duration of this call.  The returned pointer must be freed with
/// `dealloc(ptr, 4 + payload_len)` where `payload_len` is the LE-i32 at the
/// first four bytes of the returned buffer.
#[no_mangle]
pub unsafe extern "C" fn parse(ptr: *const u8, len: usize) -> *mut u8 {
    let input = std::slice::from_raw_parts(ptr, len);

    let result: Vec<u8> = match parse_event(input) {
        Ok(event) => {
            // Persist the caller for subsequent serialize calls.
            LAST_CALLER.with(|c| {
                *c.borrow_mut() = event.caller;
            });
            match serde_json::to_vec(&event) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let msg = format!("{{\"error\":\"serialize failed: {e}\"}}");
                    msg.into_bytes()
                }
            }
        }
        Err(e) => {
            let msg = format!("{{\"error\":\"{e}\"}}");
            msg.into_bytes()
        }
    };

    length_prefix_alloc(result)
}

/// Deserialise a `HookResponse` JSON and re-serialise it in the format
/// expected by the caller detected during the most recent `parse` call.
///
/// # Safety
///
/// `ptr` must point to `len` consecutive readable bytes valid for the
/// duration of this call.  The returned pointer must be freed with
/// `dealloc(ptr, 4 + payload_len)` where `payload_len` is the LE-i32 at the
/// first four bytes of the returned buffer.
#[no_mangle]
pub unsafe extern "C" fn serialize(ptr: *const u8, len: usize) -> *mut u8 {
    let input = std::slice::from_raw_parts(ptr, len);

    // Parse as a generic Value first, then dispatch on "action" to build a
    // typed HookResponse.  serde untagged deserialization can't safely
    // disambiguate the variants (ApproveResponse matches everything because it
    // has no required-unique fields), so we do it manually.
    let result: Vec<u8> = match serde_json::from_slice::<serde_json::Value>(input) {
        Ok(val) => {
            let resp = match val.get("action").and_then(|a| a.as_str()) {
                Some("block") => {
                    let msg = val
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("");
                    crate::types::HookResponse::block(msg)
                }
                Some("modify") => {
                    let input = val
                        .get("input")
                        .cloned()
                        .unwrap_or(serde_json::Value::Object(Default::default()));
                    crate::types::HookResponse::modify(input)
                }
                _ => crate::types::HookResponse::approve(),
            };
            let caller = LAST_CALLER.with(|c| *c.borrow());
            let value = serialize_response(&resp, &caller);
            match serde_json::to_vec(&value) {
                Ok(bytes) => bytes,
                Err(e) => format!("{{\"error\":\"serialize failed: {e}\"}}").into_bytes(),
            }
        }
        Err(e) => format!("{{\"error\":\"response parse failed: {e}\"}}").into_bytes(),
    };

    length_prefix_alloc(result)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Prepend a 4-byte little-endian length to `payload`, allocate, and return
/// a raw pointer to the combined buffer.  The caller owns the memory and must
/// call `dealloc(ptr, 4 + payload_len)`.
fn length_prefix_alloc(payload: Vec<u8>) -> *mut u8 {
    let payload_len = payload.len();
    let total = 4 + payload_len;

    let mut buf: Vec<u8> = Vec::with_capacity(total);
    let len_bytes = (payload_len as i32).to_le_bytes();
    buf.extend_from_slice(&len_bytes);
    buf.extend_from_slice(&payload);

    debug_assert_eq!(buf.len(), total);

    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: read a length-prefixed buffer returned by parse/serialize.
    // Returns the payload bytes and the total buffer length (4 + payload).
    unsafe fn read_length_prefixed(ptr: *mut u8) -> (Vec<u8>, usize) {
        // Read 4-byte LE length header.
        let len_bytes: [u8; 4] = std::slice::from_raw_parts(ptr, 4)
            .try_into()
            .expect("slice to array");
        let payload_len = i32::from_le_bytes(len_bytes) as usize;
        let total = 4 + payload_len;

        // Copy out the payload.
        let payload = std::slice::from_raw_parts(ptr.add(4), payload_len).to_vec();
        (payload, total)
    }

    // ---------------------------------------------------------------------------
    // alloc / dealloc
    // ---------------------------------------------------------------------------

    #[test]
    fn alloc_zero_does_not_crash() {
        unsafe {
            let ptr = alloc(0);
            // Deallocating a zero-length allocation; ptr may be dangling/null but
            // Vec::from_raw_parts(ptr, 0, 0) is defined to drop nothing.
            dealloc(ptr, 0);
        }
    }

    #[test]
    fn alloc_returns_non_null_for_nonzero_len() {
        unsafe {
            let ptr = alloc(64);
            assert!(!ptr.is_null());
            dealloc(ptr, 64);
        }
    }

    #[test]
    fn dealloc_of_alloc_does_not_crash() {
        unsafe {
            let ptr = alloc(128);
            assert!(!ptr.is_null());
            // Write something to confirm the memory is usable.
            std::ptr::write_bytes(ptr, 0xAB, 128);
            dealloc(ptr, 128);
        }
    }

    // ---------------------------------------------------------------------------
    // parse
    // ---------------------------------------------------------------------------

    #[test]
    fn parse_valid_claude_code_json_returns_tool_before() {
        let json =
            br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"s1"}"#;
        unsafe {
            let input_ptr = alloc(json.len());
            std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

            let out_ptr = parse(input_ptr as *const u8, json.len());
            assert!(!out_ptr.is_null());

            let (payload, total) = read_length_prefixed(out_ptr);
            let s = String::from_utf8(payload).expect("valid utf8");
            assert!(s.contains("tool:before"), "expected 'tool:before' in: {s}");

            dealloc(input_ptr, json.len());
            dealloc(out_ptr, total);
        }
    }

    #[test]
    fn parse_invalid_json_returns_error_payload() {
        let bad = b"this is not json";
        unsafe {
            let input_ptr = alloc(bad.len());
            std::ptr::copy_nonoverlapping(bad.as_ptr(), input_ptr, bad.len());

            let out_ptr = parse(input_ptr as *const u8, bad.len());
            assert!(!out_ptr.is_null());

            let (payload, total) = read_length_prefixed(out_ptr);
            let s = String::from_utf8(payload).expect("valid utf8");
            assert!(s.contains("error"), "expected 'error' in: {s}");

            dealloc(input_ptr, bad.len());
            dealloc(out_ptr, total);
        }
    }

    // ---------------------------------------------------------------------------
    // serialize
    // ---------------------------------------------------------------------------

    #[test]
    fn serialize_approve_returns_length_prefixed_json() {
        let json = br#"{"action":"approve"}"#;
        unsafe {
            let input_ptr = alloc(json.len());
            std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

            let out_ptr = serialize(input_ptr as *const u8, json.len());
            assert!(!out_ptr.is_null());

            let (payload, total) = read_length_prefixed(out_ptr);
            let s = String::from_utf8(payload).expect("valid utf8");
            // Should be valid JSON and not contain "error".
            let parsed: serde_json::Value =
                serde_json::from_str(&s).expect("should be valid JSON");
            assert!(!s.contains("error"), "unexpected error in: {s}");
            // Result is a JSON object.
            assert!(parsed.is_object());

            dealloc(input_ptr, json.len());
            dealloc(out_ptr, total);
        }
    }

    #[test]
    fn serialize_block_returns_length_prefixed_json_with_block_content() {
        // First parse something so LAST_CALLER is set (ClaudeCode).
        let event_json =
            br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"s2"}"#;
        unsafe {
            let ep = alloc(event_json.len());
            std::ptr::copy_nonoverlapping(event_json.as_ptr(), ep, event_json.len());
            let ep_out = parse(ep as *const u8, event_json.len());
            let (_, ep_total) = read_length_prefixed(ep_out);
            dealloc(ep, event_json.len());
            dealloc(ep_out, ep_total);
        }

        let json = br#"{"action":"block","message":"dangerous command"}"#;
        unsafe {
            let input_ptr = alloc(json.len());
            std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

            let out_ptr = serialize(input_ptr as *const u8, json.len());
            assert!(!out_ptr.is_null());

            let (payload, total) = read_length_prefixed(out_ptr);
            let s = String::from_utf8(payload).expect("valid utf8");
            // Claude Code block format should have "block" in the JSON.
            assert!(s.contains("block"), "expected 'block' in: {s}");

            dealloc(input_ptr, json.len());
            dealloc(out_ptr, total);
        }
    }

    #[test]
    fn serialize_modify_returns_length_prefixed_json() {
        // Ensure LAST_CALLER is set via parse.
        let event_json =
            br#"{"type":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"session_id":"s3"}"#;
        unsafe {
            let ep = alloc(event_json.len());
            std::ptr::copy_nonoverlapping(event_json.as_ptr(), ep, event_json.len());
            let ep_out = parse(ep as *const u8, event_json.len());
            let (_, ep_total) = read_length_prefixed(ep_out);
            dealloc(ep, event_json.len());
            dealloc(ep_out, ep_total);
        }

        let json = br#"{"action":"modify","input":{"command":"echo safe"}}"#;
        unsafe {
            let input_ptr = alloc(json.len());
            std::ptr::copy_nonoverlapping(json.as_ptr(), input_ptr, json.len());

            let out_ptr = serialize(input_ptr as *const u8, json.len());
            assert!(!out_ptr.is_null());

            let (payload, total) = read_length_prefixed(out_ptr);
            let s = String::from_utf8(payload).expect("valid utf8");
            let parsed: serde_json::Value =
                serde_json::from_str(&s).expect("should be valid JSON");
            assert!(parsed.is_object());
            assert!(!s.contains("error"), "unexpected error in: {s}");

            dealloc(input_ptr, json.len());
            dealloc(out_ptr, total);
        }
    }

    #[test]
    fn serialize_invalid_json_returns_error_payload() {
        let bad = b"not json at all!";
        unsafe {
            let input_ptr = alloc(bad.len());
            std::ptr::copy_nonoverlapping(bad.as_ptr(), input_ptr, bad.len());

            let out_ptr = serialize(input_ptr as *const u8, bad.len());
            assert!(!out_ptr.is_null());

            let (payload, total) = read_length_prefixed(out_ptr);
            let s = String::from_utf8(payload).expect("valid utf8");
            assert!(s.contains("error"), "expected 'error' in: {s}");

            dealloc(input_ptr, bad.len());
            dealloc(out_ptr, total);
        }
    }

    // ---------------------------------------------------------------------------
    // length_prefix_alloc (private helper exercised indirectly above, but also
    // tested directly via the public API round-trip)
    // ---------------------------------------------------------------------------

    #[test]
    fn length_prefix_alloc_encodes_correct_length() {
        let payload = b"hello world";
        let payload_vec = payload.to_vec();
        let payload_len = payload_vec.len();

        unsafe {
            let ptr = length_prefix_alloc(payload_vec);
            assert!(!ptr.is_null());

            // Read back the 4-byte LE header.
            let len_bytes: [u8; 4] =
                std::slice::from_raw_parts(ptr, 4).try_into().unwrap();
            let decoded_len = i32::from_le_bytes(len_bytes) as usize;
            assert_eq!(decoded_len, payload_len);

            // Verify payload bytes.
            let actual_payload = std::slice::from_raw_parts(ptr.add(4), payload_len);
            assert_eq!(actual_payload, payload);

            dealloc(ptr, 4 + payload_len);
        }
    }
}
