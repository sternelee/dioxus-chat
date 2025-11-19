// Rig-based Agent Service implementation (Temporarily using mock implementation)
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use futures::{Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::json;

// Temporarily mock rig types for compilation
#[async_trait::async_trait]
pub trait MockAgent: Send + Sync {
    async fn prompt(&self, message: &str) -> Result<String>;
}

pub struct MockAgentBuilder;
impl MockAgentBuilder {
    pub fn new(_model: &str) -> Self {
        Self
    }
    pub fn preamble(self, _preamble: &str) -> Self {
        self
    }
    pub fn tool<T>(self, _tool: T) -> Self {
        self
    }
    pub fn build(self) -> Box<dyn MockAgent> {
        Box::new(MockAgentImpl)
    }
}

pub struct MockAgentImpl;
#[async_trait::async_trait]
impl MockAgent for MockAgentImpl {
    async fn prompt(&self, message: &str) -> Result<String> {
        Ok(format!("Mock response to: {}", message))
    }
}

// Re-export types from chat_service_simple for compatibility
pub use crate::chat_service_simple::{
    AgentConfig, ChatMessage, ChatRequest, ChatResponse, GooseMode, Message, MessageContent,
    MessageMetadata, ModelConfig, Role, TokenUsage, Tool, ToolCall, ToolResult, StreamChunk,
    ModelPricing, ProviderError,
};

/// Enhanced model configuration with rig provider integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigModelConfig {
    pub base: ModelConfig,
    pub rig_provider: String,
    pub rig_model_id: String,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub api_key_env: Option<String>,
}

/// Custom tool trait for mock rig integration
#[async_trait::async_trait]
pub trait CustomTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn call(&self, args: serde_json::Value) -> Result<String>;
}

// Example custom tools
#[derive(Debug, Deserialize, Serialize)]
pub struct DateTimeTool;

#[async_trait::async_trait]
impl CustomTool for DateTimeTool {
    fn name(&self) -> &'static str {
        "get_current_time"
    }

    fn description(&self) -> &'static str {
        "Get the current date and time"
    }

    async fn call(&self, _args: serde_json::Value) -> Result<String> {
        Ok(Utc::now().to_rfc3339())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WeatherTool;

#[async_trait::async_trait]
impl CustomTool for WeatherTool {
    fn name(&self) -> &'static str {
        "get_weather"
    }

    fn description(&self) -> &'static str {
        "Get weather information for a location"
    }

    async fn call(&self, args: serde_json::Value) -> Result<String> {
        let location = args.get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown location");
        // Mock weather data
        Ok(format!("The weather in {} is sunny and 75°F", location))
    }
}

/// Main Rig-based Agent Service (Mock Implementation)
#[derive(Clone)]
pub struct RigAgentService {
    models: HashMap<String, RigModelConfig>,
    default_model: Option<String>,
    agents: Arc<RwLock<HashMap<String, Box<dyn MockAgent>>>>,
}

