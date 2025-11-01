use crate::chat_service::{Message, ModelConfig, ProviderError, Tool, Usage};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_images: bool,
    pub supports_audio: bool,
    pub max_tokens: Option<usize>,
    pub pricing: Option<PricingInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub tools: Option<Vec<Tool>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub usage: Usage,
    pub finish_reason: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<crate::chat_service::ToolResult>>,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub content: Option<String>,
    pub delta: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
}

impl ChatChunk {
    pub fn as_concat_text(&self) -> String {
        self.content
            .clone()
            .or_else(|| self.delta.clone())
            .unwrap_or_default()
    }
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn metadata() -> ProviderMetadata
    where
        Self: Sized;

    fn model_config(&self) -> &ModelConfig;

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError>;

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>;

    async fn validate_request(&self, request: &CompletionRequest) -> Result<(), ProviderError> {
        // Default validation
        if request.messages.is_empty() {
            return Err(ProviderError {
                message: "No messages provided".to_string(),
                code: Some("empty_messages".to_string()),
                retry_after: None,
            });
        }
        Ok(())
    }

    fn supports_tool_calling(&self) -> bool {
        // Default implementation - providers can override this
        true
    }

    fn supports_streaming(&self) -> bool {
        // Default implementation - providers can override this
        true
    }

    fn supports_images(&self) -> bool {
        // Default implementation - providers can override this
        false
    }

    fn extract_text_content(&self, message: &Message) -> String {
        match &message.content {
            crate::chat_service::MessageContent::Text { text } => text.clone(),
            crate::chat_service::MessageContent::ToolRequest { name, arguments, .. } => {
                format!("Tool call: {}({})", name, arguments)
            }
            crate::chat_service::MessageContent::ToolResponse { result, .. } => {
                format!("Tool result: {}", result)
            }
            crate::chat_service::MessageContent::Image { url, description } => {
                format!("[Image: {}] {}", url, description.as_ref().unwrap_or(&String::new()))
            }
        }
    }

    async fn count_tokens(&self, messages: &[Message]) -> Result<usize, ProviderError> {
        // Default implementation - can be overridden by providers
        let text: String = messages
            .iter()
            .map(|msg| self.extract_text_content(msg))
            .collect::<Vec<_>>()
            .join(" ");

        // Rough estimation: ~4 characters per token
        Ok((text.len() + 3) / 4)
    }

    async fn truncate_messages(
        &self,
        messages: &[Message],
        max_tokens: usize,
    ) -> Result<Vec<Message>, ProviderError> {
        let mut result = Vec::new();
        let mut current_tokens = 0;

        // Keep system messages if present
        for msg in messages {
            if matches!(msg.role, crate::chat_service::Role::System) {
                let tokens = self.count_tokens(&[msg.clone()]).await?;
                current_tokens += tokens;
                result.push(msg.clone());
            }
        }

        // Add other messages in reverse order until we hit the limit
        for msg in messages.iter().rev() {
            if matches!(msg.role, crate::chat_service::Role::System) {
                continue;
            }

            let tokens = self.count_tokens(&[msg.clone()]).await?;
            if current_tokens + tokens > max_tokens {
                break;
            }

            current_tokens += tokens;
            result.insert(0, msg.clone());
        }

        if result.is_empty() {
            return Err(ProviderError {
                message: "Cannot fit any messages within token limit".to_string(),
                code: Some("token_limit_exceeded".to_string()),
                retry_after: None,
            });
        }

        Ok(result)
    }

    async fn list_models(&self) -> Vec<crate::chat_service::ModelConfig> {
        // Default implementation - return the model config from this provider
        vec![self.model_config().clone()]
    }

    async fn list_tools(&self) -> Vec<crate::chat_service::Tool> {
        // Default implementation - no tools
        vec![]
    }
}

