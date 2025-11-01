use super::client::{McpToolExecutor, StdioMcpClient};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, warn};

pub fn create_default_mcp_executor() -> Result<McpToolExecutor> {
    let mut executor = McpToolExecutor::new();

    // Add common MCP servers if available
    let common_servers = get_common_mcp_servers();

    for (name, config) in common_servers {
        let client = StdioMcpClient::new(name.clone(), config.command.clone(), config.args.clone());
        executor.add_client(Box::new(client));
        info!("Added MCP server: {}", name);
    }

    Ok(executor)
}

pub fn get_common_mcp_servers() -> HashMap<String, McpServerConfig> {
    let mut servers = HashMap::new();

    // File system tools
    servers.insert(
        "filesystem".to_string(),
        McpServerConfig {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                "/tmp".to_string(), // Default to temp directory
            ],
        },
    );

    // GitHub tools (if GitHub CLI is available)
    if std::process::Command::new("gh").output().is_ok() {
        servers.insert(
            "github".to_string(),
            McpServerConfig {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-github".to_string(),
                ],
            },
        );
    }

    // SQLite tools
    servers.insert(
        "sqlite".to_string(),
        McpServerConfig {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-sqlite".to_string(),
            ],
        },
    );

    // Brave Search (if API key is available)
    if std::env::var("BRAVE_API_KEY").is_ok() {
        servers.insert(
            "brave-search".to_string(),
            McpServerConfig {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-brave-search".to_string(),
                ],
            },
        );
    }

    // Memory tools
    servers.insert(
        "memory".to_string(),
        McpServerConfig {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-memory".to_string(),
            ],
        },
    );

    servers
}

#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

pub fn create_custom_mcp_client(
    name: String,
    command: String,
    args: Vec<String>,
) -> Box<dyn super::client::McpClient> {
    let client = StdioMcpClient::new(name, command, args);
    Box::new(client)
}

