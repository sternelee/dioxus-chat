//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

pub mod chat_service;
pub mod mcp;
pub mod providers;

// Core types for AI providers, chat, and tools
pub use chat_service::{
    // Agent config types (for UI to pass as parameters)
    AgentConfig,
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ChatService,
    GooseMode,
    Message,
    MessageContent,
    MessageMetadata,
    ModelConfig,
    Role,
    TokenUsage,
    Tool,
    Usage,
};

pub use mcp::{
    create_builtin_tools, create_default_mcp_executor, execute_builtin_tool, protocol::*,
    McpClient, McpToolExecutor, StdioMcpClient,
};
pub use providers::{
    anthropic::AnthropicProvider, local::LocalProvider, ollama::OllamaProvider,
    openai::OpenAIProvider, CompletionRequest, CompletionResponse, Provider, ProviderRegistry,
};

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

/// Get available models from all registered providers
#[post("/api/models")]
pub async fn get_available_models() -> Result<Vec<ModelConfig>, ServerFnError> {
    let service = ChatService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create chat service: {}", e)))?;
    let models = service
        .get_available_models()
        .into_iter()
        .cloned()
        .collect();
    Ok(models)
}

/// Send a chat message (streaming chat response)
#[post("/api/chat")]
pub async fn send_message(request: ChatRequest) -> Result<ChatResponse, ServerFnError> {
    let service = ChatService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create chat service: {}", e)))?;
    let response = service
        .send_message(request)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to send message: {}", e)))?;
    Ok(response)
}

/// Get available tools for a specific model
#[post("/api/tools")]
pub async fn get_tools(model: String) -> Result<Vec<Tool>, ServerFnError> {
    let service = ChatService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create chat service: {}", e)))?;
    let tools = service.list_tools(&model).await;
    Ok(tools)
}


