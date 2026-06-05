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
use response::serialize_response_with_event;

// Store the caller and event from the most recently parsed event so that
// `respond` can serialise the response in the correct format.
thread_local! {
    static LAST_CALLER: RefCell<CallerKind> = const { RefCell::new(CallerKind::Unknown) };
    static LAST_EVENT: RefCell<Option<HookEventEvent>> = const { RefCell::new(None) };
}

/// Read a [`HookEvent`] from an arbitrary reader.
///
/// Reads until EOF, then parses the JSON payload. The detected [`CallerKind`]
/// and event type are stored in thread-locals so that a subsequent [`respond_to`]
/// call can serialise the response in the correct format.
pub fn read_from(r: &mut impl Read) -> Result<HookEvent, String> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)
        .map_err(|e| format!("read error: {e}"))?;

    let event = parse_event(&buf)?;

    LAST_CALLER.with(|c| {
        *c.borrow_mut() = event.caller;
    });
    LAST_EVENT.with(|e| {
        *e.borrow_mut() = Some(event.event);
    });

    Ok(event)
}

/// Write a [`HookResponse`] to an arbitrary writer in the format expected by
/// the agent that was detected during the most recent [`read_from`] call.
pub fn respond_to(w: &mut impl Write, response: &HookResponse) -> Result<(), String> {
    let caller = LAST_CALLER.with(|c| *c.borrow());
    let event = LAST_EVENT.with(|e| *e.borrow());
    let value = serialize_response_with_event(response, &caller, event);
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
#[path = "lib_tests.rs"]
mod tests;
