use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export from API module with rig integration
pub use api::{
    AgentConfig, AgentFactory, ChatMessage, ChatRequest, ChatResponse, ChunkType,
    EnhancedStreamChunk, GooseMode, MessageMetadata, ModelConfig, RigAgentService, Role,
    StreamMetadata, StreamingAgentService, TokenUsage, Tool, ToolCall, ToolResult,
};

// Simplified MessageContent for UI usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageContent {
    Text(String),
    Thinking(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
}

impl MessageContent {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(text) => Some(text),
            _ => None,
        }
    }
}

/// Agent event types for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    Message(ChatMessage),
    Token(TokenUsage),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
    Error(String),
    Done,
}

/// Simplified ChatMessage for UI usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiChatMessage {
    pub role: Role,
    pub content: MessageContent,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
    pub metadata: Option<MessageMetadata>,
}

impl From<ChatMessage> for UiChatMessage {
    fn from(msg: ChatMessage) -> Self {
        Self {
            role: msg.role,
            content: MessageContent::Text(msg.content),
            timestamp: msg.timestamp,
            tool_calls: msg.tool_calls,
            tool_results: msg.tool_results,
            metadata: msg.metadata,
        }
    }
}

/// Conversation state and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<UiChatMessage>,
    pub metadata: ConversationMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub title: Option<String>,
    pub model: Option<String>,
    pub token_usage: TokenUsage,
    pub agent_mode: Option<GooseMode>,
    pub extensions: Vec<String>,
}

