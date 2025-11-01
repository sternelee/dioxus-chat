use super::base::*;
use crate::chat_service::{Message, MessageContent, ModelConfig, ProviderError, Role, Tool, Usage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde_json::{json, Value};
use std::pin::Pin;
use tracing::{debug, warn};

pub struct OpenAIProvider {
    api_key: String,
    model_config: ModelConfig,
}

impl OpenAIProvider {
    fn extract_text_content(&self, message: &crate::chat_service::Message) -> String {
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
    pub fn new(api_key: String, model_config: ModelConfig) -> Self {
        Self {
            api_key,
            model_config,
        }
    }

    pub fn from_config(model_config: ModelConfig) -> Result<Self, ProviderError> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| ProviderError {
            message: "OPENAI_API_KEY environment variable not set".to_string(),
            code: Some("missing_api_key".to_string()),
            retry_after: None,
        })?;

        Ok(Self::new(api_key, model_config))
    }

    async fn call_openai_api(
        &self,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        // Create HTTP client
        let client = reqwest::Client::new();

        // Build request payload
        let mut payload = json!({
            "model": self.model_config.model,
            "messages": [],
            "stream": false
        });

        // Add system message if provided
        let mut messages = Vec::new();
        if let Some(system) = &request.system {
            messages.push(json!({
                "role": "system",
                "content": system
            }));
        }

        // Convert messages
        for msg in &request.messages {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::System => "system",
                Role::Tool => continue, // Skip tool messages for now
            };

            let content = self.extract_text_content(msg);
            if !content.is_empty() {
                messages.push(json!({
                    "role": role,
                    "content": content
                }));
            }
        }

        payload["messages"] = json!(messages);

        // Add parameters
        if let Some(temp) = request.temperature {
            payload["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = request.max_tokens {
            payload["max_tokens"] = json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            payload["top_p"] = json!(top_p);
        }

        // Add tools if provided
        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                let openai_tools = tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": tool.name,
                                "description": tool.description,
                                "parameters": tool.input_schema
                            }
                        })
                    })
                    .collect::<Vec<_>>();

                payload["tools"] = json!(openai_tools);
                payload["tool_choice"] = json!("auto");
            }
        }

        debug!("Calling OpenAI API for model: {}", self.model_config.model);

        // Make the request
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError {
                message: format!("OpenAI API request failed: {}", e),
                code: Some("network_error".to_string()),
                retry_after: None,
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError {
                message: format!("OpenAI API error: {} - {}", status, error_text),
                code: Some("api_error".to_string()),
                retry_after: None,
            });
        }

        let response_json: Value = response.json().await.map_err(|e| ProviderError {
            message: format!("Failed to parse OpenAI response: {}", e),
            code: Some("parse_error".to_string()),
            retry_after: None,
        })?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tool_calls = response_json["choices"][0]["message"]["tool_calls"]
            .as_array()
            .map(|calls| {
                calls
                    .iter()
                    .map(|call| ToolCall {
                        id: call["id"].as_str().unwrap_or("").to_string(),
                        name: call["function"]["name"].as_str().unwrap_or("").to_string(),
                        arguments: call["function"]["arguments"].clone(),
                    })
                    .collect()
            });

        let usage = response_json["usage"]
            .as_object()
            .map(|usage| Usage {
                prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
            })
            .unwrap_or(Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            });

        let finish_reason = response_json["choices"][0]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        Ok(CompletionResponse {
            content,
            usage,
            finish_reason,
            tool_calls,
        })
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn metadata() -> ProviderMetadata
    where
        Self: Sized,
    {
        ProviderMetadata {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            description: "Official OpenAI API provider for GPT models".to_string(),
            supports_streaming: true,
            supports_tools: true,
            supports_images: true,
            supports_audio: false,
            max_tokens: Some(128000),
            pricing: Some(PricingInfo {
                input_cost_per_1k: 0.005,
                output_cost_per_1k: 0.015,
                currency: "USD".to_string(),
            }),
        }
    }

    fn model_config(&self) -> &ModelConfig {
        &self.model_config
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        self.call_openai_api(&request).await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        // For now, convert to non-streaming and return as a single chunk
        let response = self.call_openai_api(&request).await?;

        let converted_stream = async_stream::stream! {
            yield Ok(ChatChunk {
                content: Some(response.content.clone()),
                delta: Some(response.content.clone()),
                tool_calls: response.tool_calls.clone(),
                finish_reason: response.finish_reason.clone(),
                usage: Some(response.usage),
            });
        };

        Ok(Box::pin(converted_stream))
    }

    async fn count_tokens(&self, messages: &[Message]) -> Result<usize, ProviderError> {
        // Simple estimation: ~4 characters per token
        let total_chars: usize = messages.iter().map(|msg| self.extract_text_content(msg).len()).sum();
        Ok((total_chars + 3) / 4)
    }
}

