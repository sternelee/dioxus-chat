// Advanced Agent Builder for Rig Integration
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use rig::completion::Prompt;
use rig::providers;
use rig::client::CompletionClient;

use crate::rig_agent_service::{CustomTool, DateTimeTool, WeatherTool};
use crate::chat_service_simple::{AgentConfig, GooseMode, Tool as ApiTool};

/// Agent Builder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBuilderConfig {
    pub model_id: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub agent_config: Option<AgentConfig>,
    pub tools: Option<Vec<ApiTool>>,
    pub context_docs: Vec<String>,
    pub custom_instructions: Vec<String>,
}

impl Default for AgentBuilderConfig {
    fn default() -> Self {
        Self {
            model_id: "mock-local".to_string(),
            system_prompt: None,
            temperature: None,
            max_tokens: None,
            top_p: None,
            agent_config: None,
            tools: None,
            context_docs: vec![],
            custom_instructions: vec![],
        }
    }
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn CustomTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools: HashMap<String, Box<dyn CustomTool>> = HashMap::new();

        // Register built-in tools
        tools.insert("datetime".to_string(), Box::new(DateTimeTool));
        tools.insert("weather".to_string(), Box::new(WeatherTool));

        Self { tools }
    }

    pub fn register_tool<T: CustomTool + 'static>(&mut self, name: String, tool: T) {
        self.tools.insert(name, Box::new(tool));
    }

    pub fn get_tool(&self, name: &str) -> Option<&Box<dyn CustomTool>> {
        self.tools.get(name)
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced Agent Builder with Rig Integration
pub struct RigAgentBuilder {
    config: AgentBuilderConfig,
    tool_registry: Arc<ToolRegistry>,
}

impl RigAgentBuilder {
    pub fn new(config: AgentBuilderConfig) -> Self {
        Self {
            config,
            tool_registry: Arc::new(ToolRegistry::new()),
        }
    }

    pub fn with_tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = registry;
        self
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.config.system_prompt = Some(prompt);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.config.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.config.context_docs.push(context);
        self
    }

    pub fn with_tools(mut self, tools: Vec<ApiTool>) -> Self {
        self.config.tools = Some(tools);
        self
    }

    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.config.agent_config = Some(config);
        self
    }

    fn get_agent_preamble(&self) -> String {
        let mut preamble = String::new();

        // Start with system prompt if provided
        if let Some(ref system_prompt) = self.config.system_prompt {
            preamble.push_str(system_prompt);
            preamble.push('\n');
        }

        // Add agent mode specific instructions
        if let Some(ref agent_config) = self.config.agent_config {
            match agent_config.goose_mode {
                GooseMode::Agent => {
                    preamble.push_str("\nYou are an AI agent with access to tools. ");
                    preamble.push_str("Use the available tools when they are helpful for answering the user's request. ");
                    preamble.push_str("Always think step by step and explain your reasoning when using tools.");
                },
                GooseMode::Chat => {
                    preamble.push_str("\nYou are a helpful AI assistant focused on natural conversation. ");
                    preamble.push_str("Be friendly, engaging, and conversational. ");
                    preamble.push_str("Only use tools when specifically requested or when they add clear value to the conversation.");
                },
                GooseMode::Auto => {
                    preamble.push_str("\nYou are an autonomous AI assistant. ");
                    preamble.push_str("Proactively help the user and use tools as needed. ");
                    preamble.push_str("Anticipate needs and provide comprehensive assistance. ");
                    if agent_config.enable_autopilot {
                        preamble.push_str("You can take initiative and suggest actions or tools that might be helpful.");
                    }
                },
            }

            // Add capability hints based on configuration
            if agent_config.enable_tool_inspection {
                preamble.push_str(" You can inspect and analyze tool results.");
            }

            if agent_config.enable_extensions {
                preamble.push_str(" You have access to extended capabilities and can perform complex multi-step operations.");
            }
        }

        // Add context documents
        if !self.config.context_docs.is_empty() {
            preamble.push_str("\n\nContext Information:\n");
            for (i, doc) in self.config.context_docs.iter().enumerate() {
                preamble.push_str(&format!("{}. {}\n", i + 1, doc));
            }
        }

        // Add custom instructions
        if !self.config.custom_instructions.is_empty() {
            preamble.push_str("\nAdditional Instructions:\n");
            for instruction in &self.config.custom_instructions {
                preamble.push_str(&format!("- {}\n", instruction));
            }
        }

        preamble
    }

    async fn create_openai_agent(&self) -> Result<impl Prompt> {
        let client = providers::openai::Client::from_env();

        // Build the agent with the current rig API
        let agent = client.agent(&self.config.model_id);

        // Note: The current rig API doesn't seem to support all the configuration options
        // from the old AgentBuilder. We'll need to work with what's available.
        let built_agent = agent.build();

        Ok(built_agent)
    }

    async fn create_mock_agent(&self) -> Result<impl std::fmt::Display> {
        // Simplified mock agent that just returns a formatted response
        // We'll return a simple string that implements Display
        let response = format!("Mock agent response for model: {}\nSystem: {}",
                              self.config.model_id,
                              self.get_agent_preamble());
        Ok(response)
    }

    pub async fn build(&self) -> Result<String> {
        match self.config.model_id.as_str() {
            model_id if model_id.starts_with("gpt-") || model_id.contains("openai") => {
                // For now, return a placeholder response for OpenAI agents
                let response = format!("OpenAI agent built with model: {}\nSystem: {}",
                                      self.config.model_id,
                                      self.get_agent_preamble());
                Ok(response)
            },
            "mock-local" => {
                let agent_response = self.create_mock_agent().await?;
                Ok(agent_response.to_string())
            },
            _ => {
                Ok(format!("Agent built with model: {}\nSystem: {}",
                         self.config.model_id,
                         self.get_agent_preamble()))
            }
        }
    }

    pub async fn build_with_streaming(&self) -> Result<String> {
        // For now, same as build - can be enhanced for streaming-specific agents
        self.build().await
    }
}

