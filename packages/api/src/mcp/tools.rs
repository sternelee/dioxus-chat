use super::client::{McpClient, McpToolExecutor};
use super::protocol::Tool;
use crate::chat_service::{Tool as ChatTool, ToolCall};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

// Built-in tools that are always available
pub fn create_builtin_tools() -> Vec<ChatTool> {
    vec![
        ChatTool {
            name: "shell".to_string(),
            description: "Execute shell commands. Use with caution and only when necessary."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "working_directory": {
                        "type": "string",
                        "description": "Optional working directory for the command"
                    }
                },
                "required": ["command"]
            }),
        },
        ChatTool {
            name: "file_editor".to_string(),
            description: "Read, write, edit, and search files on the filesystem".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "edit", "search", "list"],
                        "description": "The file operation to perform"
                    },
                    "path": {
                        "type": "string",
                        "description": "File or directory path"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content for write/edit operations"
                    },
                    "search_term": {
                        "type": "string",
                        "description": "Search term for search operations"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Pattern for listing files (glob pattern)"
                    }
                },
                "required": ["operation", "path"]
            }),
        },
        ChatTool {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        },
        ChatTool {
            name: "analyze_code".to_string(),
            description: "Analyze and understand code files".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the code file to analyze"
                    },
                    "analysis_type": {
                        "type": "string",
                        "enum": ["syntax", "structure", "security", "performance"],
                        "description": "Type of analysis to perform"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ChatTool {
            name: "system_info".to_string(),
            description: "Get system information and status".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "info_type": {
                        "type": "string",
                        "enum": ["os", "memory", "disk", "processes", "network"],
                        "description": "Type of system information to retrieve"
                    }
                },
                "required": ["info_type"]
            }),
        },
    ]
}

pub async fn execute_builtin_tool(tool_call: &ToolCall) -> Result<Vec<String>> {
    match tool_call.name.as_str() {
        "shell" => execute_shell_command(tool_call).await,
        "file_editor" => execute_file_operation(tool_call).await,
        "web_search" => execute_web_search(tool_call).await,
        "analyze_code" => execute_code_analysis(tool_call).await,
        "system_info" => execute_system_info(tool_call).await,
        _ => Err(anyhow::anyhow!("Unknown built-in tool: {}", tool_call.name)),
    }
}

async fn execute_shell_command(tool_call: &ToolCall) -> Result<Vec<String>> {
    let command = tool_call
        .arguments
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

    let working_dir = tool_call
        .arguments
        .get("working_directory")
        .and_then(|v| v.as_str());

    info!("Executing shell command: {}", command);

    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c").arg(command);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;

    let mut results = Vec::new();

    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        results.push(stdout.to_string());
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        results.push(format!("stderr: {}", stderr));
    }

    results.push(format!("Exit code: {}", output.status));

    if !output.status.success() {
        warn!("Shell command failed: {}", command);
    }

    Ok(results)
}

async fn execute_file_operation(tool_call: &ToolCall) -> Result<Vec<String>> {
    let operation = tool_call
        .arguments
        .get("operation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'operation' parameter"))?;

    let path = tool_call
        .arguments
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

    debug!("File operation: {} on {}", operation, path);

    match operation {
        "read" => {
            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))?;
            Ok(vec![content])
        }
        "write" => {
            let content = tool_call
                .arguments
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!("Missing 'content' parameter for write operation")
                })?;

            tokio::fs::write(path, content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;

            Ok(vec![format!("Successfully wrote to file: {}", path)])
        }
        "edit" => {
            let content = tool_call
                .arguments
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter for edit operation"))?;

            tokio::fs::write(path, content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to edit file '{}': {}", path, e))?;

            Ok(vec![format!("Successfully edited file: {}", path)])
        }
        "search" => {
            let search_term = tool_call
                .arguments
                .get("search_term")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'search_term' parameter"))?;

            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read file for search '{}': {}", path, e))?;

            let lines: Vec<&str> = content.lines().collect();
            let mut matches = Vec::new();

            for (line_num, line) in lines.iter().enumerate() {
                if line.contains(search_term) {
                    matches.push(format!("Line {}: {}", line_num + 1, line));
                }
            }

            if matches.is_empty() {
                Ok(vec![format!(
                    "No matches found for '{}' in {}",
                    search_term, path
                )])
            } else {
                matches.insert(0, format!("Found {} matches in {}:", matches.len(), path));
                Ok(matches)
            }
        }
        "list" => {
            let pattern = tool_call
                .arguments
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("*");

            let mut entries = tokio::fs::read_dir(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read directory '{}': {}", path, e))?;

            let mut results = vec![format!("Contents of {} (pattern: {}):", path, pattern)];
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?
            {
                let file_name_owned = entry.file_name();
                let file_name = file_name_owned.to_string_lossy();
                if matches_pattern(&file_name, pattern) {
                    let file_type = if entry
                        .file_type()
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to get file type: {}", e))?
                        .is_dir()
                    {
                        "DIR"
                    } else {
                        "FILE"
                    };
                    results.push(format!("  {} {}", file_type, file_name));
                }
            }

            if results.len() == 1 {
                results.push("  (no matching files)".to_string());
            }

            Ok(results)
        }
        _ => Err(anyhow::anyhow!("Unsupported file operation: {}", operation)),
    }
}

