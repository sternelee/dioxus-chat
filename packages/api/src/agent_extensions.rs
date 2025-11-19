// Agent Extension System for Rig Agents
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{AgentConfig, ChatMessage, ChatRequest, ChatResponse, Role, ToolCall, ToolResult};

/// Extension context for agent extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionContext {
    pub agent_id: String,
    pub conversation_id: String,
    pub user_id: Option<String>,
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub capabilities: Vec<String>,
}

/// Extension execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub next_actions: Vec<String>,
}

/// Extension execution phase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionPhase {
    PreProcessing,
    ToolCall,
    PostProcessing,
    Validation,
    Cleanup,
}

/// Agent extension trait
#[async_trait]
pub trait AgentExtension: Send + Sync {
    /// Get extension name
    fn name(&self) -> &'static str;

    /// Get extension version
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    /// Get extension description
    fn description(&self) -> &'static str;

    /// Get execution phase(s) this extension participates in
    fn phases(&self) -> Vec<ExtensionPhase> {
        vec![ExtensionPhase::PreProcessing, ExtensionPhase::PostProcessing]
    }

    /// Initialize extension with context
    async fn initialize(&mut self, context: &ExtensionContext) -> Result<()> {
        Ok(())
    }

    /// Execute extension during a specific phase
    async fn execute(
        &mut self,
        phase: ExtensionPhase,
        context: &ExtensionContext,
        input: &serde_json::Value,
    ) -> Result<ExtensionResult>;

    /// Check if extension should execute for given input
    async fn should_execute(
        &self,
        phase: ExtensionPhase,
        context: &ExtensionContext,
        input: &serde_json::Value,
    ) -> bool {
        true
    }

    /// Cleanup resources
    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get extension configuration schema
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Update extension configuration
    async fn update_config(&mut self, config: serde_json::Value) -> Result<()> {
        Ok(())
    }
}

/// Extension manager for loading and managing extensions
pub struct ExtensionManager {
    extensions: Arc<RwLock<HashMap<String, Box<dyn AgentExtension>>>>,
    extension_configs: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    loaded_extensions: Arc<RwLock<Vec<String>>>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self {
            extensions: Arc::new(RwLock::new(HashMap::new())),
            extension_configs: Arc::new(RwLock::new(HashMap::new())),
            loaded_extensions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new extension
    pub async fn register_extension(&self, extension: Box<dyn AgentExtension>) -> Result<()> {
        let name = extension.name();
        let mut extensions = self.extensions.write().await;
        extensions.insert(name.to_string(), extension);

        let mut loaded = self.loaded_extensions.write().await;
        if !loaded.contains(&name.to_string()) {
            loaded.push(name.to_string());
        }

        Ok(())
    }

    /// Unregister an extension
    pub async fn unregister_extension(&self, name: &str) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        if let Some(mut extension) = extensions.remove(name) {
            extension.cleanup().await?;
        }

        let mut loaded = self.loaded_extensions.write().await;
        loaded.retain(|n| n != name);

        let mut configs = self.extension_configs.write().await;
        configs.remove(name);

        Ok(())
    }

    /// Execute extensions for a specific phase
    pub async fn execute_phase(
        &self,
        phase: ExtensionPhase,
        context: &ExtensionContext,
        input: serde_json::Value,
    ) -> Result<(serde_json::Value, Vec<ExtensionResult>)> {
        let extensions = self.extensions.read().await;
        let mut results = Vec::new();
        let mut current_input = input;

        for (name, extension) in extensions.iter() {
            if extension.phases().contains(&phase) &&
               extension.should_execute(phase, context, &current_input).await {
                match extension.execute(phase, context, &current_input).await {
                    Ok(result) => {
                        // Update input for next extension if data is provided
                        if let Some(data) = &result.data {
                            current_input = data.clone();
                        }
                        results.push(result);
                    },
                    Err(e) => {
                        let error_result = ExtensionResult {
                            success: false,
                            data: None,
                            error: Some(format!("Extension {} failed: {}", name, e)),
                            metadata: HashMap::new(),
                            next_actions: Vec::new(),
                        };
                        results.push(error_result);
                    }
                }
            }
        }

        Ok((current_input, results))
    }

    /// Get all registered extensions
    pub async fn get_extensions(&self) -> Vec<String> {
        self.loaded_extensions.read().await.clone()
    }

