use crate::types::CallerKind;

/// Normalize a vendor-specific tool name to the canonical polyhook tool name.
///
/// The lookup is case-insensitive. If no mapping exists the original name is
/// returned unchanged.
pub fn normalize_tool(vendor: &str, caller: &CallerKind) -> String {
    let lower = vendor.to_lowercase();

    let canonical = match caller {
        CallerKind::ClaudeCode => normalize_claude_code(&lower),
        CallerKind::Cursor => normalize_cursor(&lower),
        CallerKind::Windsurf => normalize_windsurf(&lower),
        CallerKind::Cline => normalize_cline(&lower),
        CallerKind::Amp => normalize_amp(&lower),
        CallerKind::Unknown => None,
    };

    canonical
        .map(|s| s.to_owned())
        .unwrap_or_else(|| vendor.to_owned())
}

fn normalize_claude_code(lower: &str) -> Option<&'static str> {
    match lower {
        "bash" => Some("bash"),
        "read" => Some("read_file"),
        "write" => Some("write_file"),
        "edit" => Some("edit_file"),
        "ls" => Some("list_dir"),
        "grep" => Some("grep"),
        "glob" => Some("glob"),
        "websearch" => Some("web_search"),
        "webfetch" => Some("web_fetch"),
        "task" => Some("spawn_agent"),
        "mcp__ide__getdiagnostics" => Some("diagnostics"),
        _ => None,
    }
}

fn normalize_cursor(lower: &str) -> Option<&'static str> {
    match lower {
        "run_terminal_cmd" => Some("bash"),
        "read_file" => Some("read_file"),
        "edit_file" => Some("write_file"),
        "apply_edit" => Some("edit_file"),
        "list_dir" => Some("list_dir"),
        "grep_search" => Some("grep"),
        "file_search" => Some("glob"),
        "web_search" => Some("web_search"),
        "fetch_url" => Some("web_fetch"),
        "spawn_agent" => Some("spawn_agent"),
        "get_diagnostics" => Some("diagnostics"),
        "move_file" => Some("move_file"),
        "delete_file" => Some("delete_file"),
        "create_dir" => Some("create_dir"),
        _ => None,
    }
}

fn normalize_windsurf(lower: &str) -> Option<&'static str> {
    match lower {
        "run_command" => Some("bash"),
        "read_file" => Some("read_file"),
        "write_file" => Some("write_file"),
        "edit_file" => Some("edit_file"),
        "list_directory" => Some("list_dir"),
        "search_files" => Some("grep"),
        "find_files" => Some("glob"),
        "search_web" => Some("web_search"),
        "fetch_page" => Some("web_fetch"),
        "spawn_agent" => Some("spawn_agent"),
        "get_diagnostics" => Some("diagnostics"),
        "move_file" => Some("move_file"),
        "delete_file" => Some("delete_file"),
        "create_directory" => Some("create_dir"),
        _ => None,
    }
}

fn normalize_cline(lower: &str) -> Option<&'static str> {
    match lower {
        "execute_command" => Some("bash"),
        "read_file" => Some("read_file"),
        "write_to_file" => Some("write_file"),
        "apply_diff" => Some("edit_file"),
        "list_files" => Some("list_dir"),
        "search_files" => Some("grep"),
        "search" => Some("web_search"),
        "fetch" => Some("web_fetch"),
        "rename_file" => Some("move_file"),
        "delete_file" => Some("delete_file"),
        "create_directory" => Some("create_dir"),
        "get_diagnostics" => Some("diagnostics"),
        _ => None,
    }
}

fn normalize_amp(lower: &str) -> Option<&'static str> {
    match lower {
        "shell" => Some("bash"),
        "file.read" => Some("read_file"),
        "file.write" => Some("write_file"),
        "file.edit" => Some("edit_file"),
        "fs.list" => Some("list_dir"),
        "search.grep" => Some("grep"),
        "search.glob" => Some("glob"),
        "web.search" => Some("web_search"),
        "web.fetch" => Some("web_fetch"),
        "agent.spawn" => Some("spawn_agent"),
        "lsp.diagnostics" => Some("diagnostics"),
        "fs.move" => Some("move_file"),
        "fs.delete" => Some("delete_file"),
        "fs.mkdir" => Some("create_dir"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "tools_tests.rs"]
mod tests;