async fn execute_web_search(tool_call: &ToolCall) -> Result<Vec<String>> {
    let query = tool_call
        .arguments
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

    // For now, return a mock result
    // In a real implementation, you'd use a search API
    warn!(
        "Web search not implemented - returning mock results for query: {}",
        query
    );

    Ok(vec![
        format!("Web search results for: {}", query),
        "1. Mock search result 1".to_string(),
        "2. Mock search result 2".to_string(),
        "3. Mock search result 3".to_string(),
        "(Note: Actual web search not implemented - this is a mock result)".to_string(),
    ])
}

async fn execute_code_analysis(tool_call: &ToolCall) -> Result<Vec<String>> {
    let file_path = tool_call
        .arguments
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'file_path' parameter"))?;

    let analysis_type = tool_call
        .arguments
        .get("analysis_type")
        .and_then(|v| v.as_str())
        .unwrap_or("syntax");

    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))?;

    let mut results = vec![format!(
        "Code analysis for {} ({})",
        file_path, analysis_type
    )];

    match analysis_type {
        "syntax" => {
            let lines = content.lines().count();
            let chars = content.chars().count();
            results.push(format!("Lines of code: {}", lines));
            results.push(format!("Total characters: {}", chars));

            // Simple syntax checks
            let open_braces = content.matches('{').count();
            let close_braces = content.matches('}').count();
            let open_parens = content.matches('(').count();
            let close_parens = content.matches(')').count();

            results.push(format!(
                "Brace balance: {} open, {} close",
                open_braces, close_braces
            ));
            results.push(format!(
                "Parentheses balance: {} open, {} close",
                open_parens, close_parens
            ));
        }
        "structure" => {
            // Simple structure analysis
            let functions = content.matches("fn ").count() + content.matches("function ").count();
            let classes = content.matches("class ").count() + content.matches("struct ").count();
            let imports = content.matches("use ").count() + content.matches("import ").count();

            results.push(format!("Functions/Methods: {}", functions));
            results.push(format!("Classes/Structs: {}", classes));
            results.push(format!("Imports/Uses: {}", imports));
        }
        _ => {
            results.push(format!("Analysis type '{}' not implemented", analysis_type));
        }
    }

    Ok(results)
}

async fn execute_system_info(tool_call: &ToolCall) -> Result<Vec<String>> {
    let info_type = tool_call
        .arguments
        .get("info_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'info_type' parameter"))?;

    let mut results = Vec::new();

    match info_type {
        "os" => {
            results.push("System Information:".to_string());
            results.push(format!("OS: {}", std::env::consts::OS));
            results.push(format!("Arch: {}", std::env::consts::ARCH));
            results.push(format!("Family: {}", std::env::consts::FAMILY));
        }
        "memory" => {
            if let Ok(output) = tokio::process::Command::new("free")
                .arg("-h")
                .output()
                .await
            {
                let memory_info = String::from_utf8_lossy(&output.stdout);
                results.push("Memory Information:".to_string());
                for line in memory_info.lines() {
                    results.push(line.to_string());
                }
            } else {
                results.push("Memory info not available on this platform".to_string());
            }
        }
        "disk" => {
            if let Ok(output) = tokio::process::Command::new("df").arg("-h").output().await {
                let disk_info = String::from_utf8_lossy(&output.stdout);
                results.push("Disk Information:".to_string());
                for line in disk_info.lines() {
                    results.push(line.to_string());
                }
            } else {
                results.push("Disk info not available on this platform".to_string());
            }
        }
        "processes" => {
            if let Ok(output) = tokio::process::Command::new("ps")
                .args(&["aux"])
                .output()
                .await
            {
                let process_info = String::from_utf8_lossy(&output.stdout);
                results.push("Process Information (top 10):".to_string());
                for line in process_info.lines().take(11) {
                    results.push(line.to_string());
                }
            } else {
                results.push("Process info not available on this platform".to_string());
            }
        }
        _ => {
            results.push(format!("Unknown info type: {}", info_type));
        }
    }

    Ok(results)
}

fn matches_pattern(text: &str, pattern: &str) -> bool {
    // Simple glob pattern matching - for demonstration purposes
    if pattern == "*" {
        return true;
    }

    // Convert glob to regex (very basic implementation)
    let regex_pattern = pattern
        .replace('.', r"\.")
        .replace('*', ".*")
        .replace('?', ".");

    regex::Regex::new(&format!("^{}$", regex_pattern))
        .map(|r| r.is_match(text))
        .unwrap_or(false)
}

