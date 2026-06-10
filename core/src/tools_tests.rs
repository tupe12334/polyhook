use super::normalize_tool;
use crate::CallerKind;

#[test]
fn unknown_caller_returns_original() {
    assert_eq!(normalize_tool("SomeTool", &CallerKind::Unknown), "SomeTool");
    assert_eq!(normalize_tool("bash", &CallerKind::Unknown), "bash");
    assert_eq!(normalize_tool("Bash", &CallerKind::Unknown), "Bash");
}

// ClaudeCode extra mappings
#[test]
fn claude_ls() {
    assert_eq!(normalize_tool("ls", &CallerKind::ClaudeCode), "list_dir");
}
#[test]
fn claude_grep() {
    assert_eq!(normalize_tool("grep", &CallerKind::ClaudeCode), "grep");
}
#[test]
fn claude_glob() {
    assert_eq!(normalize_tool("glob", &CallerKind::ClaudeCode), "glob");
}
#[test]
fn claude_websearch() {
    assert_eq!(
        normalize_tool("websearch", &CallerKind::ClaudeCode),
        "web_search"
    );
}
#[test]
fn claude_webfetch() {
    assert_eq!(
        normalize_tool("webfetch", &CallerKind::ClaudeCode),
        "web_fetch"
    );
}
#[test]
fn claude_mcp_diag() {
    assert_eq!(
        normalize_tool("mcp__ide__getdiagnostics", &CallerKind::ClaudeCode),
        "diagnostics"
    );
}

// Cursor extra mappings
#[test]
fn cursor_apply_edit() {
    assert_eq!(
        normalize_tool("apply_edit", &CallerKind::Cursor),
        "edit_file"
    );
}
#[test]
fn cursor_list_dir() {
    assert_eq!(normalize_tool("list_dir", &CallerKind::Cursor), "list_dir");
}
#[test]
fn cursor_file_search() {
    assert_eq!(normalize_tool("file_search", &CallerKind::Cursor), "glob");
}
#[test]
fn cursor_fetch_url() {
    assert_eq!(
        normalize_tool("fetch_url", &CallerKind::Cursor),
        "web_fetch"
    );
}
#[test]
fn cursor_spawn_agent() {
    assert_eq!(
        normalize_tool("spawn_agent", &CallerKind::Cursor),
        "spawn_agent"
    );
}
#[test]
fn cursor_get_diag() {
    assert_eq!(
        normalize_tool("get_diagnostics", &CallerKind::Cursor),
        "diagnostics"
    );
}
#[test]
fn cursor_move_file() {
    assert_eq!(
        normalize_tool("move_file", &CallerKind::Cursor),
        "move_file"
    );
}
#[test]
fn cursor_delete_file() {
    assert_eq!(
        normalize_tool("delete_file", &CallerKind::Cursor),
        "delete_file"
    );
}
#[test]
fn cursor_create_dir() {
    assert_eq!(
        normalize_tool("create_dir", &CallerKind::Cursor),
        "create_dir"
    );
}

// Windsurf extra mappings
#[test]
fn windsurf_edit_file() {
    assert_eq!(
        normalize_tool("edit_file", &CallerKind::Windsurf),
        "edit_file"
    );
}
#[test]
fn windsurf_search_files() {
    assert_eq!(
        normalize_tool("search_files", &CallerKind::Windsurf),
        "grep"
    );
}
#[test]
fn windsurf_find_files() {
    assert_eq!(normalize_tool("find_files", &CallerKind::Windsurf), "glob");
}
#[test]
fn windsurf_fetch_page() {
    assert_eq!(
        normalize_tool("fetch_page", &CallerKind::Windsurf),
        "web_fetch"
    );
}
#[test]
fn windsurf_spawn_agent() {
    assert_eq!(
        normalize_tool("spawn_agent", &CallerKind::Windsurf),
        "spawn_agent"
    );
}
#[test]
fn windsurf_get_diag() {
    assert_eq!(
        normalize_tool("get_diagnostics", &CallerKind::Windsurf),
        "diagnostics"
    );
}
#[test]
fn windsurf_move_file() {
    assert_eq!(
        normalize_tool("move_file", &CallerKind::Windsurf),
        "move_file"
    );
}
#[test]
fn windsurf_delete_file() {
    assert_eq!(
        normalize_tool("delete_file", &CallerKind::Windsurf),
        "delete_file"
    );
}
#[test]
fn windsurf_create_dir() {
    assert_eq!(
        normalize_tool("create_directory", &CallerKind::Windsurf),
        "create_dir"
    );
}

// Cline extra mappings
#[test]
fn cline_search_files() {
    assert_eq!(normalize_tool("search_files", &CallerKind::Cline), "grep");
}
#[test]
fn cline_search() {
    assert_eq!(normalize_tool("search", &CallerKind::Cline), "web_search");
}
#[test]
fn cline_fetch() {
    assert_eq!(normalize_tool("fetch", &CallerKind::Cline), "web_fetch");
}
#[test]
fn cline_rename_file() {
    assert_eq!(
        normalize_tool("rename_file", &CallerKind::Cline),
        "move_file"
    );
}
#[test]
fn cline_delete_file() {
    assert_eq!(
        normalize_tool("delete_file", &CallerKind::Cline),
        "delete_file"
    );
}
#[test]
fn cline_create_dir() {
    assert_eq!(
        normalize_tool("create_directory", &CallerKind::Cline),
        "create_dir"
    );
}
#[test]
fn cline_get_diag() {
    assert_eq!(
        normalize_tool("get_diagnostics", &CallerKind::Cline),
        "diagnostics"
    );
}

