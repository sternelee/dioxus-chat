use super::base::*;
use crate::chat_service::{Message, MessageContent, ModelConfig, ProviderError, Role, Tool, Usage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde_json::{json, Value};
use std::pin::Pin;
use tracing::{debug, warn};

pub struct DeepSeekProvider {
    api_key: String,
    model_config: ModelConfig,
}

impl DeepSeekProvider {
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
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .or_else(|_| std::env::var("DEEPSEEK_API_KEY"))
            .map_err(|_| ProviderError {
                message: "DEEPSEEK_API_KEY environment variable not set".to_string(),
                code: Some("missing_api_key".to_string()),
                retry_after: None,
            })?;

        Ok(Self::new(api_key, model_config))
    }

    async fn call_deepseek_api(
        &self,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        // Create HTTP client
        let client = reqwest::Client::new();

        // Build request payload
        let mut payload = json!({
            "model": self.model_config.id,
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

        // DeepSeek supports thinking content for reasoning models
        if let Some(model_id) = self.model_config.id.strip_prefix("deepseek-r1") {
            // Add reasoning configuration for R1 models
            payload["reasoning_effort"] = json!("high"); // Can be "low", "medium", "high"
        }

        // Add tools if provided
        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                let deepseek_tools = tools
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

                payload["tools"] = json!(deepseek_tools);
                payload["tool_choice"] = json!("auto");
            }
        }

        debug!("Calling DeepSeek API for model: {}", self.model_config.id);

        // Make the request
        let response = client
            .post("https://api.deepseek.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError {
                message: format!("DeepSeek API request failed: {}", e),
                code: Some("network_error".to_string()),
                retry_after: None,
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError {
                message: format!("DeepSeek API error: {} - {}", status, error_text),
                code: Some("api_error".to_string()),
                retry_after: None,
            });
        }

        let response_json: Value = response.json().await.map_err(|e| ProviderError {
            message: format!("Failed to parse DeepSeek response: {}", e),
            code: Some("parse_error".to_string()),
            retry_after: None,
        })?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let reasoning_content = response_json["choices"][0]["message"]["reasoning_content"]
            .as_str()
            .map(|s| s.to_string());

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
            tool_results: None,
            reasoning_content,
        })
    }
}

#[async_trait]
impl Provider for DeepSeekProvider {
    fn metadata() -> ProviderMetadata
    where
        Self: Sized,
    {
        ProviderMetadata {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            description: "DeepSeek API provider for advanced reasoning models".to_string(),
            supports_streaming: true,
            supports_tools: true,
            supports_images: false,
            supports_audio: false,
            max_tokens: Some(64000),
            pricing: Some(PricingInfo {
                input_cost_per_1k: 0.00014, // DeepSeek pricing is very competitive
                output_cost_per_1k: 0.00028,
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
        self.call_deepseek_api(&request).await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        // For now, convert to non-streaming and return as a single chunk
        let response = self.call_deepseek_api(&request).await?;

        let converted_stream = async_stream::stream! {
            // If there's reasoning content, emit it first
            if let Some(reasoning) = &response.reasoning_content {
                yield Ok(ChatChunk {
                    content: None,
                    delta: Some(reasoning.clone()),
                    tool_calls: None,
                    finish_reason: Some("reasoning".to_string()),
                    usage: None,
                });
            }

            // Then emit the main content
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

    fn supports_tool_calling(&self) -> bool {
        // DeepSeek supports tool calling in most models
        !self.model_config.id.contains("deepseek-coder") // Coder models might not support tools
    }

    fn supports_streaming(&self) -> bool {
        // DeepSeek supports streaming
        true
    }

    async fn count_tokens(&self, messages: &[Message]) -> Result<usize, ProviderError> {
        // Simple estimation: ~4 characters per token
        let total_chars: usize = messages.iter().map(|msg| self.extract_text_content(msg).len()).sum();
        Ok((total_chars + 3) / 4)
    }

    async fn list_models(&self) -> Vec<crate::chat_service::ModelConfig> {
        vec![
            crate::chat_service::ModelConfig {
                id: "deepseek-chat".to_string(),
                name: "DeepSeek Chat".to_string(),
                provider: "deepseek".to_string(),
                description: Some("DeepSeek's chat model optimized for conversations".to_string()),
                context_limit: Some(64000),
                supports_tools: true,
                supports_streaming: true,
                supports_vision: false,
                supports_function_calling: true,
                pricing: Some(crate::chat_service::ModelPricing {
                    input_tokens: 0.00014,
                    output_tokens: 0.00028,
                    currency: "USD".to_string(),
                }),
            },
            crate::chat_service::ModelConfig {
                id: "deepseek-r1-distill-llama-70b".to_string(),
                name: "DeepSeek R1 Distill Llama 70B".to_string(),
                provider: "deepseek".to_string(),
                description: Some("DeepSeek's reasoning model with thinking capabilities".to_string()),
                context_limit: Some(64000),
                supports_tools: true,
                supports_streaming: true,
                supports_vision: false,
                supports_function_calling: true,
                pricing: Some(crate::chat_service::ModelPricing {
                    input_tokens: 0.00014,
                    output_tokens: 0.00028,
                    currency: "USD".to_string(),
                }),
            },
        ]
    }
}