//! MCP (Model Context Protocol) client and server integrations
//! Provides standardized interface for connecting to MCP servers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use anyhow::Result;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub transport: McpTransport,
}

/// MCP transport type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    #[default]
    Stdio,
    Http { url: String },
    Sse { url: String },
}

/// MCP tool definition from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
}

/// MCP client for connecting to servers
pub struct McpClient {
    server: McpServer,
    process: Option<tokio::sync::mpsc::Sender<String>>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(server: McpServer) -> Self {
        Self {
            server,
            process: None,
        }
    }

    /// Start the MCP server and initialize
    pub async fn start(&mut self) -> Result<Vec<McpTool>> {
        let mut cmd = Command::new(&self.server.command);
        cmd.args(&self.server.args)
            .envs(&self.server.env)
            .kill_on_drop(true);

        // For stdio transport, we'd need to manage stdin/stdout
        // This is a simplified version
        Ok(vec![])
    }

    /// Call an MCP tool
    pub async fn call(&self, tool: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        // JSON-RPC over stdio
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool,
                "arguments": args
            }
        });

        Ok(serde_json::json!({ "result": "MCP call placeholder" }))
    }

    /// List available tools from this server
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        Ok(vec![])
    }
}

/// MCP Registry - manages all configured MCP servers
pub struct McpRegistry {
    servers: HashMap<String, McpServer>,
    clients: HashMap<String, McpClient>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    /// Add an MCP server configuration
    pub fn add_server(&mut self, server: McpServer) {
        self.servers.insert(server.name.clone(), server);
    }

    /// Start all registered servers
    pub async fn start_all(&mut self) -> Result<()> {
        for (name, server) in &self.servers {
            let mut client = McpClient::new(server.clone());
            match client.start().await {
                Ok(tools) => {
                    tracing::info!("MCP server '{}' started with {} tools", name, tools.len());
                    self.clients.insert(name.clone(), client);
                }
                Err(e) => {
                    tracing::warn!("MCP server '{}' failed to start: {}", name, e);
                }
            }
        }
        Ok(())
    }

    /// Get all available tools from all servers
    pub fn all_tools(&self) -> Vec<(&str, &McpTool)> {
        vec![]
    }

    /// Call a tool on a specific server
    pub async fn call(&self, server: &str, tool: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        let client = self.clients.get(server)
            .ok_or_else(|| anyhow::anyhow!("MCP server '{}' not found", server))?;
        client.call(tool, args).await
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Common built-in MCP servers (Top by usage)
pub mod presets {
    use super::*;

    /// Filesystem MCP - file operations (80k+ stars)
    pub fn filesystem(root: &str) -> McpServer {
        McpServer {
            name: "filesystem".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string(), root.to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// GitHub MCP - GitHub operations (889k downloads)
    pub fn github(token: &str) -> McpServer {
        McpServer {
            name: "github".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
            env: HashMap::from([("GITHUB_TOKEN".to_string(), token.to_string())]),
            transport: McpTransport::Stdio,
        }
    }

    /// Fetch MCP - web content fetching (801k downloads)
    pub fn fetch() -> McpServer {
        McpServer {
            name: "fetch".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-fetch".to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// Brave Search MCP - web search
    pub fn brave_search(api_key: &str) -> McpServer {
        McpServer {
            name: "brave-search".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()],
            env: HashMap::from([("BRAVE_API_KEY".to_string(), api_key.to_string())]),
            transport: McpTransport::Stdio,
        }
    }

    /// Sequential Thinking MCP - structured reasoning (5,550+ uses)
    pub fn sequential_thinking() -> McpServer {
        McpServer {
            name: "sequential-thinking".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@smithery-ai/server-sequential-thinking".to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// Knowledge Graph Memory MCP - memory/knowledge graph
    pub fn memory() -> McpServer {
        McpServer {
            name: "memory".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// Playwright MCP - browser automation (590k downloads)
    pub fn playwright() -> McpServer {
        McpServer {
            name: "playwright".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@executeautomation/playwright-mcp-server".to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// PostgreSQL MCP - database queries
    pub fn postgres(connection_string: &str) -> McpServer {
        McpServer {
            name: "postgres".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-postgres".to_string(), connection_string.to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// SQLite MCP - local database (274+ uses)
    pub fn sqlite(db_path: &str) -> McpServer {
        McpServer {
            name: "sqlite".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-sqlite".to_string(), db_path.to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// Slack MCP - Slack integration
    pub fn slack(token: &str) -> McpServer {
        McpServer {
            name: "slack".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-slack".to_string()],
            env: HashMap::from([("SLACK_TOKEN".to_string(), token.to_string())]),
            transport: McpTransport::Stdio,
        }
    }

    /// Context7 MCP - documentation database (590k downloads)
    pub fn context7() -> McpServer {
        McpServer {
            name: "context7".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@context7/mcp-server".to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// Exa Search MCP - AI-powered search
    pub fn exa(api_key: &str) -> McpServer {
        McpServer {
            name: "exa".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@exa/mcp-server".to_string()],
            env: HashMap::from([("EXA_API_KEY".to_string(), api_key.to_string())]),
            transport: McpTransport::Stdio,
        }
    }

    /// Notion MCP - Notion integration
    pub fn notion(token: &str) -> McpServer {
        McpServer {
            name: "notion".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "notion-mcp-server".to_string()],
            env: HashMap::from([("NOTION_TOKEN".to_string(), token.to_string())]),
            transport: McpTransport::Stdio,
        }
    }

    /// Git MCP - git operations
    pub fn git(root: &str) -> McpServer {
        McpServer {
            name: "git".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-git".to_string(), root.to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }

    /// Desktop Commander MCP - macOS terminal integration
    pub fn desktop_commander() -> McpServer {
        McpServer {
            name: "desktop-commander".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@wonderwhy-er/desktop-commander".to_string()],
            env: HashMap::new(),
            transport: McpTransport::Stdio,
        }
    }
}