/// Agent Factory for creating different types of agents
pub struct AgentFactory {
    tool_registry: Arc<ToolRegistry>,
}

impl AgentFactory {
    pub fn new() -> Self {
        Self {
            tool_registry: Arc::new(ToolRegistry::new()),
        }
    }

    pub fn with_tool_registry(registry: Arc<ToolRegistry>) -> Self {
        Self { tool_registry: registry }
    }

    /// Create a conversational agent
    pub fn create_conversational_agent(&self, model_id: &str) -> RigAgentBuilder {
        let config = AgentBuilderConfig {
            model_id: model_id.to_string(),
            system_prompt: Some("You are a helpful and friendly AI assistant. Engage in natural conversations and provide thoughtful responses.".to_string()),
            temperature: Some(0.7),
            agent_config: Some(AgentConfig {
                goose_mode: GooseMode::Chat,
                ..Default::default()
            }),
            ..Default::default()
        };

        RigAgentBuilder::new(config).with_tool_registry(self.tool_registry.clone())
    }

    /// Create an agent with tool capabilities
    pub fn create_tool_agent(&self, model_id: &str) -> RigAgentBuilder {
        let config = AgentBuilderConfig {
            model_id: model_id.to_string(),
            system_prompt: Some("You are a capable AI assistant with access to various tools. Use tools when they are helpful for completing the user's request.".to_string()),
            temperature: Some(0.3),
            agent_config: Some(AgentConfig {
                goose_mode: GooseMode::Agent,
                enable_tool_inspection: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        RigAgentBuilder::new(config).with_tool_registry(self.tool_registry.clone())
    }

    /// Create an autonomous agent
    pub fn create_autonomous_agent(&self, model_id: &str) -> RigAgentBuilder {
        let config = AgentBuilderConfig {
            model_id: model_id.to_string(),
            system_prompt: Some("You are an autonomous AI assistant that can take initiative and work independently to help users achieve their goals.".to_string()),
            temperature: Some(0.5),
            agent_config: Some(AgentConfig {
                goose_mode: GooseMode::Auto,
                enable_autopilot: true,
                enable_tool_inspection: true,
                enable_extensions: true,
                max_iterations: 20,
                ..Default::default()
            }),
            ..Default::default()
        };

        RigAgentBuilder::new(config).with_tool_registry(self.tool_registry.clone())
    }

    /// Create a specialized agent for a specific domain
    pub fn create_domain_agent(&self, model_id: &str, domain: &str) -> RigAgentBuilder {
        let (system_prompt, temperature) = match domain {
            "programming" => (
                "You are an expert programming assistant. Provide clear, well-commented code and explain your solutions thoroughly.",
                0.2
            ),
            "research" => (
                "You are a research assistant. Provide detailed, well-researched information and cite your sources when possible.",
                0.3
            ),
            "creative" => (
                "You are a creative assistant. Help with creative writing, brainstorming, and artistic projects.",
                0.8
            ),
            "analysis" => (
                "You are an analytical assistant. Break down complex problems and provide structured, logical solutions.",
                0.1
            ),
            _ => (
                "You are a helpful AI assistant specialized in this domain.",
                0.5
            )
        };

        let config = AgentBuilderConfig {
            model_id: model_id.to_string(),
            system_prompt: Some(system_prompt.to_string()),
            temperature: Some(temperature),
            agent_config: Some(AgentConfig {
                goose_mode: GooseMode::Agent,
                ..Default::default()
            }),
            ..Default::default()
        };

        RigAgentBuilder::new(config).with_tool_registry(self.tool_registry.clone())
    }
}

impl Default for AgentFactory {
    fn default() -> Self {
        Self::new()
    }
}