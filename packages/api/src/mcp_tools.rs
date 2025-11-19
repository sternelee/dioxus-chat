// MCP Tools Integration with Rig Agents
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use rig::{
    completion::ToolDefinition,
    tool::Tool,
};
use futures::Stream;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Tool as ApiTool, ToolCall, ToolResult};

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub description: Option<String>,
    pub timeout_ms: u64,
    pub enabled: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "local-mcp".to_string(),
            command: "mcp-server".to_string(),
            args: vec![],
            description: None,
            timeout_ms: 30000,
            enabled: true,
        }
    }
}

/// MCP Tool Registry for managing MCP servers and tools
pub struct McpToolRegistry {
    servers: Arc<RwLock<HashMap<String, McpServerConfig>>>,
    tools: Arc<RwLock<HashMap<String, ApiTool>>>,
    active_clients: Arc<RwLock<HashMap<String, McpClient>>>,
}

impl McpToolRegistry {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            active_clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new MCP server configuration
    pub async fn add_server(&self, config: McpServerConfig) -> Result<()> {
        let mut servers = self.servers.write().await;
        servers.insert(config.name.clone(), config);

        // Automatically connect if enabled
        if config.enabled {
            self.connect_server(&config.name).await?;
        }

        Ok(())
    }

    /// Connect to an MCP server and discover tools
    async fn connect_server(&self, server_name: &str) -> Result<()> {
        let servers = self.servers.read().await;
        let config = servers.get(server_name)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_name))?;

        if !config.enabled {
            return Ok(());
        }

        let client = McpClient::new(config.clone()).await?;

        // Discover tools from the server
        let tools = client.list_tools().await?;

        // Update tools registry
        let mut tools_map = self.tools.write().await;
        for tool in tools {
            tools_map.insert(tool.name.clone(), tool);
        }

        // Store client connection
        let mut clients = self.active_clients.write().await;
        clients.insert(server_name.to_string(), client);

        Ok(())
    }

    /// Disconnect from an MCP server
    pub async fn disconnect_server(&self, server_name: &str) -> Result<()> {
        let mut clients = self.active_clients.write().await;
        if let Some(mut client) = clients.remove(server_name) {
            client.close().await?;
        }
        Ok(())
    }

    /// Get all available MCP tools
    pub async fn get_tools(&self) -> Vec<ApiTool> {
        self.tools.read().await.values().cloned().collect()
    }

    /// Execute a tool call
    pub async fn call_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool {} not found", tool_name))?;

        let clients = self.active_clients.read().await;
        for client in clients.values() {
            if client.has_tool(tool_name).await? {
                return client.call_tool(tool_name, arguments).await;
            }
        }

        Err(anyhow::anyhow!("No active client has tool {}", tool_name))
    }

    /// Get server configurations
    pub async fn get_servers(&self) -> Vec<McpServerConfig> {
        self.servers.read().await.values().cloned().collect()
    }

    /// Update server configuration
    pub async fn update_server(&self, name: &str, config: McpServerConfig) -> Result<()> {
        let was_enabled = {
            let servers = self.servers.read().await;
            servers.get(name).map(|s| s.enabled).unwrap_or(false)
        };

        // Disconnect if currently connected
        if was_enabled {
            self.disconnect_server(name).await?;
        }

        // Update configuration
        {
            let mut servers = self.servers.write().await;
            servers.insert(name.to_string(), config.clone());
        }

        // Reconnect if enabled
        if config.enabled {
            self.connect_server(name).await?;
        }

        Ok(())
    }

    /// Remove a server
    pub async fn remove_server(&self, name: &str) -> Result<()> {
        self.disconnect_server(name).await?;

        // Remove server tools from registry
        let mut tools = self.tools.write().await;
        tools.retain(|_, tool| !tool.name.starts_with(&format!("{}.", name)));

        // Remove server configuration
        let mut servers = self.servers.write().await;
        servers.remove(name);

        Ok(())
    }
}

impl Default for McpToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP Client for communicating with MCP servers
pub struct McpClient {
    config: McpServerConfig,
    process: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
    stdout: Option<tokio::process::ChildStdout>,
}