impl Conversation {
    pub fn new(messages: Vec<UiChatMessage>) -> Result<Self> {
        let now = chrono::Utc::now();
        let metadata = ConversationMetadata {
            title: None,
            model: None,
            token_usage: TokenUsage::default(),
            agent_mode: None,
            extensions: vec![],
        };

        Ok(Self {
            id: format!("conv_{}", now.timestamp_nanos()),
            messages,
            metadata,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn add_message(&mut self, message: UiChatMessage) {
        self.updated_at = chrono::Utc::now();
        self.messages.push(message);
    }

    pub fn last_message(&self) -> Option<&UiChatMessage> {
        self.messages.last()
    }

    pub fn get_title(&self) -> String {
        self.metadata.title.clone().unwrap_or_else(|| {
            // Generate title from first user message
            self.messages
                .iter()
                .find(|msg| matches!(msg.role, Role::User))
                .and_then(|msg| msg.content.as_text())
                .map(|content| {
                    let truncated = content.chars().take(50).collect::<String>();
                    if content.len() > 50 {
                        format!("{}...", truncated)
                    } else {
                        truncated
                    }
                })
                .unwrap_or_else(|| "New Chat".to_string())
        })
    }
}

/// Core Agent trait for extensibility with rig integration
#[async_trait]
pub trait Agent: Send + Sync {
    /// Process a conversation and return a stream of events
    async fn reply(
        &self,
        conversation: Conversation,
        system_prompt: Option<&str>,
        tools: Option<&[Tool]>,
    ) -> Result<Box<dyn futures::Stream<Item = Result<AgentEvent>> + Send + Unpin>>;

    /// Get the agent's configuration
    fn config(&self) -> &AgentConfig;

    /// Update agent configuration
    async fn update_config(&mut self, config: AgentConfig) -> Result<()>;

    /// List available extensions
    async fn list_extensions(&self) -> Vec<String>;

    /// Add an extension to the agent
    async fn add_extension(&mut self, extension: Box<dyn AgentExtension>) -> Result<()>;

    /// Remove an extension from the agent
    async fn remove_extension(&mut self, name: &str) -> Result<()>;

    /// Get available tools for this agent
    async fn get_available_tools(&self) -> Result<Vec<Tool>>;

    /// Enable streaming enhanced response
    async fn stream_with_enhanced_features(
        &self,
        conversation: Conversation,
        system_prompt: Option<&str>,
    ) -> Result<Box<dyn futures::Stream<Item = Result<EnhancedStreamChunk>> + Send + Unpin>>;
}

/// Agent extension trait for plugins
#[async_trait]
pub trait AgentExtension: Send + Sync {
    /// Get the extension name
    fn name(&self) -> &str;

    /// Get the extension description
    fn description(&self) -> &str;

    /// Get the extension version
    fn version(&self) -> &str;

    /// Process a message through the extension
    async fn process_message(
        &self,
        message: &UiChatMessage,
        context: &ExtensionContext,
    ) -> Result<Option<UiChatMessage>>;

    /// Get tools provided by this extension
    async fn get_tools(&self) -> Vec<Tool>;

    /// Execute a tool provided by this extension
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        context: &ExtensionContext,
    ) -> Result<ToolResult>;
}

/// Context for extension execution
#[derive(Debug, Clone)]
pub struct ExtensionContext {
    pub conversation_id: String,
    pub agent_mode: Option<GooseMode>,
    pub metadata: HashMap<String, String>,
}

/// Main Agent implementation with rig integration
pub struct GooseAgent {
    config: AgentConfig,
    rig_service: RigAgentService,
    streaming_service: StreamingAgentService,
    extensions: Arc<RwLock<HashMap<String, Box<dyn AgentExtension>>>>,
    conversation_history: Arc<RwLock<HashMap<String, Conversation>>>,
}

impl GooseAgent {
    pub fn new(config: AgentConfig, rig_service: RigAgentService) -> Self {
        let streaming_service = StreamingAgentService::new(rig_service.clone());
        Self {
            config,
            rig_service,
            streaming_service,
            extensions: Arc::new(RwLock::new(HashMap::new())),
            conversation_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_conversation(&self, id: &str) -> Option<Conversation> {
        self.conversation_history.read().await.get(id).cloned()
    }

    pub async fn save_conversation(&self, conversation: Conversation) {
        self.conversation_history
            .write()
            .await
            .insert(conversation.id.clone(), conversation);
    }

    pub async fn list_conversations(&self) -> Vec<Conversation> {
        self.conversation_history
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    pub async fn delete_conversation(&self, id: &str) -> bool {
        self.conversation_history.write().await.remove(id).is_some()
    }

    async fn process_extensions(
        &self,
        message: &UiChatMessage,
        context: &ExtensionContext,
    ) -> Result<Vec<UiChatMessage>> {
        let extensions = self.extensions.read().await;
        let mut results = Vec::new();

        for extension in extensions.values() {
            if let Some(processed) = extension.process_message(message, context).await? {
                results.push(processed);
            }
        }

        Ok(results)
    }
}

#[async_trait]
impl Agent for GooseAgent {
    async fn reply(
        &self,
        mut conversation: Conversation,
        system_prompt: Option<&str>,
        tools: Option<&[Tool]>,
    ) -> Result<Box<dyn futures::Stream<Item = Result<AgentEvent>> + Send + Unpin>> {
        // Create extension context
        let context = ExtensionContext {
            conversation_id: conversation.id.clone(),
            agent_mode: conversation.metadata.agent_mode,
            metadata: HashMap::new(),
        };

        // Process message through extensions
        if let Some(last_message) = conversation.last_message() {
            let extension_results = self.process_extensions(last_message, &context).await?;
            for result in extension_results {
                conversation.add_message(result);
            }
        }

        // Convert UiChatMessage to ChatMessage for rig service
        let chat_messages: Vec<ChatMessage> = conversation
            .messages
            .into_iter()
            .map(|msg| ChatMessage {
                role: msg.role,
                content: match msg.content {
                    MessageContent::Text(text) => text,
                    _ => "".to_string(),
                },
                timestamp: msg.timestamp,
                tool_calls: msg.tool_calls,
                tool_results: msg.tool_results,
            })
            .collect();

        // Create chat request with rig agent configuration
        let mut request = ChatRequest {
            messages: chat_messages,
            model: conversation
                .metadata
                .model
                .clone()
                .unwrap_or_else(|| "mock-local".to_string()),
            stream: true,
            max_tokens: Some(self.config.max_tokens.unwrap_or(2048)),
            temperature: self.config.temperature,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            agent_config: Some(AgentConfig {
                goose_mode: conversation.metadata.agent_mode.unwrap_or(GooseMode::Chat),
                max_iterations: self.config.max_iterations,
                require_confirmation: self.config.require_confirmation,
                readonly_tools: self.config.readonly_tools.clone(),
                enable_tool_inspection: self.config.enable_tool_inspection,
                enable_auto_compact: self.config.enable_auto_compact,
                compact_threshold: self.config.compact_threshold,
                max_turns_without_tools: self.config.max_turns_without_tools,
                enable_autopilot: self.config.enable_autopilot,
                enable_extensions: self.config.enable_extensions,
                extension_timeout: self.config.extension_timeout,
            }),
            tools: tools.map(|t| t.to_vec()),
            system_prompt: system_prompt.map(|s| s.to_string()),
        };

        let rig_service = self.rig_service.clone();
        let conversation_id = conversation.id.clone();

        let stream = async_stream::stream! {
            // Send the user message event
            if let Some(msg) = request.messages.last() {
                if matches!(msg.role, Role::User) {
                    let ui_msg = UiChatMessage {
                        role: msg.role.clone(),
                        content: MessageContent::Text(msg.content.clone()),
                        timestamp: msg.timestamp,
                        tool_calls: msg.tool_calls.clone(),
                        tool_results: msg.tool_results.clone(),
                        metadata: None,
                    };
                    yield Ok(AgentEvent::Message(ui_msg));
                }
            }

            // Stream the response using rig service
            match rig_service.send_message(request).await {
                Ok(response) => {
                    // Send thinking content if present
                    if let Some(ref thinking) = response.thinking_content {
                        let thinking_msg = UiChatMessage {
                            role: Role::Assistant,
                            content: MessageContent::Thinking(thinking.clone()),
                            timestamp: response.message.as_ref().and_then(|m| m.timestamp),
                            tool_calls: None,
                            tool_results: None,
                            metadata: None,
                        };
                        yield Ok(AgentEvent::Message(thinking_msg));
                    }

                    // Send main message
                    if let Some(msg) = response.message {
                        let ui_msg = UiChatMessage {
                            role: msg.role,
                            content: MessageContent::Text(msg.content),
                            timestamp: msg.timestamp,
                            tool_calls: msg.tool_calls,
                            tool_results: msg.tool_results,
                            metadata: None,
                        };
                        yield Ok(AgentEvent::Message(ui_msg));

                        // Send tool calls if present
                        if let Some(ref tool_calls) = msg.tool_calls {
                            for tool_call in tool_calls {
                                yield Ok(AgentEvent::ToolCall(tool_call.clone()));
                            }
                        }

                        // Send tool results if present
                        if let Some(ref tool_results) = msg.tool_results {
                            for tool_result in tool_results {
                                yield Ok(AgentEvent::ToolResult(tool_result.clone()));
                            }
                        }
                    }

                    // Send token usage if present
                    if let Some(usage) = response.token_usage {
                        yield Ok(AgentEvent::Token(usage));
                    }

                    yield Ok(AgentEvent::Done);
                }
                Err(e) => {
                    yield Ok(AgentEvent::Error(format!("Rig service error: {}", e)));
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn update_config(&mut self, config: AgentConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    async fn list_extensions(&self) -> Vec<String> {
        self.extensions.read().await.keys().cloned().collect()
    }

    async fn add_extension(&mut self, extension: Box<dyn AgentExtension>) -> Result<()> {
        let name = extension.name().to_string();
        self.extensions.write().await.insert(name, extension);
        Ok(())
    }

    async fn remove_extension(&mut self, name: &str) -> Result<()> {
        self.extensions
            .write()
            .await
            .remove(name)
            .is_some()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("Extension '{}' not found", name))
    }

    /// Get available tools for this agent
    async fn get_available_tools(&self) -> Result<Vec<Tool>> {
        let model_id = "mock-local"; // Use a default model for tool listing
        Ok(self.rig_service.list_tools(model_id).await)
    }

    /// Enable streaming enhanced response
    async fn stream_with_enhanced_features(
        &self,
        conversation: Conversation,
        system_prompt: Option<&str>,
    ) -> Result<Box<dyn futures::Stream<Item = Result<EnhancedStreamChunk>> + Send + Unpin>> {
        // Convert UiChatMessage to ChatMessage for rig service
        let chat_messages: Vec<ChatMessage> = conversation
            .messages
            .into_iter()
            .map(|msg| ChatMessage {
                role: msg.role,
                content: match msg.content {
                    MessageContent::Text(text) => text,
                    _ => "".to_string(),
                },
                timestamp: msg.timestamp,
                tool_calls: msg.tool_calls,
                tool_results: msg.tool_results,
            })
            .collect();

        let request = ChatRequest {
            messages: chat_messages,
            model: conversation
                .metadata
                .model
                .clone()
                .unwrap_or_else(|| "mock-local".to_string()),
            stream: true,
            max_tokens: Some(self.config.max_tokens.unwrap_or(2048)),
            temperature: self.config.temperature,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            agent_config: Some(AgentConfig {
                goose_mode: conversation.metadata.agent_mode.unwrap_or(GooseMode::Chat),
                max_iterations: self.config.max_iterations,
                require_confirmation: self.config.require_confirmation,
                readonly_tools: self.config.readonly_tools.clone(),
                enable_tool_inspection: self.config.enable_tool_inspection,
                enable_auto_compact: self.config.enable_auto_compact,
                compact_threshold: self.config.compact_threshold,
                max_turns_without_tools: self.config.max_turns_without_tools,
                enable_autopilot: self.config.enable_autopilot,
                enable_extensions: self.config.enable_extensions,
                extension_timeout: self.config.extension_timeout,
            }),
            tools: None,
            system_prompt: system_prompt.map(|s| s.to_string()),
        };

        // Use streaming service for enhanced features
        Ok(Box::pin(
            self.streaming_service.stream_chat_response(request).await,
        ))
    }
}

/// Agent factory for creating different types of agents with rig integration
pub struct AgentFactory;

impl AgentFactory {
    pub async fn create_default_agent() -> Result<Box<dyn Agent>> {
        let rig_service = RigAgentService::new()?;
        let config = AgentConfig {
            goose_mode: GooseMode::Chat,
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
        };

        Ok(Box::new(GooseAgent::new(config, rig_service)))
    }

    pub async fn create_reasoning_agent() -> Result<Box<dyn Agent>> {
        let rig_service = RigAgentService::new()?;
        let config = AgentConfig {
            goose_mode: GooseMode::Agent,
            max_iterations: 15,
            require_confirmation: false,
            readonly_tools: vec![],
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.7,
            max_turns_without_tools: 5,
            enable_autopilot: false,
            enable_extensions: true,
            extension_timeout: 60,
        };

        Ok(Box::new(GooseAgent::new(config, rig_service)))
    }

    pub async fn create_tool_agent() -> Result<Box<dyn Agent>> {
        let rig_service = RigAgentService::new()?;
        let config = AgentConfig {
            goose_mode: GooseMode::Agent,
            max_iterations: 20,
            require_confirmation: false,
            readonly_tools: vec![],
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.6,
            max_turns_without_tools: 2,
            enable_autopilot: false,
            enable_extensions: true,
            extension_timeout: 90,
        };

        Ok(Box::new(GooseAgent::new(config, rig_service)))
    }

    pub async fn create_autonomous_agent() -> Result<Box<dyn Agent>> {
        let rig_service = RigAgentService::new()?;
        let config = AgentConfig {
            goose_mode: GooseMode::Auto,
            max_iterations: 30,
            require_confirmation: false,
            readonly_tools: vec![],
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.7,
            max_turns_without_tools: 10,
            enable_autopilot: true,
            enable_extensions: true,
            extension_timeout: 120,
        };

        Ok(Box::new(GooseAgent::new(config, rig_service)))
    }

    /// Create an agent with custom configuration
    pub async fn create_agent_with_config(config: AgentConfig) -> Result<Box<dyn Agent>> {
        let rig_service = RigAgentService::new()?;
        Ok(Box::new(GooseAgent::new(config, rig_service)))
    }

    /// Create an agent for a specific model
    pub async fn create_agent_for_model(model_id: &str, mode: GooseMode) -> Result<Box<dyn Agent>> {
        let rig_service = RigAgentService::new()?;
        let config = AgentConfig {
            goose_mode: mode,
            max_iterations: match mode {
                GooseMode::Chat => 10,
                GooseMode::Agent => 15,
                GooseMode::Auto => 25,
            },
            require_confirmation: false,
            readonly_tools: vec![],
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.8,
            max_turns_without_tools: 5,
            enable_autopilot: matches!(mode, GooseMode::Auto),
            enable_extensions: true,
            extension_timeout: 60,
        };

        Ok(Box::new(GooseAgent::new(config, rig_service)))
    }
}

