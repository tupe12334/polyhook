use crate::{read_from, respond_to, HookEvent, HookResponse};

pub fn read() -> Result<HookEvent, String> {
    read_from(&mut std::io::stdin())
}

pub fn respond(response: &HookResponse) -> Result<(), String> {
    respond_to(&mut std::io::stdout(), response)
}
