use crate::client::BridgeClient;
use crate::protocol::*;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdin.lock();
    let mut out_handle = stdout.lock();

    let mut line = String::new();
    while handle.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        if let Ok(req) = serde_json::from_str::<RpcRequest>(trimmed) {
            let resp = handle_mcp_request(req);
            if let Some(r) = resp {
                let mut bytes = serde_json::to_vec(&r).unwrap_or_default();
                bytes.push(b'\n');
                out_handle.write_all(&bytes)?;
                out_handle.flush()?;
            }
        }

        line.clear();
    }

    Ok(())
}

fn handle_mcp_request(req: RpcRequest) -> Option<RpcResponse> {
    let id = req.id.clone();
    let method = req.method.as_str();

    match method {
        "initialize" => Some(RpcResponse::success(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {}
                },
                "serverInfo": {
                    "name": "statcraft-bridge",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),
        "notifications/initialized" => None,
        "ping" => Some(RpcResponse::success(id, json!({}))),
        "resources/list" => Some(RpcResponse::success(
            id,
            json!({
                "resources": [
                    {
                        "uri": "statcraft://commands",
                        "name": "Live Engine Commands & Schemas Catalog",
                        "description": "Dynamic catalog of all registered EngineCommands compiled into StatCraft, with their categories, menu paths, syntax keywords, and JSON parameter schemas.",
                        "mimeType": "text/markdown"
                    },
                    {
                        "uri": "statcraft://syntax-reference",
                        "name": "SPSS Syntax & Procedure Reference",
                        "description": "Complete reference manual for batch SPSS syntax (.sps), statistical procedures, parameter schemas, and data transformations in StatCraft.",
                        "mimeType": "text/markdown"
                    }
                ]
            }),
        )),
        "resources/read" => {
            let params = req.params.unwrap_or_default();
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            if uri == "statcraft://commands" {
                let addr = format!("127.0.0.1:{}", DEFAULT_BRIDGE_PORT);
                let text = match BridgeClient::connect(&addr) {
                    Ok(mut client) => match client.list_commands() {
                        Ok(cmds) => format_commands_markdown(&cmds),
                        Err(e) => format!("# Error Retrieving Commands\n\nFailed to fetch commands from StatCraft GUI: {}", e.message),
                    },
                    Err(_) => {
                        "# StatCraft Commands Catalog\n\n*StatCraft GUI is not running or the bridge is disabled. Launch StatCraft to inspect live commands.*".to_string()
                    }
                };
                Some(RpcResponse::success(
                    id,
                    json!({
                        "contents": [
                            {
                                "uri": "statcraft://commands",
                                "mimeType": "text/markdown",
                                "text": text
                            }
                        ]
                    }),
                ))
            } else if uri == "statcraft://syntax-reference" {
                let text = statcraft_syntax::generate_full_syntax_reference();
                Some(RpcResponse::success(
                    id,
                    json!({
                        "contents": [
                            {
                                "uri": "statcraft://syntax-reference",
                                "mimeType": "text/markdown",
                                "text": text
                            }
                        ]
                    }),
                ))
            } else {
                Some(RpcResponse::error(
                    id,
                    RpcError::invalid_params(format!("Unknown resource URI: {}", uri)),
                ))
            }
        }
        "tools/list" => Some(RpcResponse::success(
            id,
            json!({
                "tools": get_tool_definitions()
            }),
        )),
        "tools/call" => {
            let params = req.params.unwrap_or_default();
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let response_content = call_tool(tool_name, arguments);
            Some(RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": response_content
                        }
                    ]
                }),
            ))
        }
        _ => Some(RpcResponse::error(id, RpcError::method_not_found(method))),
    }
}