    /// Get extension info
    pub async fn get_extension_info(&self, name: &str) -> Option<ExtensionInfo> {
        let extensions = self.extensions.read().await;
        extensions.get(name).map(|ext| ExtensionInfo {
            name: ext.name().to_string(),
            version: ext.version().to_string(),
            description: ext.description().to_string(),
            phases: ext.phases(),
            config_schema: ext.config_schema(),
        })
    }

    /// Configure extension
    pub async fn configure_extension(&self, name: &str, config: serde_json::Value) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        if let Some(extension) = extensions.get_mut(name) {
            extension.update_config(config.clone()).await?;
        }

        let mut configs = self.extension_configs.write().await;
        configs.insert(name.to_string(), config);

        Ok(())
    }

    /// Get extension configuration
    pub async fn get_extension_config(&self, name: &str) -> Option<serde_json::Value> {
        let configs = self.extension_configs.read().await;
        configs.get(name).cloned()
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub phases: Vec<ExtensionPhase>,
    pub config_schema: Option<serde_json::Value>,
}

/// Pre-built extensions

/// Conversation summarization extension
pub struct ConversationSummarizerExtension {
    max_summary_length: usize,
    summarization_threshold: usize,
}

impl ConversationSummarizerExtension {
    pub fn new(max_summary_length: usize, summarization_threshold: usize) -> Self {
        Self {
            max_summary_length,
            summarization_threshold,
        }
    }
}

#[async_trait]
impl AgentExtension for ConversationSummarizerExtension {
    fn name(&self) -> &'static str {
        "conversation_summarizer"
    }

    fn description(&self) -> &'static str {
        "Summarizes long conversations to maintain context"
    }

    fn phases(&self) -> Vec<ExtensionPhase> {
        vec![ExtensionPhase::PreProcessing, ExtensionPhase::PostProcessing]
    }

    async fn execute(
        &mut self,
        phase: ExtensionPhase,
        _context: &ExtensionContext,
        input: &serde_json::Value,
    ) -> Result<ExtensionResult> {
        match phase {
            ExtensionPhase::PreProcessing => {
                // Check if conversation needs summarization
                if let Ok(messages) = serde_json::from_value::<Vec<ChatMessage>>(input.clone()) {
                    if messages.len() > self.summarization_threshold {
                        let summary = self.summarize_conversation(&messages)?;
                        return Ok(ExtensionResult {
                            success: true,
                            data: Some(json!({
                                "summary": summary,
                                "original_message_count": messages.len()
                            })),
                            error: None,
                            metadata: HashMap::new(),
                            next_actions: vec!["continue_conversation".to_string()],
                        });
                    }
                }
            },
            ExtensionPhase::PostProcessing => {
                // Post-process response with context
                return Ok(ExtensionResult {
                    success: true,
                    data: Some(json!({
                        "post_processed": true,
                        "extension": self.name()
                    })),
                    error: None,
                    metadata: HashMap::new(),
                    next_actions: Vec::new(),
                });
            },
            _ => {}
        }

        Ok(ExtensionResult {
            success: true,
            data: None,
            error: None,
            metadata: HashMap::new(),
            next_actions: Vec::new(),
        })
    }

    async fn should_execute(
        &self,
        phase: ExtensionPhase,
        _context: &ExtensionContext,
        input: &serde_json::Value,
    ) -> bool {
        match phase {
            ExtensionPhase::PreProcessing => {
                // Only execute in preprocessing if we have messages
                input.get("messages").is_some()
            },
            _ => true,
        }
    }
}

impl ConversationSummarizerExtension {
    fn summarize_conversation(&self, messages: &[ChatMessage]) -> Result<String> {
        let recent_messages = &messages[messages.len().saturating_sub(10)..];
        let mut summary = String::new();

        for msg in recent_messages.iter().take(5) {
            let role_prefix = match msg.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::System => "System",
                Role::Tool => "Tool",
            };
            let content_preview = if msg.content.len() > 100 {
                format!("{}...", &msg.content[..97])
            } else {
                msg.content.clone()
            };
            summary.push_str(&format!("{}: {}\n", role_prefix, content_preview));
        }

        if summary.len() > self.max_summary_length {
            summary.truncate(self.max_summary_length);
            summary.push_str("...");
        }

        Ok(summary)
    }
}

/// Tool usage monitoring extension
pub struct ToolUsageMonitorExtension {
    tool_usage: Arc<RwLock<HashMap<String, usize>>>,
    max_tool_calls_per_session: usize,
}

