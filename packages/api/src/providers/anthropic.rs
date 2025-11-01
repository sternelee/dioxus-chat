use super::base::*;
use crate::chat_service::{Message, MessageContent, ModelConfig, ProviderError, Role, Tool, Usage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde_json::{json, Value};
use std::pin::Pin;
use tracing::{debug, warn};

pub struct AnthropicProvider {
    api_key: String,
    model_config: ModelConfig,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model_config: ModelConfig) -> Self {
        Self {
            api_key,
            model_config,
        }
    }

    pub fn from_config(model_config: ModelConfig) -> Result<Self, ProviderError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| ProviderError {
            message: "ANTHROPIC_API_KEY environment variable not set".to_string(),
            code: Some("missing_api_key".to_string()),
        })?;

        Ok(Self::new(api_key, model_config))
    }

    async fn call_anthropic_api(
        &self,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        // Create HTTP client
        let client = reqwest::Client::new();

        // Build request payload
        let mut payload = json!({
            "model": self.model_config.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "messages": []
        });

        // Convert messages
        let mut messages = Vec::new();
        for msg in &request.messages {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::System => continue, // System messages handled separately
                Role::Tool => continue,   // Skip tool messages for now
            };

            let content = msg.as_concat_text();
            if !content.is_empty() {
                messages.push(json!({
                    "role": role,
                    "content": content
                }));
            }
        }

        payload["messages"] = json!(messages);

        // Add system message if provided
        if let Some(system) = &request.system {
            payload["system"] = json!(system);
        }

        // Add parameters
        if let Some(temp) = request.temperature {
            payload["temperature"] = json!(temp);
        }
        if let Some(top_p) = request.top_p {
            payload["top_p"] = json!(top_p);
        }

        // Add tools if provided
        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                let anthropic_tools = tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "name": tool.name,
                            "description": tool.description,
                            "input_schema": tool.parameters
                        })
                    })
                    .collect::<Vec<_>>();

                payload["tools"] = json!(anthropic_tools);
            }
        }

        debug!(
            "Calling Anthropic API for model: {}",
            self.model_config.model
        );

        // Make the request
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError {
                message: format!("Anthropic API request failed: {}", e),
                code: Some("network_error".to_string()),
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError {
                message: format!("Anthropic API error: {} - {}", status, error_text),
                code: Some("api_error".to_string()),
            });
        }

        let response_json: Value = response.json().await.map_err(|e| ProviderError {
            message: format!("Failed to parse Anthropic response: {}", e),
            code: Some("parse_error".to_string()),
        })?;

        let content = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tool_calls = response_json["content"].as_array().and_then(|content| {
            let mut calls = Vec::new();
            for item in content {
                if let Some(tool_use) = item.get("tool_use") {
                    if let (Some(id), Some(name), Some(input)) = (
                        tool_use.get("id").and_then(|v| v.as_str()),
                        tool_use.get("name").and_then(|v| v.as_str()),
                        tool_use.get("input"),
                    ) {
                        calls.push(ToolCall {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments: input.clone(),
                        });
                    }
                }
            }
            if calls.is_empty() {
                None
            } else {
                Some(calls)
            }
        });

        let usage = response_json["usage"]
            .as_object()
            .map(|usage| Usage {
                prompt_tokens: usage["input_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage["output_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: (usage["input_tokens"].as_u64().unwrap_or(0)
                    + usage["output_tokens"].as_u64().unwrap_or(0))
                    as u32,
            })
            .unwrap_or(Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            });

        let finish_reason = response_json["stop_reason"].as_str().map(|s| s.to_string());

        Ok(CompletionResponse {
            content,
            usage,
            finish_reason,
            tool_calls,
        })
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn metadata() -> ProviderMetadata
    where
        Self: Sized,
    {
        ProviderMetadata {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            description: "Official Anthropic API provider for Claude models".to_string(),
            supports_streaming: true,
            supports_tools: true,
            supports_images: true,
            supports_audio: false,
            max_tokens: Some(200000),
            pricing: Some(PricingInfo {
                input_cost_per_1k: 0.003,
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
        self.call_anthropic_api(&request).await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        // For now, convert to non-streaming and return as a single chunk
        let response = self.call_anthropic_api(&request).await?;

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
        let total_chars: usize = messages.iter().map(|msg| msg.as_concat_text().len()).sum();
        Ok((total_chars + 3) / 4)
    }
}

