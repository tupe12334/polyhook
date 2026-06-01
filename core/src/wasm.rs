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
    static LAST_CALLER: RefCell<CallerKind> = RefCell::new(CallerKind::Unknown);
}

// ---------------------------------------------------------------------------
// Memory helpers
// ---------------------------------------------------------------------------

/// Allocate `len` bytes and return a raw pointer.
///
/// The allocation is managed by Rust's allocator; the caller is responsible
/// for calling `dealloc` with the same pointer and length.
#[no_mangle]
pub unsafe extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(len);
    // Safety: we immediately hand ownership to the caller; they will write
    // into this memory before passing it back.
    buf.set_len(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Free memory previously allocated by `alloc` or returned by `parse`/`serialize`.
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
#[no_mangle]
pub unsafe extern "C" fn parse(ptr: *const u8, len: usize) -> *mut u8 {
    let input = std::slice::from_raw_parts(ptr, len);

    let result: Vec<u8> = match parse_event(input) {
        Ok(event) => {
            // Persist the caller for subsequent serialize calls.
            LAST_CALLER.with(|c| {
                *c.borrow_mut() = event.caller.clone();
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
            let caller = LAST_CALLER.with(|c| c.borrow().clone());
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