impl McpClient {
    pub async fn new(config: McpServerConfig) -> Result<Self> {
        let mut process = Command::new(&config.command)
            .args(&config.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take();
        let stdout = process.stdout.take();

        Ok(Self {
            config,
            process: Some(process),
            stdin,
            stdout,
        })
    }

    async fn send_message(&mut self, message: serde_json::Value) -> Result<serde_json::Value> {
        // This would implement the actual MCP protocol communication
        // For now, we'll use a simplified approach
        Ok(json!({"result": "ok"}))
    }

    pub async fn list_tools(&self) -> Result<Vec<ApiTool>> {
        // Simulated tool listing - in real implementation, would query MCP server
        Ok(vec![
            ApiTool {
                name: "mcp.file.read".to_string(),
                description: "Read file contents".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path to read"
                        }
                    },
                    "required": ["path"]
                }),
                is_mcp: true,
            },
            ApiTool {
                name: "mcp.file.write".to_string(),
                description: "Write content to file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write"
                        }
                    },
                    "required": ["path", "content"]
                }),
                is_mcp: true,
            },
            ApiTool {
                name: "mcp.http.request".to_string(),
                description: "Make HTTP request".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to request"
                        },
                        "method": {
                            "type": "string",
                            "enum": ["GET", "POST", "PUT", "DELETE"],
                            "default": "GET"
                        }
                    },
                    "required": ["url"]
                }),
                is_mcp: true,
            },
            ApiTool {
                name: "mcp.database.query".to_string(),
                description: "Execute database query".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "SQL query to execute"
                        }
                    },
                    "required": ["query"]
                }),
                is_mcp: true,
            },
            ApiTool {
                name: "mcp.computer.mouse".to_string(),
                description: "Control mouse".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["click", "move", "scroll"],
                            "description": "Mouse action"
                        },
                        "x": {
                            "type": "integer",
                            "description": "X coordinate"
                        },
                        "y": {
                            "type": "integer",
                            "description": "Y coordinate"
                        }
                    },
                    "required": ["action"]
                }),
                is_mcp: true,
            }
        ])
    }

    pub async fn has_tool(&self, tool_name: &str) -> Result<bool> {
        // Check if this client provides the specified tool
        Ok(tool_name.starts_with("mcp.") || tool_name.starts_with(&format!("{}.", self.config.name)))
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        // Simulate tool execution - in real implementation, would call MCP server
        match tool_name {
            "mcp.file.read" => {
                let path = arguments.get("path").and_then(|p| p.as_str()).unwrap_or("unknown");
                Ok(ToolResult {
                    tool_call_id: tool_name.to_string(),
                    result: json!(format!("Read content from file: {}", path)),
                    error: None,
                })
            },
            "mcp.file.write" => {
                let path = arguments.get("path").and_then(|p| p.as_str()).unwrap_or("unknown");
                let content = arguments.get("content").and_then(|c| c.as_str()).unwrap_or("");
                Ok(ToolResult {
                    tool_call_id: tool_name.to_string(),
                    result: json!(format!("Wrote {} characters to file: {}", content.len(), path)),
                    error: None,
                })
            },
            "mcp.http.request" => {
                let url = arguments.get("url").and_then(|u| u.as_str()).unwrap_or("unknown");
                let method = arguments.get("method").and_then(|m| m.as_str()).unwrap_or("GET");
                Ok(ToolResult {
                    tool_call_id: tool_name.to_string(),
                    result: json!(format!("Made {} request to: {}", method, url)),
                    error: None,
                })
            },
            "mcp.database.query" => {
                let query = arguments.get("query").and_then(|q| q.as_str()).unwrap_or("SELECT");
                Ok(ToolResult {
                    tool_call_id: tool_name.to_string(),
                    result: json!(format!("Executed database query: {}", query)),
                    error: None,
                })
            },
            "mcp.computer.mouse" => {
                let action = arguments.get("action").and_then(|a| a.as_str()).unwrap_or("click");
                let x = arguments.get("x").and_then(|x| x.as_i64()).unwrap_or(0);
                let y = arguments.get("y").and_then(|y| y.as_i64()).unwrap_or(0);
                Ok(ToolResult {
                    tool_call_id: tool_name.to_string(),
                    result: json!(format!("Executed mouse action: {} at ({}, {})", action, x, y)),
                    error: None,
                })
            },
            _ => Ok(ToolResult {
                tool_call_id: tool_name.to_string(),
                result: json!("Tool not implemented"),
                error: Some("Tool not implemented".to_string()),
            })
        }
    }

    pub async fn close(mut self) -> Result<()> {
        if let Some(process) = self.process.take() {
            let _ = process.kill().await;
        }
        Ok(())
    }
}

/// Enhanced RigAgentService with MCP support
pub struct EnhancedRigAgentService {
    base_service: crate::RigAgentService,
    mcp_registry: McpToolRegistry,
}