impl RigAgentService {
    pub fn new() -> Result<Self> {
        let mut models = HashMap::new();

        // Define model configurations
        let models_config = vec![
            RigModelConfig {
                base: ModelConfig {
                    id: "mock-local".to_string(),
                    name: "Mock Local Model".to_string(),
                    provider: "local".to_string(),
                    description: Some("A simple mock model for testing".to_string()),
                    context_limit: Some(4096),
                    supports_tools: false,
                    supports_streaming: false,
                    supports_vision: false,
                    supports_function_calling: false,
                    pricing: None,
                },
                rig_provider: "mock".to_string(),
                rig_model_id: "mock-local".to_string(),
                supports_tools: false,
                supports_streaming: false,
                api_key_env: None,
            },
            RigModelConfig {
                base: ModelConfig {
                    id: "openai/gpt-4o".to_string(),
                    name: "GPT-4o".to_string(),
                    provider: "openai".to_string(),
                    description: Some("OpenAI's flagship multimodal model".to_string()),
                    context_limit: Some(128000),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: true,
                    supports_function_calling: true,
                    pricing: Some(ModelPricing {
                        input_tokens: 0.005,
                        output_tokens: 0.015,
                        currency: "USD".to_string(),
                    }),
                },
                rig_provider: "openai".to_string(),
                rig_model_id: "gpt-4o".to_string(),
                supports_tools: true,
                supports_streaming: true,
                api_key_env: Some("OPENAI_API_KEY".to_string()),
            },
            RigModelConfig {
                base: ModelConfig {
                    id: "deepseek-chat".to_string(),
                    name: "DeepSeek Chat".to_string(),
                    provider: "deepseek".to_string(),
                    description: Some("DeepSeek's chat model optimized for conversations".to_string()),
                    context_limit: Some(64000),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: false,
                    supports_function_calling: true,
                    pricing: Some(ModelPricing {
                        input_tokens: 0.00014,
                        output_tokens: 0.00028,
                        currency: "USD".to_string(),
                    }),
                },
                rig_provider: "deepseek".to_string(),
                rig_model_id: "deepseek-chat".to_string(),
                supports_tools: true,
                supports_streaming: true,
                api_key_env: Some("DEEPSEEK_API_KEY".to_string()),
            },
            RigModelConfig {
                base: ModelConfig {
                    id: "anthropic/claude-3.5-sonnet".to_string(),
                    name: "Claude 3.5 Sonnet".to_string(),
                    provider: "anthropic".to_string(),
                    description: Some("Anthropic's most intelligent model".to_string()),
                    context_limit: Some(200000),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: true,
                    supports_function_calling: true,
                    pricing: Some(ModelPricing {
                        input_tokens: 0.003,
                        output_tokens: 0.015,
                        currency: "USD".to_string(),
                    }),
                },
                rig_provider: "anthropic".to_string(),
                rig_model_id: "claude-3-5-sonnet-20241022".to_string(),
                supports_tools: true,
                supports_streaming: true,
                api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            },
        ];

        for model in models_config {
            models.insert(model.base.id.clone(), model);
        }

        let default_model = Some("mock-local".to_string());

        Ok(Self {
            models,
            default_model,
            agents: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn get_available_models(&self) -> Vec<ModelConfig> {
        self.models.values().map(|m| m.base.clone()).collect()
    }

    async fn create_or_get_agent(&self, request: &ChatRequest) -> Result<String> {
        let model_id = if request.model.is_empty() {
            self.default_model
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No default model configured"))?
                .clone()
        } else {
            request.model.clone()
        };

        let agent_key = format!("{}:{}:{}",
            model_id,
            request.system_prompt.as_deref().unwrap_or(""),
            request.agent_config.as_ref().map(|c| format!("{:?}", c.goose_mode)).unwrap_or_default()
        );

        // Check if agent already exists
        {
            let agents = self.agents.read().await;
            if agents.contains_key(&agent_key) {
                return Ok(agent_key);
            }
        }

        // Create new agent based on model
        let model_config = self.models.get(&model_id)
            .ok_or_else(|| anyhow::anyhow!("Model {} not found", model_id))?;

        let agent: Box<dyn MockAgent> =
            match model_config.rig_provider.as_str() {
                "openai" | "anthropic" | "deepseek" => {
                    // For now, create a mock agent that simulates these providers
                    let mut builder = MockAgentBuilder::new(&model_config.rig_model_id);

                    // Add system prompt if provided
                    if let Some(ref prompt) = request.system_prompt {
                        builder = builder.preamble(prompt);
                    }

                    // Add tools if supported and requested
                    if model_config.supports_tools && request.tools.as_ref().map_or(true, |t| !t.is_empty()) {
                        builder = builder.tool(DateTimeTool).tool(WeatherTool);
                    }

                    // Configure agent settings
                    if let Some(ref agent_config) = request.agent_config {
                        match agent_config.goose_mode {
                            GooseMode::Agent => {
                                builder = builder.preamble("You are a helpful AI assistant with access to tools. Use the tools when they are helpful for answering the user's request.");
                            },
                            GooseMode::Chat => {
                                builder = builder.preamble("You are a helpful AI assistant focused on natural conversation.");
                            },
                            GooseMode::Auto => {
                                builder = builder.preamble("You are an autonomous AI assistant. Proactively help the user and use tools as needed.");
                            },
                        }
                    }

                    builder.build()
                },
                "mock" => {
                    // Create a mock agent for testing
                    MockAgentBuilder::new(&model_id).build()
                },
                _ => {
                    return Err(anyhow::anyhow!("Provider {} not yet implemented", model_config.rig_provider));
                }
            };

        // Store the agent
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_key.clone(), agent);
        }

        Ok(agent_key)
    }

  
    pub async fn send_message(&self, request: ChatRequest) -> Result<ChatResponse> {
        let agent_key = self.create_or_get_agent(&request).await?;

        // Get the last user message
        let user_message = request
            .messages
            .iter()
            .rev()
            .find(|msg| matches!(msg.role, Role::User))
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        let agents = self.agents.read().await;
        let agent = agents.get(&agent_key)
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        // Use rig agent to generate response
        let response = agent.prompt(&user_message).await?;

        // Calculate mock token usage
        let prompt_tokens = (user_message.len() + 3) / 4;
        let completion_tokens = (response.len() + 3) / 4;
        let total_tokens = prompt_tokens + completion_tokens;

        Ok(ChatResponse {
            message: Some(ChatMessage {
                role: Role::Assistant,
                content: response,
                timestamp: Some(Utc::now()),
                tool_calls: None,
                tool_results: None,
            }),
            tool_calls: None,
            token_usage: Some(TokenUsage {
                prompt_tokens: prompt_tokens as u32,
                completion_tokens: completion_tokens as u32,
                total_tokens: total_tokens as u32,
            }),
            model: request.model,
            finish_reason: Some("stop".to_string()),
            is_streaming: false,
            reasoning_content: None,
            thinking_content: None,
        })
    }

    pub async fn send_message_stream(&self, request: ChatRequest) -> Result<impl Stream<Item = StreamChunk>> {
        let agent_key = self.create_or_get_agent(&request).await?;

        // Get the last user message
        let user_message = request
            .messages
            .iter()
            .rev()
            .find(|msg| matches!(msg.role, Role::User))
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        let agents = self.agents.read().await;
        let agent = agents.get(&agent_key)
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        // Create a streaming response
        let model_id = request.model.clone();
        let mut response_text = String::new();

        // Simulate streaming by breaking the response into chunks
        match agent.prompt(&user_message).await {
            Ok(response) => {
                response_text = response;
            },
            Err(e) => return Err(anyhow::anyhow!("Failed to generate response: {}", e)),
        }

        let words: Vec<String> = response_text.split_whitespace().map(|s| s.to_string()).collect();
        let words_len = words.len();
        let model_id_clone = model_id;

        Ok(futures::stream::iter(words.into_iter().enumerate())
            .map(move |(index, word)| {
                let is_complete = index == words_len - 1;
                StreamChunk {
                    content: Some(format!("{} ", word)),
                    delta: Some(format!("{} ", word)),
                    token_usage: None,
                    model: model_id_clone.clone(),
                    finish_reason: if is_complete { Some("stop".to_string()) } else { None },
                    is_complete,
                }
            }))
    }

    pub async fn list_tools(&self, model: &str) -> Vec<Tool> {
        let mut tools = vec![];

        if let Some(model_config) = self.models.get(model) {
            if model_config.supports_tools {
                tools.push(Tool {
                    name: "get_current_time".to_string(),
                    description: "Get the current date and time".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {},
                    }),
                    is_mcp: false,
                });

                tools.push(Tool {
                    name: "get_weather".to_string(),
                    description: "Get weather information for a location".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The location to get weather for"
                            }
                        },
                        "required": ["location"],
                    }),
                    is_mcp: false,
                });
            }
        }

        tools
    }

    /// Send a message with streaming response using Server-Sent Events format
    pub async fn send_message_sse(&self, request: ChatRequest) -> Result<impl Stream<Item = String>> {
        let stream = self.send_message_stream(request).await?;

        let sse_stream = stream.map(|chunk| {
            serde_json::to_string(&chunk).unwrap_or_default()
        });

        Ok(sse_stream)
    }
}

// Default implementation for compatibility
impl Default for RigAgentService {
    fn default() -> Self {
        Self::new().expect("Failed to create RigAgentService")
    }
}