fn get_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "statcraft_list_commands",
            "description": "Inspect all available statistical procedures, data transformations, and utilities compiled into StatCraft. Returns command IDs, menu hierarchy, categories, syntax keywords, and JSON parameter schemas.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "description": "Optional category filter (e.g., 'Statistics', 'Transform', 'Data')"
                    }
                }
            }
        }),
        json!({
            "name": "statcraft_execute_command",
            "description": "Directly execute any StatCraft engine command by its ID and JSON arguments. Automatically records results into the Output Viewer and returns structured data and formatted text. Can be used for all current and future commands.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command_id": {
                        "type": "string",
                        "description": "The command ID (e.g. 'stat.descriptives', 'stat.npar.kruskal_wallis', 'transform.shift_values')"
                    },
                    "params": {
                        "type": "object",
                        "description": "JSON arguments matching the command schema (inspectable via statcraft_list_commands)"
                    }
                },
                "required": ["command_id"]
            }
        }),
        json!({
            "name": "statcraft_run_syntax",
            "description": "Execute batch SPSS syntax (.sps) in StatCraft. Controls data creation/transformation, variable properties, and statistical procedures. Also supports universal directive: RUN COMMAND \"<command_id>\" { <json_params> }.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "syntax": {
                        "type": "string",
                        "description": "SPSS syntax script (.sps) to execute"
                    },
                    "timeout_seconds": {
                        "type": "integer",
                        "description": "Optional timeout in seconds (default 45, max 300)"
                    }
                },
                "required": ["syntax"]
            }
        }),
        json!({
            "name": "statcraft_load_dataset",
            "description": "Directly load raw tabular data (CSV/TSV text) into the active StatCraft session memory.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "csv_content": {
                        "type": "string",
                        "description": "Raw CSV or TSV table text content"
                    },
                    "name": {
                        "type": "string",
                        "description": "Optional dataset name"
                    }
                },
                "required": ["csv_content"]
            }
        }),
        json!({
            "name": "statcraft_list_variables",
            "description": "Inspect the variable dictionary of the active StatCraft dataset: names, types, measurement levels (Scale, Ordinal, Nominal), labels, value labels, and missing values.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "statcraft_get_data_preview",
            "description": "Retrieve the first N rows of the active dataset for inspection of distributions, types, and values.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Number of rows to preview (default: 10)"
                    }
                }
            }
        }),
        json!({
            "name": "statcraft_insert_output_note",
            "description": "Insert an analytical finding or summary note into the StatCraft Output Viewer tree as a canonical note card.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Note title / section name"
                    },
                    "content": {
                        "type": "string",
                        "description": "Formatted findings or summary text"
                    }
                },
                "required": ["title", "content"]
            }
        }),
        json!({
            "name": "statcraft_set_active_tab",
            "description": "Switch the active tab in StatCraft ('data', 'variable', 'output', 'syntax'). Only use when presenting final results to avoid disrupting user focus.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tab": {
                        "type": "string",
                        "enum": ["data", "variable", "output", "syntax"],
                        "description": "Target tab to switch to"
                    }
                },
                "required": ["tab"]
            }
        }),
    ]
}

