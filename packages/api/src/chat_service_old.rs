use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use futures::Stream;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

// Import new provider and MCP systems
use super::mcp::{
    create_builtin_tools, create_default_mcp_executor, execute_builtin_tool, McpToolExecutor,
};
use super::providers::{
    CompletionRequest, CompletionResponse, Provider, ProviderRegistry, ToolCall as ProviderToolCall,
};

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
        result: Result<Vec<String>, String>,
    },
    #[serde(rename = "image_url")]
    ImageUrl { url: String },
    #[serde(rename = "file")]
    File {
        name: String,
        content: String, // Base64 encoded content
        mime_type: String,
        size: u64,
    },
    #[serde(rename = "image")]
    Image {
        name: String,
        content: String, // Base64 encoded image
        mime_type: String,
        width: Option<u32>,
        height: Option<u32>,
        size: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: Option<String>,
    pub role: Role,
    pub created: i64,
    pub content: Vec<MessageContent>,
    #[serde(default)]
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageMetadata {
    #[serde(default = "default_user_visible")]
    pub user_visible: bool,
    #[serde(default = "default_agent_visible")]
    pub agent_visible: bool,
}

fn default_user_visible() -> bool {
    true
}
fn default_agent_visible() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Conversation(pub Vec<Message>);

// Provider traits and types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model: String,
    pub provider: String,
    pub context_limit: Option<usize>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub toolshim: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supports_streaming: bool,
    pub supports_tools: bool,
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
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProviderError {}

// Legacy provider trait for backward compatibility
#[async_trait]
pub trait LegacyProvider: Send + Sync {
    fn metadata() -> ProviderMetadata
    where
        Self: Sized;

    async fn complete(
        &self,
        system: &Option<String>,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<(Message, Usage), ProviderError>;

    async fn stream(
        &self,
        system: &Option<String>,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Message, ProviderError>> + Send>>, ProviderError>;

    fn model_config(&self) -> &ModelConfig;
}

// Tool types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub result: Result<Vec<String>, String>,
}

// File and image handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub id: String,
    pub name: String,
    pub content: String, // Base64 encoded
    pub mime_type: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: String,
    pub has_transparency: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProcessingResult {
    pub file_id: String,
    pub status: ProcessingStatus,
    pub preview_url: Option<String>,
    pub extracted_text: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

pub type ToolExecutor = Arc<
    dyn Fn(ToolCall) -> Pin<Box<dyn std::future::Future<Output = ToolResult> + Send>> + Send + Sync,
>;

// Agent events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    Message(Message),
    ToolCall(ToolCall),
    ToolResult {
        id: String,
        result: Result<String, String>,
    },
    Error(String),
    Done,
    StreamChunk(String),
    Thinking(String),
    HistoryReplaced(Conversation),
    ModelChange {
        model: String,
        mode: String,
    },
    SystemNotification {
        notification_type: SystemNotificationType,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemNotificationType {
    InlineMessage,
    ThinkingMessage,
    ErrorMessage,
    SuccessMessage,
}

// Agent configuration
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

// Agent execution context
#[derive(Clone)]
pub struct AgentContext {
    pub session_id: String,
    pub config: AgentConfig,
    pub current_iteration: Arc<AtomicUsize>,
    pub turn_count: Arc<AtomicUsize>,
    pub tools_called: Arc<RwLock<bool>>,
    pub cancellation_token: Option<tokio_util::sync::CancellationToken>,
    pub planning_state: Arc<RwLock<PlanningState>>,
    pub memory_store: Arc<RwLock<MemoryStore>>,
    pub hooks: Arc<RwLock<AgentHooks>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningState {
    pub current_plan: Option<ExecutionPlan>,
    pub completed_steps: Vec<ExecutionStep>,
    pub failed_steps: Vec<ExecutionStep>,
    pub current_step: Option<usize>,
    pub reasoning_chain: Vec<ReasoningStep>,
}

impl Default for PlanningState {
    fn default() -> Self {
        Self {
            current_plan: None,
            completed_steps: Vec::new(),
            failed_steps: Vec::new(),
            current_step: None,
            reasoning_chain: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<ExecutionStep>,
    pub created_at: DateTime<Utc>,
    pub estimated_duration: Option<u64>, // in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: String,
    pub description: String,
    pub tool_name: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub dependencies: Vec<String>, // IDs of steps that must complete first
    pub status: StepStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub id: String,
    pub step_type: ReasoningType,
    pub content: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReasoningType {
    Analysis,
    Planning,
    ToolSelection,
    Reflection,
    ErrorCorrection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStore {
    pub short_term_memory: Vec<MemoryEntry>,
    pub long_term_memory: Vec<MemoryEntry>,
    pub episodic_memory: Vec<EpisodicMemory>,
    pub semantic_memory: HashMap<String, SemanticMemory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub importance: f32, // 0.0 to 1.0
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MemoryType {
    Factual,
    Procedural,
    Episodic,
    Semantic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub id: String,
    pub episode: String,
    pub context: String,
    pub outcome: String,
    pub importance: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub concept: String,
    pub definition: String,
    pub relationships: Vec<String>,
    pub confidence: f32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct AgentHooks {
    pub pre_process_hooks: Vec<PreProcessHook>,
    pub post_process_hooks: Vec<PostProcessHook>,
    pub tool_execution_hooks: Vec<ToolExecutionHook>,
    pub planning_hooks: Vec<PlanningHook>,
    pub memory_hooks: Vec<MemoryHook>,
    pub error_handling_hooks: Vec<ErrorHandlingHook>,
}

pub type PreProcessHook = Arc<
    dyn Fn(&mut Message, &AgentContext) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;
pub type PostProcessHook = Arc<
    dyn Fn(&mut Message, &AgentContext) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;
pub type ToolExecutionHook = Arc<
    dyn Fn(
            &ToolCall,
            &AgentContext,
        ) -> Pin<Box<dyn Future<Output = Result<ToolCall, String>> + Send>>
        + Send
        + Sync,
>;
pub type PlanningHook = Arc<
    dyn Fn(
            &mut ExecutionPlan,
            &AgentContext,
        ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;
pub type MemoryHook = Arc<
    dyn Fn(
            &mut MemoryEntry,
            &AgentContext,
        ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;
pub type ErrorHandlingHook = Arc<
    dyn Fn(
            &str,
            &AgentContext,
        ) -> Pin<Box<dyn Future<Output = Result<Option<String>, String>> + Send>>
        + Send
        + Sync,
>;

impl AgentContext {
    pub fn new(session_id: String, config: AgentConfig) -> Self {
        Self {
            session_id,
            config,
            current_iteration: Arc::new(AtomicUsize::new(0)),
            turn_count: Arc::new(AtomicUsize::new(0)),
            tools_called: Arc::new(RwLock::new(false)),
            cancellation_token: None,
            planning_state: Arc::new(RwLock::new(PlanningState::default())),
            memory_store: Arc::new(RwLock::new(MemoryStore::default())),
            hooks: Arc::new(RwLock::new(AgentHooks::default())),
        }
    }

    pub fn increment_iteration(&self) -> usize {
        self.current_iteration.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn increment_turn(&self) -> usize {
        self.turn_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub async fn set_tools_called(&self, called: bool) {
        let mut tools_called = self.tools_called.write().await;
        *tools_called = called;
    }

    pub async fn were_tools_called(&self) -> bool {
        *self.tools_called.read().await
    }

    pub fn should_stop(&self) -> bool {
        self.current_iteration.load(Ordering::SeqCst) >= self.config.max_iterations
    }

    pub fn max_turns_exceeded(&self) -> bool {
        self.turn_count.load(Ordering::SeqCst) >= self.config.max_turns_without_tools
    }

    // Planning state management
    pub async fn get_planning_state(&self) -> PlanningState {
        self.planning_state.read().await.clone()
    }

    pub async fn update_planning_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut PlanningState),
    {
        let mut state = self.planning_state.write().await;
        updater(&mut *state);
    }

    // Memory management
    pub async fn add_memory(&self, memory: MemoryEntry) -> Result<(), String> {
        let mut store = self.memory_store.write().await;

        // Apply memory hooks
        let mut memory_clone = memory.clone();
        {
            let hooks = self.hooks.read().await;
            for hook in &hooks.memory_hooks {
                if let Err(e) = hook(&mut memory_clone, self).await {
                    eprintln!("Memory hook error: {}", e);
                }
            }
        }

        // Add to appropriate memory store
        match memory_clone.memory_type {
            MemoryType::Factual | MemoryType::Procedural => {
                store.short_term_memory.push(memory_clone.clone());
            }
            MemoryType::Episodic => {
                // Convert to episodic memory
                let episodic = EpisodicMemory {
                    id: memory_clone.id.clone(),
                    episode: memory_clone.content.clone(),
                    context: "Session: ".to_string() + &self.session_id,
                    outcome: "Stored".to_string(),
                    importance: memory_clone.importance,
                    created_at: memory_clone.created_at,
                };
                store.episodic_memory.push(episodic);
            }
            MemoryType::Semantic => {
                // Parse semantic memory (simplified)
                let parts: Vec<&str> = memory_clone.content.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let semantic = SemanticMemory {
                        concept: parts[0].trim().to_string(),
                        definition: parts[1].trim().to_string(),
                        relationships: Vec::new(),
                        confidence: memory_clone.importance,
                        last_updated: Utc::now(),
                    };
                    store
                        .semantic_memory
                        .insert(semantic.concept.clone(), semantic);
                }
            }
        }

        Ok(())
    }

    pub async fn search_memory(
        &self,
        query: &str,
        memory_type: Option<MemoryType>,
    ) -> Vec<MemoryEntry> {
        let store = self.memory_store.read().await;
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        let search_in_vec = |entries: &[MemoryEntry]| {
            entries
                .iter()
                .filter(|entry| {
                    let matches_type = memory_type
                        .as_ref()
                        .map_or(true, |t| matches!(entry.memory_type, t));
                    matches_type
                        && (entry.content.to_lowercase().contains(&query_lower)
                            || entry
                                .tags
                                .iter()
                                .any(|tag| tag.to_lowercase().contains(&query_lower)))
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        // Search in short-term memory
        if memory_type.as_ref().map_or(true, |t| {
            matches!(t, MemoryType::Factual | MemoryType::Procedural)
        }) {
            results.extend(search_in_vec(&store.short_term_memory));
        }

        // Search in long-term memory
        if memory_type.as_ref().map_or(true, |t| {
            matches!(t, MemoryType::Factual | MemoryType::Procedural)
        }) {
            results.extend(search_in_vec(&store.long_term_memory));
        }

        // Sort by importance and relevance
        results.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(10); // Limit results
        results
    }

    // Hooks management
    pub async fn add_pre_process_hook(&self, hook: PreProcessHook) {
        let mut hooks = self.hooks.write().await;
        hooks.pre_process_hooks.push(hook);
    }

    pub async fn add_post_process_hook(&self, hook: PostProcessHook) {
        let mut hooks = self.hooks.write().await;
        hooks.post_process_hooks.push(hook);
    }

    pub async fn add_tool_execution_hook(&self, hook: ToolExecutionHook) {
        let mut hooks = self.hooks.write().await;
        hooks.tool_execution_hooks.push(hook);
    }

    pub async fn add_planning_hook(&self, hook: PlanningHook) {
        let mut hooks = self.hooks.write().await;
        hooks.planning_hooks.push(hook);
    }

    pub async fn add_memory_hook(&self, hook: MemoryHook) {
        let mut hooks = self.hooks.write().await;
        hooks.memory_hooks.push(hook);
    }

    pub async fn add_error_handling_hook(&self, hook: ErrorHandlingHook) {
        let mut hooks = self.hooks.write().await;
        hooks.error_handling_hooks.push(hook);
    }
}

// Enhanced chat structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<Tool>>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct ChatService {
    providers: ProviderRegistry,
    models: HashMap<String, ModelConfig>,
    default_model: Option<String>,
    db: Arc<Mutex<Connection>>,
    agent_configs: Arc<RwLock<HashMap<String, AgentConfig>>>,
    active_contexts: Arc<RwLock<HashMap<String, AgentContext>>>,
    file_storage: Arc<RwLock<HashMap<String, UploadedFile>>>,
    mcp_executor: Arc<Mutex<McpToolExecutor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    #[serde(skip)]
    pub conversation: Conversation,
    pub model: String,
    pub system_prompt: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database schema for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub id: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: Option<String>,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
    pub user_visible: bool,
    pub agent_visible: bool,
}

// Conversation implementation
impl Conversation {
    pub fn new(messages: Vec<Message>) -> Result<Self, String> {
        if messages.is_empty() {
            return Err("Conversation cannot be empty".to_string());
        }

        // Basic validation - last message should be from user
        if let Some(last) = messages.last() {
            if !matches!(last.role, Role::User) {
                return Err("Conversation must end with user message".to_string());
            }
        }

        Ok(Conversation(messages))
    }

    pub fn new_unvalidated(messages: Vec<Message>) -> Self {
        Conversation(messages)
    }

    pub fn messages(&self) -> &[Message] {
        &self.0
    }

    pub fn add_message(&mut self, message: Message) {
        self.0.push(message);
    }

    pub fn last(&self) -> Option<&Message> {
        self.0.last()
    }
}

// Message constructors
impl Message {
    pub fn new(role: Role, created: i64, content: Vec<MessageContent>) -> Self {
        Self {
            id: None,
            role,
            created,
            content,
            metadata: MessageMetadata::default(),
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        Self::new(
            Role::User,
            Utc::now().timestamp(),
            vec![MessageContent::Text { text: text.into() }],
        )
    }

    pub fn user() -> MessageBuilder {
        MessageBuilder::new(Role::User)
    }

    pub fn assistant() -> MessageBuilder {
        MessageBuilder::new(Role::Assistant)
    }

    pub fn system() -> MessageBuilder {
        MessageBuilder::new(Role::System)
    }

    pub fn as_concat_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| match c {
                MessageContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone)]
pub struct MessageBuilder {
    role: Role,
    created: Option<i64>,
    content: Vec<MessageContent>,
    metadata: MessageMetadata,
}

impl MessageBuilder {
    pub fn new(role: Role) -> Self {
        Self {
            role,
            created: None,
            content: Vec::new(),
            metadata: MessageMetadata::default(),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.content
            .push(MessageContent::Text { text: text.into() });
        self
    }

    pub fn with_tool_request(
        mut self,
        id: String,
        name: String,
        arguments: serde_json::Value,
    ) -> Self {
        self.content.push(MessageContent::ToolRequest {
            id,
            name,
            arguments,
        });
        self
    }

    pub fn with_tool_response(mut self, id: String, result: Result<Vec<String>, String>) -> Self {
        self.content
            .push(MessageContent::ToolResponse { id, result });
        self
    }

    pub fn with_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Message {
        Message {
            id: None,
            role: self.role,
            created: self.created.unwrap_or_else(|| Utc::now().timestamp()),
            content: self.content,
            metadata: self.metadata,
        }
    }
}

// Default implementations
impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            user_visible: true,
            agent_visible: true,
        }
    }
}

impl ChatService {
    pub fn new() -> Result<Self, String> {
        let mut service = Self {
            providers: ProviderRegistry::new(),
            models: HashMap::new(),
            default_model: None,
            db: Self::initialize_database()?,
            agent_configs: Arc::new(RwLock::new(HashMap::new())),
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
            file_storage: Arc::new(RwLock::new(HashMap::new())),
            mcp_executor: Arc::new(Mutex::new(McpToolExecutor::new())),
        };

        // Initialize with some default models
        service.initialize_default_models();

        // Initialize providers
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        rt.block_on(async {
            if let Err(e) = service.initialize_providers().await {
                eprintln!("Warning: Failed to initialize providers: {}", e);
            }

            // Initialize MCP executor
            if let Err(e) = service.initialize_mcp().await {
                eprintln!("Warning: Failed to initialize MCP executor: {}", e);
            }
        });

        Ok(service)
    }

    fn initialize_database() -> Result<Arc<Mutex<Connection>>, String> {
        let conn = Connection::open("chat_sessions.db")
            .map_err(|e| format!("Failed to open database: {}", e))?;

        // Create sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                model TEXT NOT NULL,
                system_prompt TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create sessions table: {}", e))?;

        // Create messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                user_visible BOOLEAN NOT NULL DEFAULT 1,
                agent_visible BOOLEAN NOT NULL DEFAULT 1,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| format!("Failed to create messages table: {}", e))?;

        // Create indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;

        Ok(Arc::new(Mutex::new(conn)))
    }

    fn initialize_default_models(&mut self) {
        // Add OpenAI models
        self.models.insert(
            "gpt-3.5-turbo".to_string(),
            ModelConfig {
                model: "gpt-3.5-turbo".to_string(),
                provider: "openai".to_string(),
                context_limit: Some(16385),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                toolshim: Some(true),
            },
        );

        self.models.insert(
            "gpt-4".to_string(),
            ModelConfig {
                model: "gpt-4".to_string(),
                provider: "openai".to_string(),
                context_limit: Some(8192),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                toolshim: Some(true),
            },
        );

        self.models.insert(
            "gpt-4-turbo".to_string(),
            ModelConfig {
                model: "gpt-4-1106-preview".to_string(),
                provider: "openai".to_string(),
                context_limit: Some(128000),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                toolshim: Some(true),
            },
        );

        // Add Anthropic models
        self.models.insert(
            "claude-3-sonnet".to_string(),
            ModelConfig {
                model: "claude-3-sonnet-20240229".to_string(),
                provider: "anthropic".to_string(),
                context_limit: Some(200000),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                toolshim: Some(true),
            },
        );

        self.models.insert(
            "claude-3-opus".to_string(),
            ModelConfig {
                model: "claude-3-opus-20240229".to_string(),
                provider: "anthropic".to_string(),
                context_limit: Some(200000),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                toolshim: Some(true),
            },
        );

        // Add Ollama models
        self.models.insert(
            "llama2".to_string(),
            ModelConfig {
                model: "llama2".to_string(),
                provider: "ollama".to_string(),
                context_limit: Some(4096),
                temperature: Some(0.7),
                max_tokens: Some(2048),
                toolshim: Some(false),
            },
        );

        self.models.insert(
            "codellama".to_string(),
            ModelConfig {
                model: "codellama".to_string(),
                provider: "ollama".to_string(),
                context_limit: Some(16384),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                toolshim: Some(false),
            },
        );

        // Add local fallback model
        self.models.insert(
            "local-mock".to_string(),
            ModelConfig {
                model: "local-mock".to_string(),
                provider: "local".to_string(),
                context_limit: Some(4096),
                temperature: Some(0.7),
                max_tokens: Some(2048),
                toolshim: Some(false),
            },
        );

        self.default_model = Some("gpt-3.5-turbo".to_string());
    }

    async fn initialize_providers(&mut self) -> Result<(), String> {
        let model_configs: Vec<_> = self.models.values().cloned().collect();
        self.providers
            .initialize_default_providers(&model_configs)
            .await
            .map_err(|e| format!("Failed to initialize providers: {}", e))?;
        Ok(())
    }

    async fn initialize_mcp(&self) -> Result<(), String> {
        let mut executor = create_default_mcp_executor()
            .map_err(|e| format!("Failed to create MCP executor: {}", e))?;

        executor
            .initialize_all()
            .await
            .map_err(|e| format!("Failed to initialize MCP clients: {}", e))?;

        let mut mcp_guard = self.mcp_executor.lock().await;
        *mcp_guard = executor;

        Ok(())
    }

    // Provider management
    pub async fn get_provider_for_model(
        &self,
        model_id: &str,
    ) -> Result<std::sync::Arc<dyn Provider>, String> {
        let model_config = self
            .get_model(model_id)
            .ok_or_else(|| format!("Model not found: {}", model_id))?;

        self.providers
            .create_provider_for_model(model_config)
            .await
            .map_err(|e| format!("Failed to create provider: {}", e))
    }

    // Agent session management with SQLite
    pub async fn create_session(
        &self,
        model: String,
        system_prompt: Option<String>,
    ) -> Result<String, String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let db = self.db.lock().await;
        db.execute(
            "INSERT INTO sessions (id, model, system_prompt, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![&session_id, &model, &system_prompt, &now.to_rfc3339()],
        )
        .map_err(|e| format!("Failed to create session: {}", e))?;

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<AgentSession>, String> {
        let db = self.db.lock().await;

        // Get session info
        let session_result = db.query_row(
            "SELECT id, model, system_prompt, created_at, updated_at FROM sessions WHERE id = ?1",
            [session_id],
            |row| {
                Ok(StoredSession {
                    id: row.get(0)?,
                    model: row.get(1)?,
                    system_prompt: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            },
        );

        match session_result {
            Ok(stored_session) => {
                // Get messages for this session
                let mut stmt = db
                    .prepare(
                        "SELECT id, role, content, created_at, user_visible, agent_visible
                     FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
                    )
                    .map_err(|e| format!("Failed to prepare statement: {}", e))?;

                let message_rows = stmt
                    .query_map([session_id], |row| {
                        Ok(StoredMessage {
                            id: row.get(0)?,
                            session_id: session_id.to_string(),
                            role: row.get(1)?,
                            content: row.get(2)?,
                            created_at: row.get(3)?,
                            user_visible: row.get(4)?,
                            agent_visible: row.get(5)?,
                        })
                    })
                    .map_err(|e| format!("Failed to query messages: {}", e))?;

                let mut messages = Vec::new();
                for msg_row in message_rows {
                    let stored_msg =
                        msg_row.map_err(|e| format!("Failed to read message row: {}", e))?;

                    let role = match stored_msg.role.as_str() {
                        "user" => Role::User,
                        "assistant" => Role::Assistant,
                        "system" => Role::System,
                        _ => Role::User,
                    };

                    let content = vec![MessageContent::Text {
                        text: stored_msg.content,
                    }];
                    let metadata = MessageMetadata {
                        user_visible: stored_msg.user_visible,
                        agent_visible: stored_msg.agent_visible,
                    };

                    let mut message = Message::new(role, stored_msg.created_at, content);
                    message.metadata = metadata;
                    if let Some(id) = stored_msg.id {
                        message = message.with_id(id);
                    }

                    messages.push(message);
                }

                Ok(Some(AgentSession {
                    id: stored_session.id,
                    conversation: Conversation::new_unvalidated(messages),
                    model: stored_session.model,
                    system_prompt: stored_session.system_prompt,
                    created_at: stored_session.created_at,
                    updated_at: stored_session.updated_at,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get session: {}", e)),
        }
    }

    pub async fn update_session(&self, session_id: &str, message: Message) -> Result<(), String> {
        let db = self.db.lock().await;

        // Check if session exists
        let session_exists: bool = db
            .query_row("SELECT 1 FROM sessions WHERE id = ?1", [session_id], |_| {
                Ok(true)
            })
            .unwrap_or(false);

        if !session_exists {
            return Err(format!("Session not found: {}", session_id));
        }

        // Insert the new message
        let message_id = message
            .id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let role_str = match message.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
            Role::Tool => "tool",
        };

        let content_text = message.as_concat_text();

        db.execute(
            "INSERT INTO messages (id, session_id, role, content, created_at, user_visible, agent_visible)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &message_id,
                session_id,
                role_str,
                content_text,
                message.created,
                message.metadata.user_visible,
                message.metadata.agent_visible,
            ],
        ).map_err(|e| format!("Failed to insert message: {}", e))?;

        // Update session's updated_at timestamp
        let now = Utc::now();
        db.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![&now.to_rfc3339(), session_id],
        )
        .map_err(|e| format!("Failed to update session timestamp: {}", e))?;

        Ok(())
    }

    pub async fn clear_session(&self, session_id: &str) -> Result<(), String> {
        let db = self.db.lock().await;

        // Delete all messages for the session
        db.execute("DELETE FROM messages WHERE session_id = ?1", [session_id])
            .map_err(|e| format!("Failed to clear session messages: {}", e))?;

        // Update session's updated_at timestamp
        let now = Utc::now();
        db.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![&now.to_rfc3339(), session_id],
        )
        .map_err(|e| format!("Failed to update session timestamp: {}", e))?;

        Ok(())
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), String> {
        let db = self.db.lock().await;

        // Delete session (cascade will delete messages)
        db.execute("DELETE FROM sessions WHERE id = ?1", [session_id])
            .map_err(|e| format!("Failed to delete session: {}", e))?;

        Ok(())
    }

    pub async fn list_sessions(&self) -> Result<Vec<StoredSession>, String> {
        let db = self.db.lock().await;

        let mut stmt = db
            .prepare(
                "SELECT id, model, system_prompt, created_at, updated_at
             FROM sessions ORDER BY updated_at DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let session_iter = stmt
            .query_map([], |row| {
                Ok(StoredSession {
                    id: row.get(0)?,
                    model: row.get(1)?,
                    system_prompt: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })
            .map_err(|e| format!("Failed to query sessions: {}", e))?;

        let mut sessions = Vec::new();
        for session in session_iter {
            sessions.push(session.map_err(|e| format!("Failed to read session row: {}", e))?);
        }

        Ok(sessions)
    }

    pub fn get_available_models(&self) -> Vec<&ModelConfig> {
        self.models.values().collect()
    }

    pub fn get_model(&self, model_id: &str) -> Option<&ModelConfig> {
        self.models.get(model_id)
    }

    // List all available providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers
            .list_providers()
            .into_iter()
            .map(|p| p.id)
            .collect()
    }

    // List available tools
    pub async fn list_tools(&self, model_id: &str) -> Vec<Tool> {
        let mut all_tools = Vec::new();

        // Check if provider supports tools
        if let Ok(provider) = self.get_provider_for_model(model_id).await {
            if provider.supports_tool_calling() {
                // Add built-in tools
                let builtin_tools = create_builtin_tools();
                all_tools.extend(builtin_tools);

                // Add MCP tools
                let mut mcp_guard = self.mcp_executor.lock().await;
                if let Ok(mcp_tools) = mcp_guard.list_all_tools().await {
                    all_tools.extend(mcp_tools);
                }
            }
        }

        all_tools
    }

    // Legacy methods for backward compatibility
    async fn send_local_message(
        &self,
        request: ChatRequest,
        model_config: &ModelConfig,
    ) -> Result<ChatResponse> {
        // This is a simplified implementation for local models
        // In a real implementation, you would load the model and generate responses
        Ok(ChatResponse {
            content: format!("This is a mock response from {} model. In a real implementation, this would be generated by the actual model.", model_config.model),
            model: request.model.clone(),
            usage: Some(TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            tool_calls: None,
            finished: true,
        })
    }

    async fn send_openai_message(
        &self,
        request: ChatRequest,
        model_config: &ModelConfig,
    ) -> Result<ChatResponse> {
        // This would implement OpenAI API calls
        // For now, return a mock response
        Ok(ChatResponse {
            content: format!("This is a mock response from {} model via OpenAI API. In a real implementation, this would call the OpenAI API.", model_config.model),
            model: request.model.clone(),
            usage: Some(TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 75,
                total_tokens: 175,
            }),
            tool_calls: None,
            finished: true,
        })
    }

    // Enhanced chat methods with agent capabilities
    pub async fn send_message(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        let model_config = self
            .get_model(&request.model)
            .or_else(|| {
                self.default_model
                    .as_ref()
                    .and_then(|id| self.get_model(id))
            })
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", request.model))
            .map_err(|e| e.to_string())?;

        let provider = self.get_provider_for_model(&request.model).await?;

        // Convert chat messages to agent messages
        let messages: Vec<Message> = request
            .messages
            .into_iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    "system" => Role::System,
                    _ => Role::User,
                };

                let content = vec![MessageContent::Text { text: msg.content }];
                Message::new(role, Utc::now().timestamp(), content)
            })
            .collect();

        // Convert tools to new format
        let tools = request.tools.unwrap_or_default();
        let completion_tools = tools
            .into_iter()
            .map(|tool| Tool {
                name: tool.name,
                description: tool.description,
                parameters: tool.parameters,
            })
            .collect::<Vec<_>>();

        // Create completion request
        let completion_request = CompletionRequest {
            messages,
            system: request.system_prompt,
            tools: if completion_tools.is_empty() {
                None
            } else {
                Some(completion_tools)
            },
            temperature: model_config.temperature,
            max_tokens: model_config.max_tokens,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: false,
        };

        let response = provider
            .complete(completion_request)
            .await
            .map_err(|e| format!("Provider error: {}", e.message))?;

        // Convert tool calls back to old format
        let tool_calls = response.tool_calls.map(|calls| {
            calls
                .into_iter()
                .map(|tc| ToolCall {
                    id: tc.id,
                    name: tc.name,
                    arguments: tc.arguments,
                })
                .collect()
        });

        Ok(ChatResponse {
            content: response.content,
            model: request.model.clone(),
            usage: Some(TokenUsage {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            }),
            tool_calls,
            finished: response.finish_reason.is_some(),
        })
    }

    pub async fn send_message_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>, String> {
        let model_config = self
            .get_model(&request.model)
            .or_else(|| {
                self.default_model
                    .as_ref()
                    .and_then(|id| self.get_model(id))
            })
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", request.model))
            .map_err(|e| e.to_string())?;

        let provider = self.get_provider_for_model(&request.model).await?;

        // Convert chat messages to agent messages
        let messages: Vec<Message> = request
            .messages
            .into_iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    "system" => Role::System,
                    _ => Role::User,
                };

                let content = vec![MessageContent::Text { text: msg.content }];
                Message::new(role, Utc::now().timestamp(), content)
            })
            .collect();

        // Convert tools to new format
        let tools = request.tools.unwrap_or_default();
        let completion_tools = tools
            .into_iter()
            .map(|tool| Tool {
                name: tool.name,
                description: tool.description,
                parameters: tool.parameters,
            })
            .collect::<Vec<_>>();

        // Create completion request
        let completion_request = CompletionRequest {
            messages,
            system: request.system_prompt,
            tools: if completion_tools.is_empty() {
                None
            } else {
                Some(completion_tools)
            },
            temperature: model_config.temperature,
            max_tokens: model_config.max_tokens,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: true,
        };

        let stream = provider
            .stream(completion_request)
            .await
            .map_err(|e| format!("Provider error: {}", e.message))?;

        let converted_stream = async_stream::stream! {
            use futures::StreamExt;
            let mut stream = stream;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        if let Some(content) = chunk.content {
                            yield Ok(content);
                        }
                    }
                    Err(error) => yield Err(error.message),
                }
            }
        };

        Ok(Box::pin(converted_stream))
    }

    // Agent-style streaming with events - enhanced implementation
    pub async fn agent_reply(
        &self,
        session_id: &str,
        user_message: String,
        agent_config: Option<AgentConfig>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, String>> + Send + '_>>, String> {
        let session = self
            .get_session(session_id)
            .await
            .map_err(|_| format!("Session not found: {}", session_id))?
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        let config = agent_config.unwrap_or_default();
        let context = AgentContext::new(session_id.to_string(), config.clone());

        // Store context for this session
        {
            let mut contexts = self.active_contexts.write().await;
            contexts.insert(session_id.to_string(), context.clone());
        }

        let provider = self.get_provider_for_model(&session.model).await?;
        let service_clone = self.clone();
        let session_id_clone = session_id.to_string();
        let session_clone = session.clone();

        let stream = async_stream::stream! {
            // Create and add user message
            let user_msg = Message::user().with_text(user_message).build();

            // Yield user message first
            yield Ok(AgentEvent::Message(user_msg.clone()));

            // Add user message to session
            if let Err(e) = service_clone.update_session(&session_id_clone, user_msg.clone()).await {
                yield Ok(AgentEvent::Error(format!("Failed to add user message: {}", e)));
                return;
            }

            // Check for auto-compaction
            let current_session = service_clone.get_session(&session_id_clone).await.unwrap().unwrap();
            let current_session_clone = current_session.clone();
            let messages = current_session.conversation.messages().to_vec();
            let context_limit = service_clone.get_context_limit(&current_session.model).await.unwrap_or(4096);
            let current_usage = service_clone.estimate_token_usage(&messages);

            if config.enable_auto_compact && (current_usage as f32) > (context_limit as f32 * config.compact_threshold) {
                yield Ok(AgentEvent::SystemNotification {
                    notification_type: SystemNotificationType::InlineMessage,
                    content: format!("Context limit reached ({} tokens). Compacting to continue conversation...", current_usage),
                });

                yield Ok(AgentEvent::SystemNotification {
                    notification_type: SystemNotificationType::ThinkingMessage,
                    content: "Compacting conversation history...".to_string(),
                });

                match service_clone.compact_conversation(&session_id_clone, &current_session).await {
                    Ok(compacted_conversation) => {
                        yield Ok(AgentEvent::HistoryReplaced(compacted_conversation.clone()));
                        yield Ok(AgentEvent::SystemNotification {
                            notification_type: SystemNotificationType::SuccessMessage,
                            content: "Compaction complete".to_string(),
                        });
                    }
                    Err(e) => {
                        yield Ok(AgentEvent::Error(format!("Failed to compact conversation: {}", e)));
                        return;
                    }
                }
            }

            // Main agent loop
            let mut current_conversation = current_session.conversation;
            let mut turn_count = 0;
            let mut no_tools_called_streak = 0;

            loop {
                if context.should_stop() {
                    yield Ok(AgentEvent::Error("Maximum iterations reached".to_string()));
                    break;
                }

                if turn_count > 0 && no_tools_called_streak >= config.max_turns_without_tools {
                    yield Ok(AgentEvent::Message(
                        Message::assistant().with_text(
                            "I've reached the maximum number of actions I can do without user input. Would you like me to continue?"
                        ).build()
                    ));
                    break;
                }

                context.increment_iteration();
                turn_count += 1;

                // Get available tools
                let tools = service_clone.list_tools(&current_session.model).await;
                let system_prompt = service_clone.build_system_prompt(&current_session_clone, &tools).await;

                // Send thinking notification
                yield Ok(AgentEvent::Thinking("Processing your request...".to_string()));

                // Stream response from provider
                let request = CompletionRequest {
                    messages: current_conversation.messages().to_vec(),
                    system: Some(system_prompt),
                    tools: Some(tools.to_vec()),
                    temperature: None,
                    max_tokens: None,
                    top_p: None,
                    frequency_penalty: None,
                    presence_penalty: None,
                    stop: None,
                    stream: true,
                };

                let provider_stream = match provider.stream(request).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        yield Ok(AgentEvent::Error(format!("Provider error: {}", e.message)));
                        break;
                    }
                };

                let mut accumulated_response = String::new();
                let mut tool_calls: Vec<ToolCall> = Vec::new();
                let mut streaming_text = String::new();

                // Process streaming response
                use futures::StreamExt;
                let mut stream = provider_stream;
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(message) => {
                            let text = message.as_concat_text();
                            if !text.is_empty() {
                                accumulated_response.push_str(&text);
                                streaming_text.push_str(&text);
                                yield Ok(AgentEvent::StreamChunk(text));
                            }

                            // Extract tool calls if present
                            // This is simplified - in a real implementation, you'd parse structured tool calls
                            // from the message content
                        }
                        Err(e) => {
                            yield Ok(AgentEvent::Error(format!("Stream error: {}", e.message)));
                            break;
                        }
                    }
                }

                // Create assistant message
                let assistant_msg = Message::assistant().with_text(accumulated_response).build();
                current_conversation.add_message(assistant_msg.clone());

                // Update session with assistant message
                if let Err(e) = service_clone.update_session(&session_id_clone, assistant_msg.clone()).await {
                    yield Ok(AgentEvent::Error(format!("Failed to save assistant message: {}", e)));
                }

                // Yield the complete message
                yield Ok(AgentEvent::Message(assistant_msg.clone()));

                // If no tools were called, check if we should continue
                if tool_calls.is_empty() {
                    no_tools_called_streak += 1;
                    context.set_tools_called(false).await;

                    // For chat mode, we're done after one response
                    if config.goose_mode == GooseMode::Chat {
                        break;
                    }

                    // Check if we should continue in agent mode
                    if config.goose_mode == GooseMode::Agent && no_tools_called_streak >= config.max_turns_without_tools {
                        break;
                    }
                } else {
                    no_tools_called_streak = 0;
                    context.set_tools_called(true).await;

                    // Execute tools and add results to conversation
                    for tool_call in tool_calls {
                        yield Ok(AgentEvent::ToolCall(tool_call.clone()));

                        let tool_result = service_clone.execute_tool(&tool_call).await;

                        let result_string = match &tool_result {
                            Ok(messages) => messages.join("\n"),
                            Err(e) => e.clone(),
                        };
                        yield Ok(AgentEvent::ToolResult {
                            id: tool_call.id.clone(),
                            result: Ok(result_string),
                        });

                        // Add tool result as a user message
                        let tool_result_msg = Message::user().with_tool_response(
                            tool_call.id.clone(),
                            tool_result
                        ).build();

                        current_conversation.add_message(tool_result_msg.clone());

                        // Update session with tool result
                        if let Err(e) = service_clone.update_session(&session_id_clone, tool_result_msg).await {
                            yield Ok(AgentEvent::Error(format!("Failed to save tool result: {}", e)));
                        }
                    }

                    // Continue the loop to get the assistant's response to tool results
                    continue;
                }

                // If we reach here and no tools were called, we're done
                break;
            }

            yield Ok(AgentEvent::Done);
        };

        Ok(Box::pin(stream))
    }

    // Helper methods for enhanced agent functionality
    async fn get_context_limit(&self, model_id: &str) -> Option<usize> {
        self.get_model(model_id)
            .and_then(|config| config.context_limit)
    }

    fn estimate_token_usage(&self, messages: &[Message]) -> usize {
        // Simple estimation: count characters and divide by 4 (rough approximation)
        // In a real implementation, you'd use a proper tokenizer
        let total_chars: usize = messages.iter().map(|msg| msg.as_concat_text().len()).sum();
        (total_chars + 3) / 4 // Round up division
    }

    async fn compact_conversation(
        &self,
        session_id: &str,
        session: &AgentSession,
    ) -> Result<Conversation, String> {
        // Simple compaction: keep the system prompt and last N messages
        let messages = session.conversation.messages();
        let messages_to_keep = 10; // Keep last 10 messages

        let mut compacted_messages = Vec::new();

        // Keep system messages and first user message
        for msg in messages {
            if matches!(msg.role, Role::System) {
                compacted_messages.push(msg.clone());
            }
        }

        // Add last N messages
        let start_idx = if messages.len() > messages_to_keep {
            messages.len() - messages_to_keep
        } else {
            0
        };

        for msg in messages.iter().skip(start_idx) {
            if !matches!(msg.role, Role::System) {
                compacted_messages.push(msg.clone());
            }
        }

        // Create summary message
        if messages.len() > messages_to_keep {
            let summary_msg = Message::assistant().with_text(
                format!("Note: Previous conversation history has been compacted. {} messages were summarized to maintain context.",
                        messages.len() - messages_to_keep)
            );
            compacted_messages.insert(
                compacted_messages.len().saturating_sub(1),
                summary_msg.build(),
            );
        }

        let compacted_conversation = Conversation::new_unvalidated(compacted_messages);

        // Update the session in database
        self.clear_session(session_id).await?;

        // Re-add compacted messages
        for msg in compacted_conversation.messages() {
            self.update_session(session_id, msg.clone()).await?;
        }

        Ok(compacted_conversation)
    }

    async fn build_system_prompt(&self, session: &AgentSession, tools: &[Tool]) -> String {
        let base_prompt = session.system_prompt.clone().unwrap_or_else(|| {
            "You are a helpful AI assistant. Respond clearly and concisely.".to_string()
        });

        if tools.is_empty() {
            return base_prompt;
        }

        let tools_desc = tools
            .iter()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{}\n\nAvailable tools:\n{}\n\nUse tools when helpful to accomplish the user's request.",
            base_prompt, tools_desc
        )
    }

    async fn execute_builtin_tool(&self, tool_call: &ToolCall) -> Result<Vec<String>, String> {
        match tool_call.name.as_str() {
            "shell" => {
                let command = tool_call
                    .arguments
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'command' parameter".to_string())?;

                // Execute shell command (in a real implementation, you'd want proper sandboxing)
                match tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .await
                {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                        if output.status.success() {
                            Ok(vec![format!(
                                "Command executed successfully.\nstdout:\n{}",
                                stdout
                            )])
                        } else {
                            Ok(vec![format!(
                                "Command failed with exit code {}.\nstdout:\n{}\nstderr:\n{}",
                                output.status.code().unwrap_or(-1),
                                stdout,
                                stderr
                            )])
                        }
                    }
                    Err(e) => Ok(vec![format!("Failed to execute command: {}", e)]),
                }
            }
            "file_editor" => {
                let operation = tool_call
                    .arguments
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'command' parameter".to_string())?;

                let path = tool_call
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'path' parameter".to_string())?;

                match operation {
                    "view" => match std::fs::read_to_string(path) {
                        Ok(content) => Ok(vec![content]),
                        Err(e) => Ok(vec![format!("Failed to read file: {}", e)]),
                    },
                    "write" => {
                        let file_text = tool_call
                            .arguments
                            .get("file_text")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| "Missing 'file_text' parameter".to_string())?;

                        match std::fs::write(path, file_text) {
                            Ok(()) => Ok(vec!["File written successfully".to_string()]),
                            Err(e) => Ok(vec![format!("Failed to write file: {}", e)]),
                        }
                    }
                    _ => Ok(vec![format!("Unsupported file operation: {}", operation)]),
                }
            }
            _ => Ok(vec![format!("Unknown tool: {}", tool_call.name)]),
        }
    }

    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<Vec<String>, String> {
        // First try built-in tools
        if let Ok(result) = self.execute_builtin_tool(tool_call).await {
            return Ok(result);
        }

        // Then try MCP tools
        let mut mcp_guard = self.mcp_executor.lock().await;
        if let Ok(result) = mcp_guard
            .execute_tool(&tool_call.name, Some(tool_call.arguments.clone()))
            .await
        {
            return Ok(result);
        }

        // If neither works, return an error
        Err(format!(
            "Unknown tool or execution failed: {}",
            tool_call.name
        ))
    }

    // Agent configuration management
    pub async fn set_agent_config(&self, session_id: String, config: AgentConfig) {
        let mut configs = self.agent_configs.write().await;
        configs.insert(session_id, config);
    }

    pub async fn get_agent_config(&self, session_id: &str) -> Option<AgentConfig> {
        let configs = self.agent_configs.read().await;
        configs.get(session_id).cloned()
    }

    // Active context management
    pub async fn get_active_context(&self, session_id: &str) -> Option<AgentContext> {
        let contexts = self.active_contexts.read().await;
        contexts.get(session_id).cloned()
    }

    pub async fn cancel_session(&self, session_id: &str) -> Result<(), String> {
        let mut contexts = self.active_contexts.write().await;
        if let Some(context) = contexts.remove(session_id) {
            if let Some(token) = context.cancellation_token {
                token.cancel();
            }
            Ok(())
        } else {
            Err(format!("No active context for session: {}", session_id))
        }
    }

    // Enhanced streaming API for real-time updates
    pub async fn stream_agent_event(
        &self,
        session_id: &str,
        user_message: String,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, String>> + Send + '_>>, String> {
        let config = self.get_agent_config(session_id).await;
        self.agent_reply(session_id, user_message, config).await
    }

    // Batch processing for multiple messages
    pub async fn process_batch(
        &self,
        session_id: &str,
        messages: Vec<String>,
    ) -> Result<Vec<AgentEvent>, String> {
        let mut events = Vec::new();

        for message in messages {
            let mut stream = self.agent_reply(session_id, message, None).await?;

            use futures::StreamExt;
            while let Some(result) = stream.next().await {
                events.push(result?);
            }
        }

        Ok(events)
    }

    // File upload and processing methods
    pub async fn upload_file(
        &self,
        name: String,
        content: Vec<u8>,
        mime_type: String,
    ) -> Result<UploadedFile, String> {
        let file_id = Uuid::new_v4().to_string();
        let base64_content = general_purpose::STANDARD.encode(&content);
        let size = content.len() as u64;
        let created_at = Utc::now();

        let uploaded_file = UploadedFile {
            id: file_id.clone(),
            name,
            content: base64_content,
            mime_type: mime_type.clone(),
            size,
            created_at,
        };

        // Store in memory (in production, you'd use proper file storage)
        {
            let mut storage = self.file_storage.write().await;
            storage.insert(file_id.clone(), uploaded_file.clone());
        }

        // Process file if it's an image
        if mime_type.starts_with("image/") {
            let _ = self.process_image(&file_id).await;
        } else {
            let _ = self.extract_text(&file_id).await;
        }

        Ok(uploaded_file)
    }

    pub async fn get_file(&self, file_id: &str) -> Option<UploadedFile> {
        let storage = self.file_storage.read().await;
        storage.get(file_id).cloned()
    }

    pub async fn delete_file(&self, file_id: &str) -> Result<(), String> {
        let mut storage = self.file_storage.write().await;
        if storage.remove(file_id).is_some() {
            Ok(())
        } else {
            Err(format!("File not found: {}", file_id))
        }
    }

    pub async fn list_files(&self) -> Vec<UploadedFile> {
        let storage = self.file_storage.read().await;
        storage.values().cloned().collect()
    }

    async fn process_image(&self, file_id: &str) -> Result<ImageMetadata, String> {
        let storage = self.file_storage.read().await;
        let file = storage
            .get(file_id)
            .ok_or_else(|| format!("File not found: {}", file_id))?;

        // Decode base64 content
        let _content = general_purpose::STANDARD
            .decode(&file.content)
            .map_err(|e| format!("Failed to decode file content: {}", e))?;

        // For now, just extract basic info
        // In a real implementation, you'd use image processing libraries
        let format = file
            .mime_type
            .split('/')
            .nth(1)
            .unwrap_or("unknown")
            .to_string();

        Ok(ImageMetadata {
            width: None,
            height: None,
            format,
            has_transparency: None,
        })
    }

    async fn extract_text(&self, file_id: &str) -> Result<String, String> {
        let storage = self.file_storage.read().await;
        let file = storage
            .get(file_id)
            .ok_or_else(|| format!("File not found: {}", file_id))?;

        // For text files, decode and return content
        if file.mime_type.starts_with("text/") {
            let content = general_purpose::STANDARD
                .decode(&file.content)
                .map_err(|e| format!("Failed to decode file content: {}", e))?;

            String::from_utf8(content).map_err(|e| format!("Failed to parse text content: {}", e))
        } else {
            // For other file types, you'd implement specific extractors
            Ok(format!("File '{}' ({}, {} bytes) uploaded successfully. Content extraction not implemented for this file type.",
                      file.name, file.mime_type, file.size))
        }
    }

    // Enhanced message creation with file support
    pub fn create_message_with_files(
        &self,
        role: Role,
        text: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<Message, String> {
        let mut content = Vec::new();

        // Add text content if provided
        if let Some(text) = text {
            content.push(MessageContent::Text { text });
        }

        // Add file content
        for file_id in file_ids {
            if let Some(file) = futures::executor::block_on(self.get_file(&file_id)) {
                if file.mime_type.starts_with("image/") {
                    content.push(MessageContent::Image {
                        name: file.name,
                        content: file.content,
                        mime_type: file.mime_type,
                        width: None,
                        height: None,
                        size: file.size,
                    });
                } else {
                    content.push(MessageContent::File {
                        name: file.name,
                        content: file.content,
                        mime_type: file.mime_type,
                        size: file.size,
                    });
                }
            }
        }

        if content.is_empty() {
            return Err("Message must have either text or file content".to_string());
        }

        Ok(Message::new(role, Utc::now().timestamp(), content))
    }

    // Enhanced agent reply with file support
    pub async fn agent_reply_with_files(
        &self,
        session_id: &str,
        user_message: String,
        file_ids: Vec<String>,
        agent_config: Option<AgentConfig>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, String>> + Send + '_>>, String> {
        // Create user message with files
        let user_msg = self.create_message_with_files(Role::User, Some(user_message), file_ids)?;

        // Add user message to session
        self.update_session(session_id, user_msg.clone()).await?;

        // Get the session and create enhanced agent stream
        let session = self
            .get_session(session_id)
            .await
            .map_err(|_| format!("Session not found: {}", session_id))?
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        let config = agent_config.unwrap_or_default();
        let context = AgentContext::new(session_id.to_string(), config.clone());

        // Store context for this session
        {
            let mut contexts = self.active_contexts.write().await;
            contexts.insert(session_id.to_string(), context.clone());
        }

        let provider = self.get_provider_for_model(&session.model).await?;
        let service_clone = self.clone();
        let session_id_clone = session_id.to_string();

        let stream = async_stream::stream! {
            // Yield user message first
            yield Ok(AgentEvent::Message(user_msg.clone()));

            // Process files if present
            if let MessageContent::File { .. } | MessageContent::Image { .. } = user_msg.content.get(1).unwrap_or(&MessageContent::Text { text: String::new() }) {
                yield Ok(AgentEvent::Thinking("Processing uploaded files...".to_string()));
            }

            // Get current conversation with files
            let current_session = service_clone.get_session(&session_id_clone).await.unwrap().unwrap();
            let current_session_clone = current_session.clone();
            let mut current_conversation = current_session.conversation;
            let mut turn_count = 0;

            loop {
                if context.should_stop() {
                    yield Ok(AgentEvent::Error("Maximum iterations reached".to_string()));
                    break;
                }

                if turn_count > 0 && turn_count >= config.max_turns_without_tools {
                    yield Ok(AgentEvent::Message(
                        Message::assistant().with_text(
                            "I've reached the maximum number of actions I can do without user input. Would you like me to continue?"
                        ).build()
                    ));
                    break;
                }

                context.increment_iteration();
                turn_count += 1;

                // Get available tools (include file analysis tools)
                let mut tools = service_clone.list_tools(&current_session.model).await;
                tools.push(Tool {
                    name: "analyze_file".to_string(),
                    description: "Analyze uploaded files and extract content".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "file_id": {
                                "type": "string",
                                "description": "ID of the file to analyze"
                            }
                        },
                        "required": ["file_id"]
                    }),
                });

                let system_prompt = service_clone.build_system_prompt(&current_session_clone, &tools).await;

                // Send thinking notification
                yield Ok(AgentEvent::Thinking("Processing your request...".to_string()));

                // Stream response from provider
                let request = CompletionRequest {
                    messages: current_conversation.messages().to_vec(),
                    system: Some(system_prompt),
                    tools: Some(tools.to_vec()),
                    temperature: None,
                    max_tokens: None,
                    top_p: None,
                    frequency_penalty: None,
                    presence_penalty: None,
                    stop: None,
                    stream: true,
                };

                let provider_stream = match provider.stream(request).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        yield Ok(AgentEvent::Error(format!("Provider error: {}", e.message)));
                        break;
                    }
                };

                let mut accumulated_response = String::new();

                // Process streaming response
                use futures::StreamExt;
                let mut stream = provider_stream;
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(message) => {
                            let text = message.as_concat_text();
                            if !text.is_empty() {
                                accumulated_response.push_str(&text);
                                yield Ok(AgentEvent::StreamChunk(text));
                            }
                        }
                        Err(e) => {
                            yield Ok(AgentEvent::Error(format!("Stream error: {}", e.message)));
                            break;
                        }
                    }
                }

                // Create assistant message
                let assistant_msg = Message::assistant().with_text(accumulated_response).build();
                current_conversation.add_message(assistant_msg.clone());

                // Update session with assistant message
                if let Err(e) = service_clone.update_session(&session_id_clone, assistant_msg.clone()).await {
                    yield Ok(AgentEvent::Error(format!("Failed to save assistant message: {}", e)));
                }

                // Yield the complete message
                yield Ok(AgentEvent::Message(assistant_msg.clone()));

                // For simplicity, break after one response (tool execution would be added here)
                break;
            }

            yield Ok(AgentEvent::Done);
        };

        Ok(Box::pin(stream))
    }

    // Advanced planning and reasoning methods
    pub async fn create_plan(
        &self,
        goal: &str,
        context: &str,
        session_id: &str,
    ) -> Result<ExecutionPlan, String> {
        let plan_id = Uuid::new_v4().to_string();

        // This is a simplified planning implementation
        // In a real implementation, you'd use the LLM to generate plans
        let steps = self.generate_plan_steps(goal, context).await?;

        let plan = ExecutionPlan {
            id: plan_id,
            goal: goal.to_string(),
            steps,
            created_at: Utc::now(),
            estimated_duration: Some(300), // 5 minutes default
        };

        // Apply planning hooks
        let context = self
            .get_active_context(session_id)
            .await
            .ok_or_else(|| format!("No active context for session: {}", session_id))?;

        let mut plan_clone = plan.clone();
        {
            let hooks = context.hooks.read().await;
            for hook in &hooks.planning_hooks {
                if let Err(e) = hook(&mut plan_clone, &context).await {
                    eprintln!("Planning hook error: {}", e);
                }
            }
        }

        Ok(plan_clone)
    }

    async fn generate_plan_steps(
        &self,
        goal: &str,
        context: &str,
    ) -> Result<Vec<ExecutionStep>, String> {
        // Simplified plan generation - in reality, you'd use the LLM
        let mut steps = Vec::new();

        // Analyze the goal
        if goal.to_lowercase().contains("file") || goal.to_lowercase().contains("code") {
            steps.push(ExecutionStep {
                id: Uuid::new_v4().to_string(),
                description: "Analyze file or code requirements".to_string(),
                tool_name: Some("analyze_request".to_string()),
                parameters: Some(serde_json::json!({
                    "goal": goal,
                    "context": context
                })),
                dependencies: Vec::new(),
                status: StepStatus::Pending,
                result: None,
                error: None,
                created_at: Utc::now(),
                completed_at: None,
            });

            steps.push(ExecutionStep {
                id: Uuid::new_v4().to_string(),
                description: "Execute file operations or code generation".to_string(),
                tool_name: Some("file_editor".to_string()),
                parameters: Some(serde_json::json!({
                    "operation": "auto",
                    "goal": goal
                })),
                dependencies: vec![steps[0].id.clone()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                created_at: Utc::now(),
                completed_at: None,
            });
        } else if goal.to_lowercase().contains("search") || goal.to_lowercase().contains("find") {
            steps.push(ExecutionStep {
                id: Uuid::new_v4().to_string(),
                description: "Search for relevant information".to_string(),
                tool_name: Some("search".to_string()),
                parameters: Some(serde_json::json!({
                    "query": goal,
                    "context": context
                })),
                dependencies: Vec::new(),
                status: StepStatus::Pending,
                result: None,
                error: None,
                created_at: Utc::now(),
                completed_at: None,
            });
        } else {
            // Generic plan: analyze, plan, execute
            steps.push(ExecutionStep {
                id: Uuid::new_v4().to_string(),
                description: "Analyze the request and context".to_string(),
                tool_name: Some("analyze_request".to_string()),
                parameters: Some(serde_json::json!({
                    "goal": goal,
                    "context": context
                })),
                dependencies: Vec::new(),
                status: StepStatus::Pending,
                result: None,
                error: None,
                created_at: Utc::now(),
                completed_at: None,
            });

            steps.push(ExecutionStep {
                id: Uuid::new_v4().to_string(),
                description: "Determine best approach and tools".to_string(),
                tool_name: Some("planning".to_string()),
                parameters: Some(serde_json::json!({
                    "goal": goal,
                    "analysis": "analysis_from_step_1"
                })),
                dependencies: vec![steps[0].id.clone()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                created_at: Utc::now(),
                completed_at: None,
            });

            steps.push(ExecutionStep {
                id: Uuid::new_v4().to_string(),
                description: "Execute the determined approach".to_string(),
                tool_name: Some("execute_plan".to_string()),
                parameters: Some(serde_json::json!({
                    "plan": "plan_from_step_2"
                })),
                dependencies: vec![steps[1].id.clone()],
                status: StepStatus::Pending,
                result: None,
                error: None,
                created_at: Utc::now(),
                completed_at: None,
            });
        }

        Ok(steps)
    }

    pub async fn execute_plan(
        &self,
        plan: ExecutionPlan,
        session_id: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, String>> + Send + '_>>, String> {
        let context = self
            .get_active_context(session_id)
            .await
            .ok_or_else(|| format!("No active context for session: {}", session_id))?;

        // Store the plan in context
        context
            .update_planning_state(|state| {
                state.current_plan = Some(plan.clone());
                state.current_step = Some(0);
            })
            .await;

        let service_clone = self.clone();
        let session_id_clone = session_id.to_string();
        let plan_id = plan.id.clone();

        let stream = async_stream::stream! {
            yield Ok(AgentEvent::SystemNotification {
                notification_type: SystemNotificationType::ThinkingMessage,
                content: format!("Executing plan: {}", plan.goal),
            });

            let mut completed_steps = Vec::new();

            for (index, step) in plan.steps.iter().enumerate() {
                // Update current step
                context.update_planning_state(|state| {
                    state.current_step = Some(index);
                }).await;

                yield Ok(AgentEvent::SystemNotification {
                    notification_type: SystemNotificationType::ThinkingMessage,
                    content: format!("Step {}: {}", index + 1, step.description),
                });

                // Check dependencies
                let dependencies_met = step.dependencies.iter().all(|dep_id| {
                    completed_steps.iter().any(|completed_step: &ExecutionStep| completed_step.id == *dep_id)
                });

                if !dependencies_met {
                    yield Ok(AgentEvent::Error(format!("Dependencies not met for step: {}", step.description)));
                    break;
                }

                // Execute the step
                let step_result = service_clone.execute_step(step, &session_id_clone).await;

                match step_result {
                    Ok(result) => {
                        completed_steps.push(result.clone());

                        // Update planning state
                        context.update_planning_state(|state| {
                            if let Some(current_plan) = &mut state.current_plan {
                                if let Some(step_idx) = state.current_step {
                                    if step_idx < current_plan.steps.len() {
                                        current_plan.steps[step_idx].status = StepStatus::Completed;
                                        current_plan.steps[step_idx].result = Some(serde_json::to_value(&result).unwrap_or_default());
                                        current_plan.steps[step_idx].completed_at = Some(Utc::now());
                                    }
                                }
                                state.completed_steps.push(result.clone());
                            }
                        }).await;

                        yield Ok(AgentEvent::SystemNotification {
                            notification_type: SystemNotificationType::SuccessMessage,
                            content: format!("Completed: {}", step.description),
                        });

                        // If this step produced a message, yield it
                        if let Some(content) = result.result {
                            if let Some(text) = content.as_str() {
                                let msg = Message::assistant().with_text(text).build();
                                yield Ok(AgentEvent::Message(msg));
                            }
                        }
                    }
                    Err(e) => {
                        // Update planning state with failure
                        context.update_planning_state(|state| {
                            if let Some(current_plan) = &mut state.current_plan {
                                if let Some(step_idx) = state.current_step {
                                    if step_idx < current_plan.steps.len() {
                                        current_plan.steps[step_idx].status = StepStatus::Failed;
                                        current_plan.steps[step_idx].error = Some(e.clone());
                                    }
                                }
                                state.failed_steps.push(step.clone());
                            }
                        }).await;

                        yield Ok(AgentEvent::Error(format!("Failed step '{}': {}", step.description, e)));
                        break;
                    }
                }
            }

            yield Ok(AgentEvent::SystemNotification {
                notification_type: SystemNotificationType::SuccessMessage,
                content: "Plan execution completed".to_string(),
            });

            yield Ok(AgentEvent::Done);
        };

        Ok(Box::pin(stream))
    }

    async fn execute_step(
        &self,
        step: &ExecutionStep,
        session_id: &str,
    ) -> Result<ExecutionStep, String> {
        let mut result_step = step.clone();
        result_step.status = StepStatus::InProgress;

        // Apply tool execution hooks
        let context = self
            .get_active_context(session_id)
            .await
            .ok_or_else(|| format!("No active context for session: {}", session_id))?;

        if let Some(tool_name) = &step.tool_name {
            let tool_call = ToolCall {
                id: step.id.clone(),
                name: tool_name.clone(),
                arguments: step.parameters.clone().unwrap_or_default(),
            };

            // Apply tool execution hooks
            let mut final_tool_call = tool_call;
            {
                let hooks = context.hooks.read().await;
                for hook in &hooks.tool_execution_hooks {
                    match hook(&final_tool_call, &context).await {
                        Ok(modified_call) => final_tool_call = modified_call,
                        Err(e) => {
                            eprintln!("Tool execution hook error: {}", e);
                        }
                    }
                }
            }

            // Execute the tool
            let tool_result = self.execute_tool(&final_tool_call).await?;

            result_step.result = Some(serde_json::json!({
                "tool_call": final_tool_call,
                "result": tool_result
            }));
            result_step.status = StepStatus::Completed;
            result_step.completed_at = Some(Utc::now());
        } else {
            // For steps without tools, just mark as completed
            result_step.status = StepStatus::Completed;
            result_step.completed_at = Some(Utc::now());
            result_step.result = Some(serde_json::json!({
                "message": "Step completed without tool execution"
            }));
        }

        Ok(result_step)
    }

    pub async fn add_reasoning_step(
        &self,
        session_id: &str,
        step_type: ReasoningType,
        content: String,
        confidence: f32,
    ) -> Result<(), String> {
        let context = self
            .get_active_context(session_id)
            .await
            .ok_or_else(|| format!("No active context for session: {}", session_id))?;

        let reasoning_step = ReasoningStep {
            id: Uuid::new_v4().to_string(),
            step_type,
            content,
            confidence,
            created_at: Utc::now(),
        };

        context
            .update_planning_state(|state| {
                state.reasoning_chain.push(reasoning_step);
            })
            .await;

        Ok(())
    }

    pub async fn get_reasoning_chain(&self, session_id: &str) -> Vec<ReasoningStep> {
        if let Some(context) = self.get_active_context(session_id).await {
            context.get_planning_state().await.reasoning_chain
        } else {
            Vec::new()
        }
    }

    // Enhanced agent reply with planning capabilities
    pub async fn agent_reply_with_planning(
        &self,
        session_id: &str,
        user_message: String,
        agent_config: Option<AgentConfig>,
        enable_planning: bool,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, String>> + Send + '_>>, String> {
        let session = self
            .get_session(session_id)
            .await
            .map_err(|_| format!("Session not found: {}", session_id))?
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        let config = agent_config.unwrap_or_default();
        let context = AgentContext::new(session_id.to_string(), config.clone());

        // Store context for this session
        {
            let mut contexts = self.active_contexts.write().await;
            contexts.insert(session_id.to_string(), context.clone());
        }

        let service_clone = self.clone();
        let session_id_clone = session_id.to_string();
        let message_clone = user_message.clone();

        let stream = async_stream::stream! {
            // Add reasoning step for analysis
            if let Err(e) = service_clone.add_reasoning_step(
                &session_id_clone,
                ReasoningType::Analysis,
                format!("Analyzing user request: {}", message_clone),
                0.8,
            ).await {
                yield Ok(AgentEvent::Error(format!("Failed to add reasoning step: {}", e)));
            }

            // Create and add user message
            let user_msg = Message::user().with_text(user_message).build();
            yield Ok(AgentEvent::Message(user_msg.clone()));

            // Add user message to session
            if let Err(e) = service_clone.update_session(&session_id_clone, user_msg.clone()).await {
                yield Ok(AgentEvent::Error(format!("Failed to add user message: {}", e)));
                return;
            }

            // Determine if we need planning
            let needs_planning = enable_planning &&
                (message_clone.to_lowercase().contains("plan") ||
                 message_clone.to_lowercase().contains("step") ||
                 message_clone.to_lowercase().contains("process") ||
                 message_clone.to_lowercase().contains("execute") ||
                 message_clone.len() > 200); // Long messages might need planning

            if needs_planning {
                yield Ok(AgentEvent::SystemNotification {
                    notification_type: SystemNotificationType::ThinkingMessage,
                    content: "Creating execution plan...".to_string(),
                });

                // Add planning reasoning step
                if let Err(e) = service_clone.add_reasoning_step(
                    &session_id_clone,
                    ReasoningType::Planning,
                    "Determining if planning is needed for this request".to_string(),
                    0.7,
                ).await {
                    yield Ok(AgentEvent::Error(format!("Failed to add reasoning step: {}", e)));
                }

                match service_clone.create_plan(
                    &message_clone,
                    &format!("Session: {}, Previous context available", session_id_clone),
                    &session_id_clone,
                ).await {
                    Ok(plan) => {
                        yield Ok(AgentEvent::SystemNotification {
                            notification_type: SystemNotificationType::SuccessMessage,
                            content: format!("Created plan with {} steps", plan.steps.len()),
                        });

                        // Execute the plan
                        let plan_stream = match service_clone.execute_plan(plan, &session_id_clone).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                yield Ok(AgentEvent::Error(format!("Failed to execute plan: {}", e)));
                                return;
                            }
                        };

                        // Stream plan execution events
                        use futures::StreamExt;
                        let mut stream = plan_stream;
                        while let Some(result) = stream.next().await {
                            yield result;
                        }
                    }
                    Err(e) => {
                        yield Ok(AgentEvent::Error(format!("Failed to create plan: {}", e)));

                        // Fall back to regular agent reply
                        let fallback_stream = match service_clone.agent_reply(&session_id_clone, message_clone, Some(config)).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                yield Ok(AgentEvent::Error(format!("Fallback agent reply failed: {}", e)));
                                return;
                            }
                        };

                        use futures::StreamExt;
                        let mut stream = fallback_stream;
                        while let Some(result) = stream.next().await {
                            yield result;
                        }
                    }
                }
            } else {
                // Regular agent reply without planning
                let regular_stream = match service_clone.agent_reply(&session_id_clone, message_clone, Some(config)).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        yield Ok(AgentEvent::Error(format!("Agent reply failed: {}", e)));
                        return;
                    }
                };

                use futures::StreamExt;
                let mut stream = regular_stream;
                while let Some(result) = stream.next().await {
                    yield result;
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

impl Default for ChatService {
    fn default() -> Self {
        Self::new().expect("Failed to create ChatService with default configuration")
    }
}

// Implement Clone for ChatService
impl Clone for ChatService {
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(), // ProviderRegistry can be cloned
            models: self.models.clone(),
            default_model: self.default_model.clone(),
            db: Arc::clone(&self.db),
            agent_configs: Arc::clone(&self.agent_configs),
            active_contexts: Arc::clone(&self.active_contexts),
            file_storage: Arc::clone(&self.file_storage),
            mcp_executor: Arc::clone(&self.mcp_executor),
        }
    }
}

// Helper function to convert legacy ChatMessage to new Message
impl From<ChatMessage> for Message {
    fn from(chat_msg: ChatMessage) -> Self {
        let role = match chat_msg.role.as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "system" => Role::System,
            _ => Role::User,
        };

        let content = vec![MessageContent::Text {
            text: chat_msg.content,
        }];
        let created = chat_msg
            .timestamp
            .and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok())
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|| Utc::now().timestamp());

        Message::new(role, created, content)
    }
}

// Helper function to convert new Message to legacy ChatMessage
impl From<Message> for ChatMessage {
    fn from(msg: Message) -> Self {
        let role = match msg.role {
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::System => "system".to_string(),
            Role::Tool => "tool".to_string(),
        };

        let timestamp = Some(
            DateTime::from_timestamp(msg.created, 0)
                .unwrap_or_else(|| Utc::now())
                .to_rfc3339(),
        );

        ChatMessage {
            role,
            content: msg.as_concat_text(),
            timestamp,
            tool_calls: None,   // TODO: Convert tool calls
            tool_results: None, // TODO: Convert tool results
        }
    }
}
