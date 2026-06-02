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
mod tests {
    use super::normalize_tool;
    use crate::CallerKind;

    #[test]
    fn unknown_caller_returns_original() {
        assert_eq!(normalize_tool("SomeTool", &CallerKind::Unknown), "SomeTool");
        assert_eq!(normalize_tool("bash", &CallerKind::Unknown), "bash");
        assert_eq!(normalize_tool("Bash", &CallerKind::Unknown), "Bash");
    }

    // ClaudeCode extra mappings
    #[test] fn claude_ls()          { assert_eq!(normalize_tool("ls",                         &CallerKind::ClaudeCode), "list_dir"); }
    #[test] fn claude_grep()        { assert_eq!(normalize_tool("grep",                       &CallerKind::ClaudeCode), "grep"); }
    #[test] fn claude_glob()        { assert_eq!(normalize_tool("glob",                       &CallerKind::ClaudeCode), "glob"); }
    #[test] fn claude_websearch()   { assert_eq!(normalize_tool("websearch",                  &CallerKind::ClaudeCode), "web_search"); }
    #[test] fn claude_webfetch()    { assert_eq!(normalize_tool("webfetch",                   &CallerKind::ClaudeCode), "web_fetch"); }
    #[test] fn claude_mcp_diag()    { assert_eq!(normalize_tool("mcp__ide__getdiagnostics",   &CallerKind::ClaudeCode), "diagnostics"); }

    // Cursor extra mappings
    #[test] fn cursor_apply_edit()     { assert_eq!(normalize_tool("apply_edit",     &CallerKind::Cursor), "edit_file"); }
    #[test] fn cursor_list_dir()       { assert_eq!(normalize_tool("list_dir",       &CallerKind::Cursor), "list_dir"); }
    #[test] fn cursor_file_search()    { assert_eq!(normalize_tool("file_search",    &CallerKind::Cursor), "glob"); }
    #[test] fn cursor_fetch_url()      { assert_eq!(normalize_tool("fetch_url",      &CallerKind::Cursor), "web_fetch"); }
    #[test] fn cursor_spawn_agent()    { assert_eq!(normalize_tool("spawn_agent",    &CallerKind::Cursor), "spawn_agent"); }
    #[test] fn cursor_get_diag()       { assert_eq!(normalize_tool("get_diagnostics",&CallerKind::Cursor), "diagnostics"); }
    #[test] fn cursor_move_file()      { assert_eq!(normalize_tool("move_file",      &CallerKind::Cursor), "move_file"); }
    #[test] fn cursor_delete_file()    { assert_eq!(normalize_tool("delete_file",    &CallerKind::Cursor), "delete_file"); }
    #[test] fn cursor_create_dir()     { assert_eq!(normalize_tool("create_dir",     &CallerKind::Cursor), "create_dir"); }

    // Windsurf extra mappings
    #[test] fn windsurf_edit_file()    { assert_eq!(normalize_tool("edit_file",       &CallerKind::Windsurf), "edit_file"); }
    #[test] fn windsurf_search_files() { assert_eq!(normalize_tool("search_files",    &CallerKind::Windsurf), "grep"); }
    #[test] fn windsurf_find_files()   { assert_eq!(normalize_tool("find_files",      &CallerKind::Windsurf), "glob"); }
    #[test] fn windsurf_fetch_page()   { assert_eq!(normalize_tool("fetch_page",      &CallerKind::Windsurf), "web_fetch"); }
    #[test] fn windsurf_spawn_agent()  { assert_eq!(normalize_tool("spawn_agent",     &CallerKind::Windsurf), "spawn_agent"); }
    #[test] fn windsurf_get_diag()     { assert_eq!(normalize_tool("get_diagnostics", &CallerKind::Windsurf), "diagnostics"); }
    #[test] fn windsurf_move_file()    { assert_eq!(normalize_tool("move_file",       &CallerKind::Windsurf), "move_file"); }
    #[test] fn windsurf_delete_file()  { assert_eq!(normalize_tool("delete_file",     &CallerKind::Windsurf), "delete_file"); }
    #[test] fn windsurf_create_dir()   { assert_eq!(normalize_tool("create_directory",&CallerKind::Windsurf), "create_dir"); }

    // Cline extra mappings
    #[test] fn cline_search_files(){ assert_eq!(normalize_tool("search_files",      &CallerKind::Cline), "grep"); }
    #[test] fn cline_search()      { assert_eq!(normalize_tool("search",           &CallerKind::Cline), "web_search"); }
    #[test] fn cline_fetch()       { assert_eq!(normalize_tool("fetch",            &CallerKind::Cline), "web_fetch"); }
    #[test] fn cline_rename_file() { assert_eq!(normalize_tool("rename_file",      &CallerKind::Cline), "move_file"); }
    #[test] fn cline_delete_file() { assert_eq!(normalize_tool("delete_file",      &CallerKind::Cline), "delete_file"); }
    #[test] fn cline_create_dir()  { assert_eq!(normalize_tool("create_directory", &CallerKind::Cline), "create_dir"); }
    #[test] fn cline_get_diag()    { assert_eq!(normalize_tool("get_diagnostics",  &CallerKind::Cline), "diagnostics"); }

    // Amp extra mappings
    #[test] fn amp_search_grep()   { assert_eq!(normalize_tool("search.grep",    &CallerKind::Amp), "grep"); }
    #[test] fn amp_search_glob()   { assert_eq!(normalize_tool("search.glob",    &CallerKind::Amp), "glob"); }
    #[test] fn amp_web_fetch()     { assert_eq!(normalize_tool("web.fetch",      &CallerKind::Amp), "web_fetch"); }
    #[test] fn amp_agent_spawn()   { assert_eq!(normalize_tool("agent.spawn",    &CallerKind::Amp), "spawn_agent"); }
    #[test] fn amp_lsp_diag()      { assert_eq!(normalize_tool("lsp.diagnostics",&CallerKind::Amp), "diagnostics"); }
    #[test] fn amp_fs_move()       { assert_eq!(normalize_tool("fs.move",        &CallerKind::Amp), "move_file"); }
    #[test] fn amp_fs_delete()     { assert_eq!(normalize_tool("fs.delete",      &CallerKind::Amp), "delete_file"); }
    #[test] fn amp_fs_mkdir()      { assert_eq!(normalize_tool("fs.mkdir",       &CallerKind::Amp), "create_dir"); }

    // Fallthrough: unmapped tool names pass through unchanged for each known caller
    #[test] fn claude_unknown_falls_through()   { assert_eq!(normalize_tool("no_such_tool", &CallerKind::ClaudeCode), "no_such_tool"); }
    #[test] fn cursor_unknown_falls_through()   { assert_eq!(normalize_tool("no_such_tool", &CallerKind::Cursor),    "no_such_tool"); }
    #[test] fn windsurf_unknown_falls_through() { assert_eq!(normalize_tool("no_such_tool", &CallerKind::Windsurf),  "no_such_tool"); }
    #[test] fn cline_unknown_falls_through()    { assert_eq!(normalize_tool("no_such_tool", &CallerKind::Cline),     "no_such_tool"); }
    #[test] fn amp_unknown_falls_through()      { assert_eq!(normalize_tool("no_such_tool", &CallerKind::Amp),       "no_such_tool"); }
}
