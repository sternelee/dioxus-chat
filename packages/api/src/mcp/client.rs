use super::protocol::*;
use crate::chat_service::{Tool as ChatTool, ToolCall, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

#[async_trait]
pub trait McpClient: Send + Sync {
    async fn initialize(&mut self) -> Result<()>;
    async fn list_tools(&mut self) -> Result<Vec<Tool>>;
    async fn call_tool(&mut self, name: &str, arguments: Option<Value>) -> Result<CallToolResult>;
    fn name(&self) -> &str;
    fn is_ready(&self) -> bool;
}

pub struct StdioMcpClient {
    name: String,
    command: String,
    args: Vec<String>,
    child: Option<Child>,
    request_id: i32,
    ready: bool,
}

impl StdioMcpClient {
    pub fn new(name: String, command: String, args: Vec<String>) -> Self {
        Self {
            name,
            command,
            args,
            child: None,
            request_id: 0,
            ready: false,
        }
    }

    async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let request_id = self.next_id();
        let child = self.child.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Child process not started"))?;
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(request_id)),
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;
        
        // Send request to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(request_json.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        } else {
            return Err(anyhow::anyhow!("Cannot write to child stdin"));
        }

        // Read response from stdout
        if let Some(stdout) = child.stdout.as_mut() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            
            match reader.read_line(&mut line).await {
                Ok(0) => return Err(anyhow::anyhow!("EOF while reading response")),
                Ok(_) => {
                    let response: JsonRpcResponse = serde_json::from_str(&line.trim())
                        .map_err(|e| anyhow::anyhow!("Failed to parse JSON-RPC response: {}", e))?;
                    
                    if let Some(error) = response.error {
                        return Err(anyhow::anyhow!("JSON-RPC error: {} - {}", error.code, error.message));
                    }
                    
                    Ok(response.result.unwrap_or(json!(null)))
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to read response: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("Cannot read from child stdout"))
        }
    }

    fn next_id(&mut self) -> i32 {
        self.request_id += 1;
        self.request_id
    }

    fn convert_mcp_tool_to_chat_tool(&self, mcp_tool: Tool) -> ChatTool {
        ChatTool {
            name: mcp_tool.name,
            description: mcp_tool.description,
            input_schema: mcp_tool.input_schema,
            is_mcp: true,
        }
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing MCP client: {}", self.name);

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start MCP server '{}': {}", self.command, e))?;

        // Initialize the MCP connection
        let init_params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                experimental: None,
                sampling: Some(SamplingCapability {}),
            },
            client_info: ClientInfo {
                name: "dioxus-chat".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        let result = self.send_request("initialize", Some(json!(init_params))).await?;
        debug!("Initialize result: {}", result);

        // Send initialized notification
        self.send_request("notifications/initialized", Some(json!({}))).await?;

        self.child = Some(child);
        self.ready = true;

        info!("MCP client '{}' initialized successfully", self.name);
        Ok(())
    }

    async fn list_tools(&mut self) -> Result<Vec<Tool>> {
        if !self.ready {
            return Err(anyhow::anyhow!("MCP client not initialized"));
        }

        let result = self.send_request("tools/list", None).await?;
        let list_result: ListToolsResult = serde_json::from_value(result)
            .map_err(|e| anyhow::anyhow!("Failed to parse tools list: {}", e))?;

        Ok(list_result.tools)
    }

    async fn call_tool(&mut self, name: &str, arguments: Option<Value>) -> Result<CallToolResult> {
        if !self.ready {
            return Err(anyhow::anyhow!("MCP client not initialized"));
        }

        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        let result = self.send_request("tools/call", Some(json!(params))).await?;
        let tool_result: CallToolResult = serde_json::from_value(result)
            .map_err(|e| anyhow::anyhow!("Failed to parse tool result: {}", e))?;

        Ok(tool_result)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Drop for StdioMcpClient {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            debug!("Terminating MCP client: {}", self.name);
            let _ = child.kill();
        }
    }
}

