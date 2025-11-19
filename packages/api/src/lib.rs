//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

// Include chat service modules
pub mod agent_builder;
pub mod chat_service_simple;
pub mod rig_agent_service;
pub mod streaming_service;

// Temporarily comment out advanced modules that have compilation issues
// pub mod mcp_tools;
// pub mod multimodal;
// pub mod agent_extensions;
// pub mod rag_system;

// Export types from chat_service_simple for backward compatibility
pub use chat_service_simple::{
    AgentConfig, ChatMessage, ChatRequest, ChatResponse, GooseMode, Message, MessageContent,
    MessageMetadata, ModelConfig, ModelPricing, ProviderError, Role,
    SimpleChatService as ChatService, StreamChunk, TokenUsage, Tool, ToolCall, ToolResult,
};

// Export new rig-based agent services
pub use agent_builder::{AgentBuilderConfig, AgentFactory, RigAgentBuilder, ToolRegistry};
pub use rig_agent_service::{CustomTool, RigAgentService, RigModelConfig};
pub use streaming_service::{
    ChunkType, EnhancedStreamChunk, StreamMetadata, StreamingAgentService, StreamingConfig,
};

// Temporarily comment out advanced feature exports to focus on core functionality
// pub use mcp_tools::{McpToolRegistry, McpServerConfig, McpClient, EnhancedRigAgentService as MCPEnabledAgentService};
// pub use multimodal::{
//     MultimodalService, MediaContent, MediaData, MediaType, MultimodalMessage, MultimodalConfig,
//     MultimodalChatRequest, MultimodalRigAgentService, VisionAnalysisTool, SpeechToTextTool, DocumentProcessorTool
// };
// pub use agent_extensions::{
//     AgentExtension, ExtensionManager, ExtensionContext, ExtensionResult, ExtensionPhase,
//     ExtensionInfo, ExtendedRigAgentService, ConversationSummarizerExtension, ToolUsageMonitorExtension,
//     SafetyFilterExtension
// };
// pub use rag_system::{
//     VectorStore, VectorEmbedding, DocumentMetadata, SearchQuery, SearchResult,
//     DocumentChunk, InMemoryVectorStore, EmbeddingService, DocumentProcessor, MockEmbeddingService,
//     RAGSystem, RAGApiTool as RAGTool, KnowledgeBaseApiTool as KnowledgeBaseTool, RAGStatistics, RAGEnabledAgentService
// };

// Core traits for extensibility
use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
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
        self.send_message(request)
            .await
            .map(|response| serde_json::to_string(&response).unwrap_or_default())
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
        vec![
            "chat".to_string(),
            "streaming".to_string(),
            "thinking".to_string(),
            "tools".to_string(),
        ]
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
    let service = RigAgentService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create rig agent service: {}", e)))?;
    Ok(service.get_available_models())
}

/// Send a chat message (using rig agent service)
#[post("/api/chat")]
pub async fn send_message(request: ChatRequest) -> Result<ChatResponse, ServerFnError> {
    let service = RigAgentService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create rig agent service: {}", e)))?;
    let response = service
        .send_message(request)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to send message: {}", e)))?;
    Ok(response)
}

/// Send a chat message with streaming response using enhanced streaming service
#[post("/api/chat/stream")]
pub async fn send_message_stream(request: ChatRequest) -> Result<String, ServerFnError> {
    let agent_service = RigAgentService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create rig agent service: {}", e)))?;
    let streaming_service = StreamingAgentService::new(agent_service);

    // Create a stream for real-time updates using enhanced streaming
    let stream = streaming_service
        .stream_chat_response(request)
        .await
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
        if let Some(content) = chunk.base.content {
            full_content.push_str(&content);
        }
        if chunk.base.token_usage.is_some() {
            final_usage = chunk.base.token_usage;
        }
        if chunk.base.finish_reason.is_some() {
            finish_reason = chunk.base.finish_reason;
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
        thinking_content: None,
    };

    serde_json::to_string(&response)
        .map_err(|e| ServerFnError::new(format!("Failed to serialize response: {}", e)))
}

/// Get available tools for a specific model
#[post("/api/tools")]
pub async fn get_tools(model: String) -> Result<Vec<Tool>, ServerFnError> {
    let service = RigAgentService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create rig agent service: {}", e)))?;
    let tools = service.list_tools(&model).await;
    Ok(tools)
}

// Additional API endpoints for enhanced agent functionality

/// Create a specialized agent with custom configuration
#[post("/api/agents/create")]
pub async fn create_agent(config: AgentBuilderConfig) -> Result<String, ServerFnError> {
    let builder = RigAgentBuilder::new(config);

    // For now, just return a success message
    // In a full implementation, this would store the agent and return an ID
    Ok("Agent created successfully".to_string())
}

/// Get available agent types
#[post("/api/agents/types")]
pub async fn get_agent_types() -> Result<Vec<String>, ServerFnError> {
    Ok(vec![
        "conversational".to_string(),
        "tool_agent".to_string(),
        "autonomous".to_string(),
        "programming".to_string(),
        "research".to_string(),
        "creative".to_string(),
        "analysis".to_string(),
    ])
}

/// Stream chat with enhanced features including tool visualization
#[post("/api/chat/stream/enhanced")]
pub async fn send_message_enhanced_stream(request: ChatRequest) -> Result<String, ServerFnError> {
    let agent_service = RigAgentService::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create rig agent service: {}", e)))?;
    let streaming_service = StreamingAgentService::new(agent_service);

    // Create an enhanced stream with tool visualization
    let stream = streaming_service
        .stream_chat_with_tools(request)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create enhanced stream: {}", e)))?;

    // For now, collect the stream and return the complete response
    use futures::StreamExt;
    let chunks: Vec<_> = stream.collect().await;

    // Combine all content chunks into a single response
    let mut full_content = String::new();
    let mut metadata_chunks = Vec::new();

    for chunk in chunks {
        match chunk.chunk_type {
            ChunkType::Content | ChunkType::Thinking => {
                if let Some(content) = chunk.base.content {
                    full_content.push_str(&content);
                }
            }
            ChunkType::Metadata => {
                metadata_chunks.push(chunk);
            }
            _ => {}
        }
    }

    // Create response including metadata
    let response = serde_json::json!({
        "message": {
            "role": "assistant",
            "content": full_content,
            "timestamp": chrono::Utc::now(),
        },
        "metadata": metadata_chunks.into_iter().map(|c| c.metadata).collect::<Vec<_>>(),
        "is_streaming": false,
        "model": "enhanced_agent",
    });

    serde_json::to_string(&response)
        .map_err(|e| ServerFnError::new(format!("Failed to serialize response: {}", e)))
}