// Amp extra mappings
#[test]
fn amp_search_grep() {
    assert_eq!(normalize_tool("search.grep", &CallerKind::Amp), "grep");
}
#[test]
fn amp_search_glob() {
    assert_eq!(normalize_tool("search.glob", &CallerKind::Amp), "glob");
}
#[test]
fn amp_web_fetch() {
    assert_eq!(normalize_tool("web.fetch", &CallerKind::Amp), "web_fetch");
}
#[test]
fn amp_agent_spawn() {
    assert_eq!(
        normalize_tool("agent.spawn", &CallerKind::Amp),
        "spawn_agent"
    );
}
#[test]
fn amp_lsp_diag() {
    assert_eq!(
        normalize_tool("lsp.diagnostics", &CallerKind::Amp),
        "diagnostics"
    );
}
#[test]
fn amp_fs_move() {
    assert_eq!(normalize_tool("fs.move", &CallerKind::Amp), "move_file");
}
#[test]
fn amp_fs_delete() {
    assert_eq!(normalize_tool("fs.delete", &CallerKind::Amp), "delete_file");
}
#[test]
fn amp_fs_mkdir() {
    assert_eq!(normalize_tool("fs.mkdir", &CallerKind::Amp), "create_dir");
}

// Fallthrough: unmapped tool names pass through unchanged for each known caller
#[test]
fn claude_unknown_falls_through() {
    assert_eq!(
        normalize_tool("no_such_tool", &CallerKind::ClaudeCode),
        "no_such_tool"
    );
}
#[test]
fn cursor_unknown_falls_through() {
    assert_eq!(
        normalize_tool("no_such_tool", &CallerKind::Cursor),
        "no_such_tool"
    );
}
#[test]
fn windsurf_unknown_falls_through() {
    assert_eq!(
        normalize_tool("no_such_tool", &CallerKind::Windsurf),
        "no_such_tool"
    );
}
#[test]
fn cline_unknown_falls_through() {
    assert_eq!(
        normalize_tool("no_such_tool", &CallerKind::Cline),
        "no_such_tool"
    );
}
#[test]
fn amp_unknown_falls_through() {
    assert_eq!(
        normalize_tool("no_such_tool", &CallerKind::Amp),
        "no_such_tool"
    );
}
#[test]
fn gemini_cli_unknown_falls_through() {
    assert_eq!(
        normalize_tool("no_such_tool", &CallerKind::GeminiCli),
        "no_such_tool"
    );
}

// GeminiCli tool name mappings
#[test]
fn gc_run_shell_command() {
    assert_eq!(
        normalize_tool("run_shell_command", &CallerKind::GeminiCli),
        "bash"
    );
}
#[test]
fn gc_read_file() {
    assert_eq!(
        normalize_tool("read_file", &CallerKind::GeminiCli),
        "read_file"
    );
}
#[test]
fn gc_write_file() {
    assert_eq!(
        normalize_tool("write_file", &CallerKind::GeminiCli),
        "write_file"
    );
}
#[test]
fn gc_replace() {
    assert_eq!(
        normalize_tool("replace", &CallerKind::GeminiCli),
        "edit_file"
    );
}
#[test]
fn gc_list_directory() {
    assert_eq!(
        normalize_tool("list_directory", &CallerKind::GeminiCli),
        "list_dir"
    );
}
#[test]
fn gc_glob() {
    assert_eq!(normalize_tool("glob", &CallerKind::GeminiCli), "glob");
}
#[test]
fn gc_grep() {
    assert_eq!(normalize_tool("grep", &CallerKind::GeminiCli), "grep");
}
#[test]
fn gc_google_web_search() {
    assert_eq!(
        normalize_tool("google_web_search", &CallerKind::GeminiCli),
        "web_search"
    );
}
#[test]
fn gc_fetch() {
    assert_eq!(normalize_tool("fetch", &CallerKind::GeminiCli), "web_fetch");
}
#[test]
fn gc_move_file() {
    assert_eq!(
        normalize_tool("move_file", &CallerKind::GeminiCli),
        "move_file"
    );
}
#[test]
fn gc_delete_file() {
    assert_eq!(
        normalize_tool("delete_file", &CallerKind::GeminiCli),
        "delete_file"
    );
}
#[test]
fn gc_make_directory() {
    assert_eq!(
        normalize_tool("make_directory", &CallerKind::GeminiCli),
        "create_dir"
    );
}

// Hermes tool name mappings
#[test]
fn hermes_terminal() {
    assert_eq!(normalize_tool("terminal", &CallerKind::Hermes), "bash");
}
#[test]
fn hermes_read_file() {
    assert_eq!(
        normalize_tool("read_file", &CallerKind::Hermes),
        "read_file"
    );
}
#[test]
fn hermes_write_file() {
    assert_eq!(
        normalize_tool("write_file", &CallerKind::Hermes),
        "write_file"
    );
}
#[test]
fn hermes_patch() {
    assert_eq!(normalize_tool("patch", &CallerKind::Hermes), "edit_file");
}
#[test]
fn hermes_search_files() {
    assert_eq!(normalize_tool("search_files", &CallerKind::Hermes), "grep");
}
#[test]
fn hermes_delegate_task() {
    assert_eq!(
        normalize_tool("delegate_task", &CallerKind::Hermes),
        "spawn_agent"
    );
}
#[test]
fn hermes_browser_tool() {
    assert_eq!(
        normalize_tool("browser_navigate", &CallerKind::Hermes),
        "browser"
    );
}
#[test]
fn hermes_unknown_falls_through() {
    assert_eq!(
        normalize_tool("no_such_tool", &CallerKind::Hermes),
        "no_such_tool"
    );
}
