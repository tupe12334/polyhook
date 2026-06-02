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
#[path = "lib_tests.rs"]
mod tests;
