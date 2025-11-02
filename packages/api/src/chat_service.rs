use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

// Import provider and MCP systems
use super::mcp::{create_default_mcp_executor, McpToolExecutor};
use super::providers::{CompletionRequest, Provider, ProviderRegistry};

// Core message types

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_request")]
    ToolRequest {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename = "tool_response")]
    ToolResponse {
        id: String,
        name: String,
        result: serde_json::Value,
    },
    #[serde(rename = "image")]
    Image {
        url: String,
        description: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: MessageContent,
    pub timestamp: Option<DateTime<Utc>>,
    pub metadata: Option<MessageMetadata>,
}

impl Message {
    pub fn as_concat_text(&self) -> String {
        match &self.content {
            MessageContent::Text { text } => text.clone(),
            MessageContent::ToolRequest { name, arguments, .. } => {
                format!("Tool call: {}({})", name, arguments)
            }
            MessageContent::ToolResponse { result, .. } => {
                format!("Tool result: {}", result)
            }
            MessageContent::Image { url, description } => {
                format!("[Image: {}] {}", url, description.as_ref().unwrap_or(&String::new()))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageMetadata {
    pub model: Option<String>,
    pub token_usage: Option<TokenUsage>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub reasoning_content: Option<String>,
    pub is_streaming: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conversation(pub Vec<Message>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub context_limit: Option<usize>,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub pricing: Option<ModelPricing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub name: String,
    pub version: Option<String>,
    pub api_base: Option<String>,
    pub supported_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderError {
    pub message: String,
    pub code: Option<String>,
    pub retry_after: Option<u64>,
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProviderError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub is_mcp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
}


// Agent configuration types (for UI to pass as parameters)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    pub max_iterations: usize,
    pub require_confirmation: bool,
    pub readonly_tools: Vec<String>,
    pub enable_tool_inspection: bool,
    pub enable_auto_compact: bool,
    pub compact_threshold: f32,
    pub max_turns_without_tools: usize,
    pub enable_autopilot: bool,
    pub enable_extensions: bool,
    pub extension_timeout: u64,
    pub goose_mode: GooseMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GooseMode {
    Chat,
    Agent,
    Auto,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            require_confirmation: false,
            readonly_tools: vec![],
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.8,
            max_turns_without_tools: 3,
            enable_autopilot: false,
            enable_extensions: true,
            extension_timeout: 30,
            goose_mode: GooseMode::Agent,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub currency: String,
}

// Chat request and response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stream: bool,
    pub agent_config: Option<AgentConfig>,
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Option<ChatMessage>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub token_usage: Option<TokenUsage>,
    pub model: String,
    pub finish_reason: Option<String>,
    pub is_streaming: bool,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// Simplified ChatService - focuses on AI providers, streaming, and MCP tools
pub struct ChatService {
    providers: ProviderRegistry,
    models: HashMap<String, ModelConfig>,
    default_model: Option<String>,
    mcp_executor: Arc<RwLock<McpToolExecutor>>,
}

impl ChatService {
    pub fn new() -> Result<Self> {
        let mut registry = ProviderRegistry::new();

        // Register default providers
        // Note: For now, we'll create a simple local provider as fallback
        let local_config = ModelConfig {
            id: "local-default".to_string(),
            name: "Local Default".to_string(),
            provider: "local".to_string(),
            description: Some("Default local provider".to_string()),
            context_limit: Some(4096),
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            supports_function_calling: true,
            pricing: None,
        };
        registry.register(Box::new(
            super::providers::local::LocalProvider::from_config(local_config)?,
        ))?;

        let mut models = HashMap::new();

        // Load models from providers
        for provider in registry.list_providers() {
            // Note: This is a simplified approach. In a real implementation,
            // you'd need to get actual provider instances and call list_models on them
            // For now, we'll just use the default local model
            let local_model = ModelConfig {
                id: "local-default".to_string(),
                name: "Local Default Model".to_string(),
                provider: "local".to_string(),
                description: Some("Default local model for testing".to_string()),
                context_limit: Some(4096),
                supports_tools: false,
                supports_streaming: false,
                supports_vision: false,
                supports_function_calling: false,
                pricing: None,
            };
            models.insert(local_model.id.clone(), local_model);
            break; // Just use the first one for now
        }

        let default_model = models.keys().next().cloned();

        // Initialize MCP executor
        let mcp_executor = Arc::new(RwLock::new(create_default_mcp_executor().unwrap_or_else(|_| {
            warn!("Failed to create MCP executor, using empty one");
            McpToolExecutor::new()
        })));

        Ok(Self {
            providers: registry,
            models,
            default_model,
            mcp_executor,
        })
    }

    pub fn get_available_models(&self) -> Vec<ModelConfig> {
        self.models.values().cloned().collect()
    }

    pub async fn send_message(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model_id = if request.model.is_empty() {
            self.default_model
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No default model configured"))?
                .clone()
        } else {
            request.model.clone()
        };

        let provider = self
            .providers
            .get_provider_for_model(&model_id)
            .ok_or_else(|| anyhow::anyhow!("No provider found for model: {}", model_id))?;

        let completion_request = CompletionRequest {
            messages: request
                .messages
                .into_iter()
                .map(|msg| Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: msg.role,
                    content: MessageContent::Text { text: msg.content },
                    timestamp: msg.timestamp,
                    metadata: None,
                })
                .collect(),
            system: request.system_prompt,
            tools: request.tools,
            temperature: request.temperature,
            max_tokens: request.max_tokens.map(|t| t as u32),
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: None,
            stream: request.stream,
        };

        let mut response = provider.complete(completion_request).await?;

        // Process MCP tool calls if any
        if let Some(tool_calls) = &response.tool_calls {
            let converted_tool_calls: Vec<crate::chat_service::ToolCall> = tool_calls
                .iter()
                .map(|tc| crate::chat_service::ToolCall {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    arguments: tc.arguments.clone(),
                })
                .collect();

            let mut executor = self.mcp_executor.write().await;
            let tool_results = executor.execute_tool_calls(&converted_tool_calls).await;

            // Add tool results to response
            response.tool_results = Some(tool_results);
        }

        // Convert to ChatResponse
        let converted_tool_calls: Option<Vec<crate::chat_service::ToolCall>> = response.tool_calls.as_ref().map(|tool_calls| {
            tool_calls.iter().map(|tc| crate::chat_service::ToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arguments: tc.arguments.clone(),
            }).collect()
        });

        Ok(ChatResponse {
            message: Some(ChatMessage {
                role: Role::Assistant,
                content: response.content,
                timestamp: Some(Utc::now()),
                tool_calls: converted_tool_calls.clone(),
                tool_results: response.tool_results,
            }),
            tool_calls: converted_tool_calls,
            token_usage: Some(TokenUsage {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            }),
            model: model_id.clone(),
            finish_reason: response.finish_reason,
            is_streaming: request.stream,
            reasoning_content: response.reasoning_content,
        })
    }

    pub async fn list_tools(&self, model: &str) -> Vec<Tool> {
        // Get provider tools
        if let Some(provider) = self.providers.get_provider_for_model(model) {
            let mut tools = provider.list_tools().await;

            // Add MCP tools - temporarily disabled until MCP issues are fixed
            // let executor = self.mcp_executor.read().await;
            // let mcp_tools = executor.list_available_tools().await;
            // tools.extend(mcp_tools);

            tools
        } else {
            vec![]
        }
    }
}

