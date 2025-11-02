//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

// Only include the simplified chat service for now
pub mod chat_service_simple;

// Export only the essential types from chat_service module
pub use chat_service_simple::{
    SimpleChatService as ChatService, ChatRequest, ChatResponse, ChatMessage, ModelConfig, Role, TokenUsage,
    AgentConfig, GooseMode, Message, MessageContent, MessageMetadata, Tool, ToolCall, ToolResult,
};

// Core traits for extensibility
use async_trait::async_trait;
use futures::Stream;
use anyhow::Result;
use std::sync::Arc;

/// Provider trait for different AI providers
#[async_trait]
pub trait ChatProvider: Send + Sync {
    /// Send a message and get a streaming response
    async fn send_message_stream(&self, request: ChatRequest) -> Result<String>;

    /// Get available models from this provider
    async fn list_models(&self) -> Result<Vec<ModelConfig>>;

    /// Get the currently active model name
    fn get_active_model_name(&self) -> String;

    /// Check if this provider supports a specific feature
    fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "streaming" => true,
            "tools" => true,
            "thinking" => self.supports_thinking(),
            _ => false,
        }
    }

    /// Check if this provider supports thinking/reasoning content
    fn supports_thinking(&self) -> bool {
        false
    }

    /// Get provider capabilities
    fn capabilities(&self) -> Vec<String> {
        vec!["chat".to_string(), "streaming".to_string()]
    }
}

/// Default implementation for our SimpleChatService
#[async_trait]
impl ChatProvider for ChatService {
    async fn send_message_stream(&self, request: ChatRequest) -> Result<String> {
        self.send_message(request).await.map(|response| {
            serde_json::to_string(&response).unwrap_or_default()
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelConfig>> {
        self.list_models().await
    }

    fn get_active_model_name(&self) -> String {
        "default-model".to_string()
    }

    fn supports_thinking(&self) -> bool {
        true // Our SimpleChatService supports thinking
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["chat".to_string(), "streaming".to_string(), "thinking".to_string(), "tools".to_string()]
    }
}

/// Provider factory for creating different providers
pub struct ProviderFactory;

impl ProviderFactory {
    pub async fn create_default_provider() -> Result<Arc<dyn ChatProvider>> {
        let service = ChatService::new()?;
        Ok(Arc::new(service))
    }

    pub async fn create_deepseek_provider(api_key: &str) -> Result<Arc<dyn ChatProvider>> {
        let service = ChatService::new()?;
        // Note: For now we just return the default service
        // In a real implementation, this would configure DeepSeek-specific settings
        Ok(Arc::new(service))
    }

    pub async fn create_openrouter_provider(api_key: &str) -> Result<Arc<dyn ChatProvider>> {
        let service = ChatService::new()?;
        // Note: For now we just return the default service
        // In a real implementation, this would configure OpenRouter-specific settings
        Ok(Arc::new(service))
    }
}

// Note: MCP and providers modules are temporarily disabled to avoid compilation issues
// They can be re-enabled once the compilation errors are fixed
/*
pub use mcp::{
    create_builtin_tools, create_default_mcp_executor, execute_builtin_tool, protocol::*,
    McpClient, McpToolExecutor, StdioMcpClient,
};
pub use providers::{
    anthropic::AnthropicProvider, local::LocalProvider, ollama::OllamaProvider,
    openai::OpenAIProvider, CompletionRequest, CompletionResponse, Provider, ProviderRegistry,
};
*/

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

/// Send a chat message with streaming response using Server-Sent Events
#[post("/api/chat/stream")]
pub async fn send_message_stream(request: ChatRequest) -> Result<String, ServerFnError> {
    let service = ChatService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create chat service: {}", e)))?;

    // Create a stream for real-time updates
    let stream = service.send_message_stream(request).await
        .map_err(|e| ServerFnError::new(format!("Failed to create stream: {}", e)))?;

    // For now, collect the stream and return the complete response
    // In a full implementation, we'd return the stream directly
    use futures::StreamExt;
    let chunks: Vec<_> = stream.collect().await;

    // Combine all chunks into a single response
    let mut full_content = String::new();
    let mut final_usage = None;
    let mut finish_reason = None;

    for chunk in chunks {
        if let Some(content) = chunk.content {
            full_content.push_str(&content);
        }
        if chunk.token_usage.is_some() {
            final_usage = chunk.token_usage;
        }
        if chunk.finish_reason.is_some() {
            finish_reason = chunk.finish_reason;
        }
    }

    // Create a response that includes all the streaming content
    let response = ChatResponse {
        message: Some(ChatMessage {
            role: Role::Assistant,
            content: full_content,
            timestamp: Some(chrono::Utc::now()),
            tool_calls: None,
            tool_results: None,
        }),
        tool_calls: None,
        token_usage: final_usage,
        model: request.model,
        finish_reason,
        is_streaming: false, // We've collected the full response
        reasoning_content: None,
    };

    serde_json::to_string(&response)
        .map_err(|e| ServerFnError::new(format!("Failed to serialize response: {}", e)))
}

/// Get available tools for a specific model
#[post("/api/tools")]
pub async fn get_tools(model: String) -> Result<Vec<Tool>, ServerFnError> {
    let service = ChatService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create chat service: {}", e)))?;
    let tools = service.list_tools(&model).await;
    Ok(tools)
}