fn call_tool(name: &str, args: Value) -> String {
    let addr = format!("127.0.0.1:{}", DEFAULT_BRIDGE_PORT);
    let mut client = match BridgeClient::connect(&addr) {
        Ok(c) => c,
        Err(_) => {
            return "StatCraft GUI is not running or the bridge is disabled. Please launch StatCraft to execute commands.".to_string();
        }
    };

    match name {
        "statcraft_list_commands" => {
            let cat_filter = args.get("category").and_then(|v| v.as_str());
            match client.list_commands() {
                Ok(mut cmds) => {
                    if let Some(cat) = cat_filter {
                        cmds.retain(|c| c.category.eq_ignore_ascii_case(cat));
                    }
                    serde_json::to_string_pretty(&cmds).unwrap_or_else(|_| "[]".to_string())
                }
                Err(err) => format!("Error listing commands: {}", err.message),
            }
        }
        "statcraft_execute_command" => {
            let cmd_id = args
                .get("command_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let params = args.get("params").cloned().unwrap_or(json!({}));
            match client.execute_command(cmd_id, params) {
                Ok(res) => {
                    let mut text = format!("Command '{}' executed successfully.", res.command_id);
                    if let Some(out) = res.output_text {
                        if !out.trim().is_empty() {
                            text.push_str("\n\n");
                            text.push_str(&out);
                        }
                    } else {
                        text.push_str("\n\nResult JSON:\n");
                        let pretty = serde_json::to_string_pretty(&res.result).unwrap_or_default();
                        text.push_str(&pretty);
                    }
                    text
                }
                Err(err) => format!("Error executing command '{}': {}", cmd_id, err.message),
            }
        }
        "statcraft_run_syntax" => {
            let syntax = args.get("syntax").and_then(|v| v.as_str()).unwrap_or("");
            let timeout = args.get("timeout_seconds").and_then(|v| v.as_u64());
            match client.execute_syntax(syntax, timeout) {
                Ok(res) => {
                    let mut text = format!(
                        "Executed {} command(s). {}",
                        res.executed_commands, res.message
                    );
                    if let Some(out) = res.output_text {
                        if !out.trim().is_empty() {
                            text.push_str("\n\n");
                            text.push_str(&out);
                        }
                    }
                    text
                }
                Err(err) => format!("Error executing syntax: {}", err.message),
            }
        }
        "statcraft_load_dataset" => {
            let csv = args
                .get("csv_content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let ds_name = args.get("name").and_then(|v| v.as_str());
            match client.load_csv(csv, ds_name) {
                Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_default(),
                Err(err) => format!("Error loading dataset: {}", err.message),
            }
        }
        "statcraft_list_variables" => match client.get_variables() {
            Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_default(),
            Err(err) => format!("Error retrieving variables: {}", err.message),
        },
        "statcraft_get_data_preview" => {
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            match client.get_data_preview(limit) {
                Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_default(),
                Err(err) => format!("Error retrieving preview: {}", err.message),
            }
        }
        "statcraft_insert_output_note" => {
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("Note");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            match client.add_output_note(title, content) {
                Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_default(),
                Err(err) => format!("Error adding note: {}", err.message),
            }
        }
        "statcraft_set_active_tab" => {
            let tab = args.get("tab").and_then(|v| v.as_str()).unwrap_or("data");
            match client.set_active_tab(tab) {
                Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_default(),
                Err(err) => format!("Error setting active tab: {}", err.message),
            }
        }
        _ => format!("Unknown tool: {}", name),
    }
}

fn format_commands_markdown(cmds: &[CommandSpecSummary]) -> String {
    let mut out = String::new();
    out.push_str("# StatCraft Compiled Engine Commands Catalog\n\n");
    out.push_str(&format!("Total registered commands: {}\n\n", cmds.len()));
    out.push_str("| Command ID | Category | Menu Path | Syntax Keywords |\n");
    out.push_str("|---|---|---|---|\n");
    for cmd in cmds {
        let kw = if cmd.syntax_names.is_empty() {
            "-".to_string()
        } else {
            cmd.syntax_names.join(", ")
        };
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            cmd.id, cmd.category, cmd.menu_path, kw
        ));
    }
    out.push_str("\n## Detailed Parameter Schemas\n\n");
    for cmd in cmds {
        out.push_str(&format!("### `{}`\n", cmd.id));
        out.push_str(&format!("- **Category**: {}\n", cmd.category));
        out.push_str(&format!("- **Menu Path**: {}\n", cmd.menu_path));
        out.push_str("- **Parameter Schema**:\n```json\n");
        let pretty_schema =
            serde_json::to_string_pretty(&cmd.schema).unwrap_or_else(|_| "{}".to_string());
        out.push_str(&pretty_schema);
        out.push_str("\n```\n\n");
    }
    out
}
