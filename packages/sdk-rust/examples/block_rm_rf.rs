use polyhook::{read, respond, HookResponse};

fn main() {
    let event = match read() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("polyhook: failed to read event: {e}");
            std::process::exit(1);
        }
    };

    let response = match event.tool.as_deref() {
        Some("bash") => {
            let cmd = event
                .input
                .as_ref()
                .and_then(|i| i.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if cmd.contains("rm -rf /") {
                HookResponse::block("Refusing to delete from root")
            } else {
                HookResponse::approve()
            }
        }
        _ => HookResponse::approve(),
    };

    if let Err(e) = respond(&response) {
        eprintln!("polyhook: failed to write response: {e}");
        std::process::exit(1);
    }
}
