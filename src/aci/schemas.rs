pub fn get_all_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read full contents of a file within the sandbox, returning text and provenance metadata.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative file path within sandbox" }
                    },
                    "required": ["path"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "write",
                "description": "Write text content to a file in the sandbox, automatically propagating provenance.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative file path" },
                        "content": { "type": "string", "description": "Text content to write" }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "view_lines",
                "description": "View specific line ranges in a file with 1-indexed line numbers to preserve context tokens.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative file path" },
                        "start_line": { "type": "integer", "description": "Starting line number (1-indexed)" },
                        "end_line": { "type": "integer", "description": "Ending line number (inclusive)" }
                    },
                    "required": ["path", "start_line", "end_line"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "edit_block",
                "description": "Surgically search and replace an exact block of text inside a file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative file path" },
                        "target_content": { "type": "string", "description": "Exact text to search and replace" },
                        "replacement_content": { "type": "string", "description": "New replacement text" }
                    },
                    "required": ["path", "target_content", "replacement_content"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "search_files",
                "description": "Search sandbox for files matching a glob pattern (e.g. '*.rs', 'src/**/*.py').",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Glob pattern to match" }
                    },
                    "required": ["pattern"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "grep",
                "description": "Search for exact string or regex matches across all files in the sandbox.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Text or regex pattern to search" }
                    },
                    "required": ["query"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fetch",
                "description": "Ingest external web resource. Automatically tagged as UNTRUSTED_WEB.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "Remote URL to fetch" },
                        "save_as": { "type": "string", "description": "Optional relative path to save response" }
                    },
                    "required": ["url"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "exec",
                "description": "Execute a command in the sandbox. Boundary policy intercepts untrusted inputs and blocks privileged actions.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "program": { "type": "string", "description": "Command or program to execute" },
                        "args": { "type": "array", "items": { "type": "string" }, "description": "Command arguments" }
                    },
                    "required": ["program"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "snapshot",
                "description": "Create an immutable time-travel checkpoint of current workspace and taint ledger.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "description": { "type": "string", "description": "Human-readable label for checkpoint" }
                    }
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "rewind",
                "description": "Roll back sandbox files and taint status to a prior checkpoint.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "snapshot_id": { "type": "string", "description": "ID of snapshot to rewind to" }
                    },
                    "required": ["snapshot_id"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "observe",
                "description": "Return workspace mutations, file list, and active taint inventory.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        }),
    ]
}
