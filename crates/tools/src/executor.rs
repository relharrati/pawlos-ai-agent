use anyhow::Result;
use serde_json::Value;
use memory::tool::{MemoryTool};

use crate::shell::ShellTool;
use crate::file::FileTool;
use crate::web::WebTool;
use crate::skill::{Skill, skill_load_definition, skill_create_definition};
use crate::agent::AgentTool;
use crate::browser::BrowserTool;
use crate::mcp::{McpRegistry, McpServer};

/// Dispatches tool calls from the LLM to the right implementation
pub struct ToolExecutor;

impl ToolExecutor {
    /// Returns all tool definitions to pass to the LLM
    pub fn all_definitions() -> Vec<Value> {
        vec![
            MemoryTool::tool_definition(),
            AgentTool::tool_definition(),
            BrowserTool::tool_definition(),
            ShellTool::tool_definition(),
            FileTool::read_definition(),
            FileTool::write_definition(),
            FileTool::edit_definition(),
            WebTool::search_definition(),
            WebTool::fetch_definition(),
            skill_load_definition(),
            skill_create_definition(),
            // MCP tools - these are dynamically added based on configured servers
        ]
    }

    /// Add MCP server tools (called at startup)
    pub fn add_mcp_tools(registry: &McpRegistry) -> Vec<Value> {
        let mut tools = vec![];
        for (name, _server) in registry.servers.iter() {
            tools.push(serde_json::json!({
                "name": format!("mcp_{}", name),
                "description": format!("Call MCP server '{}' - use this for {} operations", name, name),
                "parameters": {
                    "type": "object",
                    "properties": {
                        "server": { "type": "string", "description": "MCP server name" },
                        "tool": { "type": "string", "description": "Tool to call on the server" },
                        "args": { "type": "object", "description": "Tool arguments" }
                    },
                    "required": ["server", "tool"]
                }
            }));
        }
        tools
    }

    /// Execute a named tool and return its string result
    pub async fn execute(name: &str, args: &Value) -> Result<String> {
        tracing::debug!("Tool call: {name} args={args}");
        match name {
            "memory" => {
                let action = MemoryTool::parse_action(args)?;
                MemoryTool::execute(action).await
            }
            "shell" => ShellTool::execute(args).await,
            "file_read"  => FileTool::read(args).await,
            "file_write" => FileTool::write(args).await,
            "file_edit"  => FileTool::edit(args).await,
            "web_search" => WebTool::search(args).await,
            "web_fetch"  => WebTool::fetch(args).await,
            "skill_load" => {
                let skill_name = args["name"].as_str().unwrap_or("");
                match Skill::load(skill_name) {
                    Ok(skill) => Ok(format!(
                        "Skill '{}' loaded.\n\nDescription: {}\n\nPrompt:\n{}",
                        skill.name, skill.description, skill.prompt
                    )),
                    Err(e) => Ok(format!("Error loading skill: {e}")),
                }
            }
            "skill_create" => {
                let skill = Skill {
                    name:        args["name"].as_str().unwrap_or("").to_string(),
                    description: args["description"].as_str().unwrap_or("").to_string(),
                    prompt:      args["prompt"].as_str().unwrap_or("").to_string(),
                    tool_requirements: args["tool_requirements"]
                        .as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default(),
                };
                skill.save()?;
                Ok(format!("Skill '{}' created and saved.", skill.name))
            }
            "agent" => {
                let action = AgentTool::parse_action(args)?;
                AgentTool::execute(action)
            }
            "browser" => {
                let action = BrowserTool::parse_action(args)?;
                BrowserTool::execute(action).await
            }
            other if other.starts_with("mcp_") => {
                let server = other.trim_start_matches("mcp_");
                let mcp_tool = args["tool"].as_str().unwrap_or("");
                let mcp_args = &args["args"];
                Ok(format!("MCP call to '{}': {}({:?}) - MCP integration requires server to be running", 
                    server, mcp_tool, mcp_args))
            }
            other => Ok(format!("Unknown tool: {other}")),
        }
    }
}
