use polyhook::{read, respond, HookResponse};

fn main() {
    let event = read().unwrap_or_else(|e| {
        eprintln!("polyhook: {e}");
        std::process::exit(1);
    });

    if let Some(tool) = &event.tool {
        eprintln!(
            "[hook] caller={} event={} tool={}",
            serde_json::to_string(&event.caller).unwrap_or_default().trim_matches('"'),
            event.event,
            tool
        );
    }

    respond(&HookResponse::approve()).unwrap();
}