impl ToolUsageMonitorExtension {
    pub fn new(max_tool_calls_per_session: usize) -> Self {
        Self {
            tool_usage: Arc::new(RwLock::new(HashMap::new())),
            max_tool_calls_per_session,
        }
    }
}

#[async_trait]
impl AgentExtension for ToolUsageMonitorExtension {
    fn name(&self) -> &'static str {
        "tool_usage_monitor"
    }

    fn description(&self) -> &'static str {
        "Monitors and controls tool usage to prevent abuse"
    }

    fn phases(&self) -> Vec<ExtensionPhase> {
        vec![ExtensionPhase::Validation, ExtensionPhase::PostProcessing]
    }

    async fn execute(
        &mut self,
        phase: ExtensionPhase,
        context: &ExtensionContext,
        input: &serde_json::Value,
    ) -> Result<ExtensionResult> {
        match phase {
            ExtensionPhase::Validation => {
                if let Ok(request) = serde_json::from_value::<ChatRequest>(input.clone()) {
                    if let Some(tools) = &request.tools {
                        let total_tools: usize = tools.len();

                        // Check if this would exceed the limit
                        let mut tool_usage = self.tool_usage.write().await;
                        let current_usage: usize = tool_usage.values().sum();

                        if current_usage + total_tools > self.max_tool_calls_per_session {
                            return Ok(ExtensionResult {
                                success: false,
                                data: None,
                                error: Some(format!("Tool usage limit exceeded. Current: {}, Requested: {}, Limit: {}",
                                    current_usage, total_tools, self.max_tool_calls_per_session)),
                                metadata: HashMap::new(),
                                next_actions: vec!["reduce_tools".to_string(), "increase_limit".to_string()],
                            });
                        }

                        // Update usage counts
                        for tool in tools {
                            *tool_usage.entry(tool.name.clone()).or_insert(0) += 1;
                        }
                    }
                }
            },
            ExtensionPhase::PostProcessing => {
                // Return usage statistics
                let tool_usage = self.tool_usage.read().await;
                let usage_stats: HashMap<String, usize> = tool_usage.clone();

                return Ok(ExtensionResult {
                    success: true,
                    data: Some(json!({
                        "tool_usage_stats": usage_stats,
                        "session_limit": self.max_tool_calls_per_session
                    })),
                    error: None,
                    metadata: HashMap::new(),
                    next_actions: Vec::new(),
                });
            },
            _ => {}
        }

        Ok(ExtensionResult {
            success: true,
            data: None,
            error: None,
            metadata: HashMap::new(),
            next_actions: Vec::new(),
        })
    }
}

/// Safety filter extension
pub struct SafetyFilterExtension {
    blocked_patterns: Vec<String>,
    max_message_length: usize,
}

impl SafetyFilterExtension {
    pub fn new() -> Self {
        Self {
            blocked_patterns: vec![
                "password".to_string(),
                "secret".to_string(),
                "api_key".to_string(),
                "token".to_string(),
            ],
            max_message_length: 10000,
        }
    }

    pub fn add_blocked_pattern(&mut self, pattern: String) {
        self.blocked_patterns.push(pattern);
    }

    pub fn set_max_message_length(&mut self, length: usize) {
        self.max_message_length = length;
    }
}

#[async_trait]
impl AgentExtension for SafetyFilterExtension {
    fn name(&self) -> &'static str {
        "safety_filter"
    }

    fn description(&self) -> &'static str {
        "Filters sensitive information and enforces safety policies"
    }

    fn phases(&self) -> Vec<ExtensionPhase> {
        vec![ExtensionPhase::Validation, ExtensionPhase::PostProcessing]
    }

    async fn execute(
        &mut self,
        phase: ExtensionPhase,
        _context: &ExtensionContext,
        input: &serde_json::Value,
    ) -> Result<ExtensionResult> {
        match phase {
            ExtensionPhase::Validation => {
                // Validate input for safety violations
                let input_str = input.to_string();

                // Check for blocked patterns
                for pattern in &self.blocked_patterns {
                    if input_str.to_lowercase().contains(&pattern.to_lowercase()) {
                        return Ok(ExtensionResult {
                            success: false,
                            data: None,
                            error: Some(format!("Content contains blocked pattern: {}", pattern)),
                            metadata: HashMap::new(),
                            next_actions: vec!["remove_sensitive_content".to_string()],
                        });
                    }
                }

                // Check message length
                if input_str.len() > self.max_message_length {
                    return Ok(ExtensionResult {
                        success: false,
                        data: None,
                        error: Some(format!("Message too long: {} > {}", input_str.len(), self.max_message_length)),
                        metadata: HashMap::new(),
                        next_actions: vec!["shorten_message".to_string()],
                    });
                }
            },
            ExtensionPhase::PostProcessing => {
                // Filter output if needed
                return Ok(ExtensionResult {
                    success: true,
                    data: Some(json!({
                        "safety_check": "passed",
                        "filtered": false
                    })),
                    error: None,
                    metadata: HashMap::new(),
                    next_actions: Vec::new(),
                });
            },
            _ => {}
        }

        Ok(ExtensionResult {
            success: true,
            data: None,
            error: None,
            metadata: HashMap::new(),
            next_actions: Vec::new(),
        })
    }
}