impl EnhancedRigAgentService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            base_service: crate::RigAgentService::new()?,
            mcp_registry: McpToolRegistry::new(),
        })
    }

    /// Initialize with default MCP servers
    pub async fn with_default_mcp_servers(self) -> Result<Self> {
        // Add some default MCP server configurations
        let default_servers = vec![
            McpServerConfig {
                name: "filesystem".to_string(),
                command: "npx".to_string(),
                args: vec!["@modelcontextprotocol/server-filesystem", "/tmp"],
                description: Some("Filesystem access MCP server".to_string()),
                timeout_ms: 10000,
                enabled: true,
            },
            McpServerConfig {
                name: "git".to_string(),
                command: "npx".to_string(),
                args: vec!["@modelcontextprotocol/server-git", "/tmp/repo"],
                description: Some("Git operations MCP server".to_string()),
                timeout_ms: 15000,
                enabled: true,
            },
            McpServerConfig {
                name: "computer".to_string(),
                command: "npx".to_string(),
                args: vec!["@modelcontextprotocol/server-computer"],
                description: Some("Computer control MCP server".to_string()),
                timeout_ms: 20000,
                enabled: false, // Disabled by default for safety
            },
        ];

        for server in default_servers {
            if let Err(e) = self.mcp_registry.add_server(server).await {
                eprintln!("Failed to add MCP server: {}", e);
            }
        }

        Ok(self)
    }

    /// Get all available tools (both built-in and MCP)
    pub async fn get_all_tools(&self, model_id: &str) -> Vec<ApiTool> {
        let mut all_tools = self.base_service.list_tools(model_id).await;
        all_tools.extend(self.mcp_registry.get_tools().await);
        all_tools
    }

    /// Enhanced send_message that supports MCP tools
    pub async fn send_message_with_mcp(&self, request: crate::ChatRequest) -> Result<crate::ChatResponse> {
        // First try normal response
        let mut response = self.base_service.send_message(request.clone()).await?;

        // If no tools were requested or available, return normal response
        if request.tools.as_ref().map_or(true, |t| t.is_empty()) {
            return Ok(response);
        }

        // If tools were requested, check if any are MCP tools
        let model_id = &request.model;
        let all_tools = self.get_all_tools(model_id).await;
        let mcp_tools: Vec<_> = all_tools.iter()
            .filter(|t| t.is_mcp)
            .collect();

        if mcp_tools.is_empty() {
            return Ok(response);
        }

        // Process MCP tool calls if present in response
        if let Some(ref message) = response.message {
            if let Some(ref tool_calls) = message.tool_calls {
                let mut tool_results = Vec::new();

                for tool_call in tool_calls {
                    if mcp_tools.iter().any(|t| t.name == tool_call.name) {
                        // This is an MCP tool call
                        match self.mcp_registry.call_tool(&tool_call.name, tool_call.arguments.clone()).await {
                            Ok(result) => tool_results.push(result),
                            Err(e) => {
                                tool_results.push(ToolResult {
                                    tool_call_id: tool_call.name.clone(),
                                    result: json!(null),
                                    error: Some(format!("MCP tool execution failed: {}", e)),
                                });
                            }
                        }
                    }
                }

                // Add tool results to response
                response.tool_results = Some(tool_results);
            }
        }

        Ok(response)
    }

    /// Get MCP server configurations
    pub async fn get_mcp_servers(&self) -> Vec<McpServerConfig> {
        self.mcp_registry.get_servers().await
    }

    /// Add MCP server
    pub async fn add_mcp_server(&self, config: McpServerConfig) -> Result<()> {
        self.mcp_registry.add_server(config).await
    }

    /// Update MCP server
    pub async fn update_mcp_server(&self, name: &str, config: McpServerConfig) -> Result<()> {
        self.mcp_registry.update_server(name, config).await
    }

    /// Remove MCP server
    pub async fn remove_mcp_server(&self, name: &str) -> Result<()> {
        self.mcp_registry.remove_server(name).await
    }

    /// Test MCP server connection
    pub async fn test_mcp_server(&self, name: &str) -> Result<String> {
        self.mcp_registry.disconnect_server(name).await?;
        self.mcp_registry.connect_server(name).await?;
        let tools = self.mcp_registry.get_tools().await;
        let mcp_tools: Vec<_> = tools.iter()
            .filter(|t| t.is_mcp)
            .collect();

        Ok(format!("Connected successfully. Found {} MCP tools.", mcp_tools.len()))
    }
}

impl Default for EnhancedRigAgentService {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

// Implement delegation for base service methods
impl std::ops::Deref for EnhancedRigAgentService {
    type Target = crate::RigAgentService;

    fn deref(&self) -> &Self::Target {
        &self.base_service
    }
}