use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export from API module
pub use api::{
    ChatMessage, ChatRequest, ChatResponse, Role, TokenUsage,
    Tool, ToolCall, ToolResult, MessageMetadata,
    AgentConfig, GooseMode, ModelConfig,
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

/// Core Agent trait for extensibility
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

/// Main Agent implementation
pub struct GooseAgent {
    config: AgentConfig,
    provider: Arc<dyn api::ChatProvider>,
    extensions: Arc<RwLock<HashMap<String, Box<dyn AgentExtension>>>>,
    conversation_history: Arc<RwLock<HashMap<String, Conversation>>>,
}

impl GooseAgent {
    pub fn new(
        config: AgentConfig,
        provider: Arc<dyn api::ChatProvider>,
    ) -> Self {
        Self {
            config,
            provider,
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

        // Create chat request
        let request = ChatRequest {
            messages: conversation.messages.clone(),
            model: conversation.metadata.model.clone().unwrap_or_default(),
            stream: true,
            max_tokens: None,
            temperature: Some(self.config.temperature.unwrap_or(0.7)),
            tools: tools.map(|t| t.to_vec()),
            system_prompt: system_prompt.map(|s| s.to_string()),
        };

        // Stream response from provider
        let provider = Arc::clone(&self.provider);
        let conversation_id = conversation.id.clone();

        let stream = async_stream::stream! {
            // Send the user message event
            if let Some(msg) = conversation.messages.last() {
                if matches!(msg.role, Role::User) {
                    yield Ok(AgentEvent::Message(msg.clone()));
                }
            }

            // Stream the response
            match provider.send_message_stream(request).await {
                Ok(response_json) => {
                    match serde_json::from_str::<ChatResponse>(&response_json) {
                        Ok(response) => {
                            if let Some(msg) = response.message {
                                // Send thinking content if present
                                if let Some(ref thinking) = response.reasoning_content {
                                    let thinking_msg = ChatMessage {
                                        role: Role::Assistant,
                                        content: MessageContent::Text(thinking.clone()),
                                        timestamp: msg.timestamp.clone(),
                                        tool_calls: None,
                                        tool_results: None,
                                        metadata: Some(MessageMetadata {
                                            content_type: "thinking".to_string(),
                                            ..Default::default()
                                        }),
                                    };
                                    yield Ok(AgentEvent::Message(thinking_msg));
                                }

                                // Send main message
                                yield Ok(AgentEvent::Message(msg.clone()));

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
                            yield Ok(AgentEvent::Error(format!("Failed to parse response: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    yield Ok(AgentEvent::Error(format!("Provider error: {}", e)));
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
        self.extensions.write().await.remove(name).is_some()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("Extension '{}' not found", name))
    }
}

/// Agent factory for creating different types of agents
pub struct AgentFactory;

impl AgentFactory {
    pub async fn create_default_agent(
        provider: Arc<dyn api::ChatProvider>,
    ) -> Result<Box<dyn Agent>> {
        let config = AgentConfig {
            name: "Default Agent".to_string(),
            description: "Default chat agent".to_string(),
            version: "1.0.0".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2048),
            mode: Some(GooseMode::Chat),
            tools_enabled: true,
            extensions_enabled: true,
        };

        Ok(Box::new(GooseAgent::new(config, provider)))
    }

    pub async fn create_reasoning_agent(
        provider: Arc<dyn api::ChatProvider>,
    ) -> Result<Box<dyn Agent>> {
        let config = AgentConfig {
            name: "Reasoning Agent".to_string(),
            description: "Agent with enhanced reasoning capabilities".to_string(),
            version: "1.0.0".to_string(),
            temperature: Some(0.3),
            max_tokens: Some(4096),
            mode: Some(GooseMode::ChainOfThought),
            tools_enabled: true,
            extensions_enabled: true,
        };

        Ok(Box::new(GooseAgent::new(config, provider)))
    }

    pub async fn create_tool_agent(
        provider: Arc<dyn api::ChatProvider>,
    ) -> Result<Box<dyn Agent>> {
        let config = AgentConfig {
            name: "Tool Agent".to_string(),
            description: "Agent specialized in tool usage".to_string(),
            version: "1.0.0".to_string(),
            temperature: Some(0.1),
            max_tokens: Some(8192),
            mode: Some(GooseMode::Tool),
            tools_enabled: true,
            extensions_enabled: true,
        };

        Ok(Box::new(GooseAgent::new(config, provider)))
    }
}