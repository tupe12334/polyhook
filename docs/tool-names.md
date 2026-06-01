# Tool Name Mappings

Full mapping from each AI tool's vendor name to the canonical polyhook name.

---

## File Operations

| polyhook | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `read_file` | `Read` | `read_file` | `read_file` | `read_file` | `file.read` |
| `write_file` | `Write` | `edit_file` | `write_file` | `write_to_file` | `file.write` |
| `edit_file` | `Edit` | `apply_edit` | `edit_file` | `apply_diff` | `file.edit` |
| `list_dir` | `LS` | `list_dir` | `list_directory` | `list_files` | `fs.list` |
| `move_file` | — | `move_file` | `move_file` | `rename_file` | `fs.move` |
| `delete_file` | — | `delete_file` | `delete_file` | `delete_file` | `fs.delete` |
| `create_dir` | — | `create_dir` | `create_directory` | `create_directory` | `fs.mkdir` |

## Shell

| polyhook | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `bash` | `Bash` | `run_terminal_cmd` | `run_command` | `execute_command` | `shell` |

## Search

| polyhook | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `grep` | `Grep` | `grep_search` | `search_files` | `search_files` | `search.grep` |
| `glob` | `Glob` | `file_search` | `find_files` | `list_files` | `search.glob` |

## Web

| polyhook | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `web_search` | `WebSearch` | `web_search` | `search_web` | `search` | `web.search` |
| `web_fetch` | `WebFetch` | `fetch_url` | `fetch_page` | `fetch` | `web.fetch` |

## Code Intelligence

| polyhook | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `diagnostics` | `mcp__ide__getDiagnostics` | `get_diagnostics` | `get_diagnostics` | `get_diagnostics` | `lsp.diagnostics` |

## Agents / Subprocesses

| polyhook | Claude Code | Cursor | Windsurf | Cline | Amp |
|---|---|---|---|---|---|
| `spawn_agent` | `Task` | `spawn_agent` | `spawn_agent` | — | `agent.spawn` |

---

## Notes

- A `—` means the tool has no equivalent for that operation.
- Vendor names are matched case-insensitively during normalization.
- If a tool name is not in this table, polyhook passes it through as-is and sets `tool` to the raw vendor name.
- To propose a new mapping, open a PR editing `crates/polyhook-core/src/tools.rs` and this file.