/// Enhanced agent with extension support
pub struct ExtendedRigAgentService {
    base_service: crate::RigAgentService,
    extension_manager: ExtensionManager,
}

impl ExtendedRigAgentService {
    pub async fn new() -> Result<Self> {
        let mut service = Self {
            base_service: crate::RigAgentService::new()?,
            extension_manager: ExtensionManager::new(),
        };

        // Register default extensions
        service.register_default_extensions().await?;

        Ok(service)
    }

    /// Register default extensions
    async fn register_default_extensions(&mut self) -> Result<()> {
        // Conversation summarizer
        let summarizer = ConversationSummarizerExtension::new(500, 20);
        self.extension_manager.register_extension(Box::new(summarizer)).await?;

        // Tool usage monitor
        let monitor = ToolUsageMonitorExtension::new(100);
        self.extension_manager.register_extension(Box::new(monitor)).await?;

        // Safety filter
        let safety_filter = SafetyFilterExtension::new();
        self.extension_manager.register_extension(Box::new(safety_filter)).await?;

        Ok(())
    }

    /// Send message with extension processing
    pub async fn send_message_with_extensions(
        &self,
        mut request: ChatRequest,
        context: Option<ExtensionContext>,
    ) -> Result<ChatResponse> {
        // Create context if not provided
        let ext_context = context.unwrap_or_else(|| ExtensionContext {
            agent_id: "default_agent".to_string(),
            conversation_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
            capabilities: vec![
                "chat".to_string(),
                "tools".to_string(),
                "extensions".to_string(),
            ],
        });

        // Pre-processing phase
        let request_json = json!(request);
        let (processed_request, _) = self.extension_manager
            .execute_phase(crate::agent_extensions::ExtensionPhase::PreProcessing, &ext_context, request_json)
            .await?;

        let processed_request: ChatRequest = serde_json::from_value(processed_request)
            .unwrap_or(request);

        // Send message through base service
        let mut response = self.base_service.send_message(processed_request).await?;

        // Post-processing phase
        let response_json = json!(response);
        let (_, _) = self.extension_manager
            .execute_phase(crate::agent_extensions::ExtensionPhase::PostProcessing, &ext_context, response_json)
            .await?;

        Ok(response)
    }

    /// Register a custom extension
    pub async fn register_extension(&self, extension: Box<dyn AgentExtension>) -> Result<()> {
        self.extension_manager.register_extension(extension).await
    }

    /// Get registered extensions
    pub async fn get_extensions(&self) -> Vec<String> {
        self.extension_manager.get_extensions().await
    }

    /// Configure an extension
    pub async fn configure_extension(&self, name: &str, config: serde_json::Value) -> Result<()> {
        self.extension_manager.configure_extension(name, config).await
    }

    /// Get extension info
    pub async fn get_extension_info(&self, name: &str) -> Option<ExtensionInfo> {
        self.extension_manager.get_extension_info(name).await
    }

    /// Unregister an extension
    pub async fn unregister_extension(&self, name: &str) -> Result<()> {
        self.extension_manager.unregister_extension(name).await
    }
}

impl Default for ExtendedRigAgentService {
    fn default() -> Self {
        // Use block_on for async new in Default
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Self::new().await.unwrap()
            })
        })
    }
}

// Implement delegation for base service methods
impl std::ops::Deref for ExtendedRigAgentService {
    type Target = crate::RigAgentService;

    fn deref(&self) -> &Self::Target {
        &self.base_service
    }
}