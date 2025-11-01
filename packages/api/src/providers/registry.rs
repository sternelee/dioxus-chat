use super::anthropic::AnthropicProvider;
use super::base::*;
use super::local::LocalProvider;
use super::ollama::OllamaProvider;
use super::openai::OpenAIProvider;
use crate::chat_service::{ModelConfig, ProviderError};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct ProviderRegistry {
    providers: DashMap<String, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
        }
    }

    pub fn register_provider(&self, provider: Arc<dyn Provider>) {
        info!("Registering provider");
        // For now, use a simple ID based on the provider type name
        let provider_id = format!("provider_{}", self.providers.len());
        self.providers.insert(provider_id, provider);
    }

    pub fn register(&mut self, provider: Box<dyn Provider>) -> Result<(), ProviderError> {
        let provider_arc = Arc::new(provider);
        self.register_provider(provider_arc);
        Ok(())
    }

    pub fn get_provider(&self, provider_id: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(provider_id).map(|p| p.clone())
    }

    pub fn get_provider_for_model(&self, model_id: &str) -> Option<Arc<dyn Provider>> {
        // For now, return the first provider that supports the model
        // In a real implementation, you would have a mapping from models to providers
        self.providers.iter().next().map(|entry| entry.value().clone())
    }

    pub fn list_providers(&self) -> Vec<ProviderMetadata> {
        self.providers
            .iter()
            .map(|entry| {
                let provider = entry.value();
                // Call metadata as an associated function - we need to determine the concrete type
                // For now, return a generic metadata
                ProviderMetadata {
                    id: "unknown".to_string(),
                    name: "Unknown Provider".to_string(),
                    description: "Provider metadata unavailable".to_string(),
                    supports_streaming: provider.supports_streaming(),
                    supports_tools: provider.supports_tool_calling(),
                    supports_images: provider.supports_images(),
                    supports_audio: false,
                    max_tokens: None,
                    pricing: None,
                }
            })
            .collect()
    }

    pub async fn initialize_default_providers(
        &self,
        models: &[ModelConfig],
    ) -> Result<(), ProviderError> {
        let mut seen_providers = std::collections::HashSet::new();

        for model_config in models {
            if seen_providers.contains(&model_config.provider) {
                continue;
            }
            seen_providers.insert(model_config.provider.clone());

            match model_config.provider.as_str() {
                "openai" => match OpenAIProvider::from_config(model_config.clone()) {
                    Ok(provider) => {
                        self.register_provider(Arc::new(provider));
                        info!("Initialized OpenAI provider");
                    }
                    Err(e) => {
                        warn!("Failed to initialize OpenAI provider: {}", e);
                    }
                },
                "anthropic" => match AnthropicProvider::from_config(model_config.clone()) {
                    Ok(provider) => {
                        self.register_provider(Arc::new(provider));
                        info!("Initialized Anthropic provider");
                    }
                    Err(e) => {
                        warn!("Failed to initialize Anthropic provider: {}", e);
                    }
                },
                "ollama" => match OllamaProvider::from_config(model_config.clone()) {
                    Ok(provider) => {
                        self.register_provider(Arc::new(provider));
                        info!("Initialized Ollama provider");
                    }
                    Err(e) => {
                        warn!("Failed to initialize Ollama provider: {}", e);
                    }
                },
                "local" => match LocalProvider::from_config(model_config.clone()) {
                    Ok(provider) => {
                        self.register_provider(Arc::new(provider));
                        info!("Initialized Local provider");
                    }
                    Err(e) => {
                        error!("Failed to initialize Local provider: {}", e);
                    }
                },
                _ => {
                    warn!("Unknown provider type: {}", model_config.provider);
                }
            }
        }

        // Always register a local provider as fallback
        if !self.providers.contains_key("local") {
            let fallback_config = ModelConfig {
                model: "mock-local".to_string(),
                provider: "local".to_string(),
                context_limit: Some(4096),
                temperature: Some(0.7),
                max_tokens: Some(2048),
                toolshim: Some(false),
            };

            if let Ok(provider) = LocalProvider::from_config(fallback_config) {
                self.register_provider(Arc::new(provider));
                info!("Registered fallback Local provider");
            }
        }

        Ok(())
    }

    pub async fn create_provider_for_model(
        &self,
        model_config: &ModelConfig,
    ) -> Result<Arc<dyn Provider>, ProviderError> {
        // First try to get an existing provider
        if let Some(provider) = self.get_provider(&model_config.provider) {
            return Ok(provider);
        }

        // If not found, try to create a new one
        match model_config.provider.as_str() {
            "openai" => {
                let provider = OpenAIProvider::from_config(model_config.clone())?;
                let provider_arc = Arc::new(provider);
                self.register_provider(provider_arc.clone());
                Ok(provider_arc)
            }
            "anthropic" => {
                let provider = AnthropicProvider::from_config(model_config.clone())?;
                let provider_arc = Arc::new(provider);
                self.register_provider(provider_arc.clone());
                Ok(provider_arc)
            }
            "ollama" => {
                let provider = OllamaProvider::from_config(model_config.clone())?;
                let provider_arc = Arc::new(provider);
                self.register_provider(provider_arc.clone());
                Ok(provider_arc)
            }
            "local" => {
                let provider = LocalProvider::from_config(model_config.clone())?;
                let provider_arc = Arc::new(provider);
                self.register_provider(provider_arc.clone());
                Ok(provider_arc)
            }
            _ => Err(ProviderError {
                message: format!("Unknown provider type: {}", model_config.provider),
                code: Some("unknown_provider".to_string()),
                retry_after: None,
            }),
        }
    }

    pub fn get_providers_supporting_tools(&self) -> Vec<Arc<dyn Provider>> {
        self.providers
            .iter()
            .filter(|entry| entry.value().supports_tool_calling())
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn get_providers_supporting_streaming(&self) -> Vec<Arc<dyn Provider>> {
        self.providers
            .iter()
            .filter(|entry| entry.value().supports_streaming())
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn get_providers_supporting_images(&self) -> Vec<Arc<dyn Provider>> {
        self.providers
            .iter()
            .filter(|entry| entry.value().supports_images())
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub async fn test_provider(&self, provider_id: &str) -> Result<(), ProviderError> {
        let provider = self
            .get_provider(provider_id)
            .ok_or_else(|| ProviderError {
                message: format!("Provider not found: {}", provider_id),
                code: Some("provider_not_found".to_string()),
                retry_after: None,
            })?;

        // Create a simple test request
        use crate::chat_service::{Message, MessageContent, Role};

        let test_message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            role: Role::User,
            content: MessageContent::Text { text: "Hello, this is a test message.".to_string() },
            timestamp: None,
            metadata: None,
        };
        let test_request = CompletionRequest {
            messages: vec![test_message],
            system: Some("You are a helpful assistant.".to_string()),
            tools: None,
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: false,
        };

        // Try to validate the request first
        provider.validate_request(&test_request).await?;

        // Try a completion
        let _response = provider.complete(test_request).await?;

        Ok(())
    }

    pub async fn health_check(&self) -> Vec<(String, Result<(), ProviderError>)> {
        let mut results = Vec::new();

        for entry in self.providers.iter() {
            let provider_id = entry.key().clone();
            let provider = entry.value().clone();

            let result = async move {
                // Try to validate a simple request
                use crate::chat_service::{Message, MessageContent, Role};

                let test_message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            role: Role::User,
            content: MessageContent::Text { text: "Test".to_string() },
            timestamp: None,
            metadata: None,
        };
                let test_request = CompletionRequest {
                    messages: vec![test_message],
                    system: None,
                    tools: None,
                    temperature: None,
                    max_tokens: Some(10),
                    top_p: None,
                    frequency_penalty: None,
                    presence_penalty: None,
                    stop: None,
                    stream: false,
                };

                provider.validate_request(&test_request).await
            }
            .await;

            results.push((provider_id, result));
        }

        results
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ProviderRegistry {
    fn clone(&self) -> Self {
        // Note: This creates a new empty registry
        // In a real implementation, you might want to clone the actual providers
        Self::new()
    }
}