pub struct McpToolExecutor {
    clients: HashMap<String, Box<dyn McpClient>>,
}

impl McpToolExecutor {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client: Box<dyn McpClient>) {
        info!("Adding MCP client: {}", client.name());
        self.clients.insert(client.name().to_string(), client);
    }

    pub async fn initialize_all(&mut self) -> Result<()> {
        for (name, client) in &mut self.clients {
            match client.initialize().await {
                Ok(()) => info!("MCP client '{}' initialized successfully", name),
                Err(e) => warn!("Failed to initialize MCP client '{}': {}", name, e),
            }
        }
        Ok(())
    }

    pub async fn list_all_tools(&mut self) -> Result<Vec<ChatTool>> {
        let mut all_tools = Vec::new();

        for (client_name, client) in &mut self.clients {
            if !client.is_ready() {
                continue;
            }

            match client.list_tools().await {
                Ok(tools) => {
                    for tool in tools {
                        all_tools.push(ChatTool {
                            name: format!("{}:{}", client_name, tool.name),
                            description: format!("{} - {}", tool.description, client_name),
                            input_schema: tool.input_schema,
                            is_mcp: true,
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to list tools from client '{}': {}", client_name, e);
                }
            }
        }

        Ok(all_tools)
    }

    pub async fn execute_tool(&mut self, tool_name: &str, arguments: Option<Value>) -> Result<Vec<String>> {
        // Parse tool name to extract client and actual tool name
        let parts: Vec<&str> = tool_name.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid tool name format: {}", tool_name));
        }

        let client_name = parts[0];
        let actual_tool_name = parts[1];

        let client = self.clients.get_mut(client_name)
            .ok_or_else(|| anyhow::anyhow!("MCP client not found: {}", client_name))?;

        if !client.is_ready() {
            return Err(anyhow::anyhow!("MCP client not ready: {}", client_name));
        }

        let result = client.call_tool(actual_tool_name, arguments).await?;

        let mut outputs = Vec::new();
        for content in result.content {
            match content {
                ToolContent::Text { text } => {
                    outputs.push(text);
                }
                ToolContent::Image { data, mime_type } => {
                    outputs.push(format!("Image ({}): {} bytes", mime_type, data.len()));
                }
                ToolContent::Resource { uri, .. } => {
                    outputs.push(format!("Resource: {}", uri));
                }
            }
        }

        if result.is_error.unwrap_or(false) {
            Err(anyhow::anyhow!("Tool execution failed: {}", outputs.join("\n")))
        } else {
            Ok(outputs)
        }
    }

    pub fn get_ready_clients(&self) -> Vec<&str> {
        self.clients
            .iter()
            .filter(|(_, client)| client.is_ready())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    pub async fn execute_tool_calls(&mut self, tool_calls: &[crate::chat_service::ToolCall]) -> Vec<crate::chat_service::ToolResult> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            let result = match self.execute_tool(&tool_call.name, Some(tool_call.arguments.clone())).await {
                Ok(outputs) => crate::chat_service::ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    result: serde_json::to_value(outputs.join("\n")).unwrap_or(serde_json::Value::String("Tool executed successfully".to_string())),
                    error: None,
                },
                Err(e) => crate::chat_service::ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    result: serde_json::Value::Null,
                    error: Some(e.to_string()),
                },
            };
            results.push(result);
        }

        results
    }

    pub async fn list_available_tools(&mut self) -> Vec<crate::chat_service::Tool> {
        match self.list_all_tools().await {
            Ok(chat_tools) => {
                chat_tools.into_iter().map(|ct| crate::chat_service::Tool {
                    name: ct.name,
                    description: ct.description,
                    input_schema: ct.input_schema,
                    is_mcp: true,
                }).collect()
            },
            Err(_) => vec![],
        }
    }
}

impl Default for McpToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}