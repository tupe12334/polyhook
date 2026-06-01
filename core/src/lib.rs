pub mod detect;
pub mod events;
pub mod parse;
pub mod response;
pub mod tools;
pub mod types;
mod type_impls;
pub mod wasm;

pub use types::*;

use std::cell::RefCell;
use std::io::Read;

use parse::parse_event;
use response::serialize_response;

// Store the caller from the most recently parsed event so that `respond` can
// serialise the response in the correct format without the caller needing to
// thread the CallerKind through their code.
thread_local! {
    static LAST_CALLER: RefCell<CallerKind> = RefCell::new(CallerKind::Unknown);
}

/// Read a [`HookEvent`] from standard input.
///
/// Blocks until stdin is fully closed (i.e. the invoking agent has written the
/// complete JSON payload and closed its end of the pipe).
pub fn read() -> Result<HookEvent, String> {
    let mut buf = Vec::new();
    std::io::stdin()
        .read_to_end(&mut buf)
        .map_err(|e| format!("stdin read error: {e}"))?;

    let event = parse_event(&buf)?;

    // Persist caller so `respond` can use it.
    LAST_CALLER.with(|c| {
        *c.borrow_mut() = event.caller.clone();
    });

    Ok(event)
}

/// Write a [`HookResponse`] to standard output in the format expected by the
/// agent that was detected during the most recent [`read`] call.
pub fn respond(response: &HookResponse) -> Result<(), String> {
    let caller = LAST_CALLER.with(|c| c.borrow().clone());
    let value = serialize_response(response, &caller);
    let json = serde_json::to_string(&value).map_err(|e| format!("JSON encode error: {e}"))?;

    use std::io::Write;
    std::io::stdout()
        .write_all(json.as_bytes())
        .map_err(|e| format!("stdout write error: {e}"))?;

    Ok(())
}
