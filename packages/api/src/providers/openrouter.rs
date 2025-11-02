use super::base::*;
use crate::chat_service::{Message, MessageContent, ModelConfig, ProviderError, Role, Tool, Usage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde_json::{json, Value};
use std::pin::Pin;
use tracing::{debug, warn};

pub struct OpenRouterProvider {
    api_key: String,
    model_config: ModelConfig,
}

impl OpenRouterProvider {
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
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .or_else(|_| std::env::var("OPENROUTER_API_KEY"))
            .map_err(|_| ProviderError {
                message: "OPENROUTER_API_KEY environment variable not set".to_string(),
                code: Some("missing_api_key".to_string()),
                retry_after: None,
            })?;

        Ok(Self::new(api_key, model_config))
    }

    async fn call_openrouter_api(
        &self,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        // Create HTTP client
        let client = reqwest::Client::new();

        // Build request payload
        let mut payload = json!({
            "model": self.model_config.id,
            "messages": []
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
                Role::Tool => "tool",
            };

            // Handle tool messages specially for OpenRouter
            if msg.role == Role::Tool {
                if let MessageContent::ToolResponse { name, result, .. } = &msg.content {
                    messages.push(json!({
                        "role": "tool",
                        "content": serde_json::to_string(result).unwrap_or_default(),
                        "tool_call_id": name, // OpenRouter uses name as tool_call_id for simplicity
                    }));
                }
                continue;
            }

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

        // OpenRouter supports additional routing parameters
        payload["route"] = json!("fallback"); // Can be "fallback", "data", "speed"

        // Add models for fallback routing
        if self.model_config.provider == "openrouter" {
            // Set fallback models based on the primary model
            let fallback_models = match self.model_config.id.as_str() {
                "anthropic/claude-3.5-sonnet" => vec![
                    "openai/gpt-4o",
                    "google/gemini-1.5-pro",
                ],
                "openai/gpt-4o" => vec![
                    "anthropic/claude-3.5-sonnet",
                    "google/gemini-1.5-pro",
                ],
                _ => vec![
                    "openai/gpt-4o",
                    "anthropic/claude-3.5-sonnet",
                ],
            };
            payload["models"] = json!(fallback_models);
        }

        // Add tools if provided
        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                let openrouter_tools = tools
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

                payload["tools"] = json!(openrouter_tools);
                payload["tool_choice"] = json!("auto");
            }
        }

        // Add headers for OpenRouter
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("HTTP-Referer", "https://dioxus-chat.com".parse().unwrap());
        headers.insert("X-Title", "Dioxus Chat".parse().unwrap());

        debug!("Calling OpenRouter API for model: {}", self.model_config.id);

        // Make the request
        let response = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError {
                message: format!("OpenRouter API request failed: {}", e),
                code: Some("network_error".to_string()),
                retry_after: None,
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError {
                message: format!("OpenRouter API error: {} - {}", status, error_text),
                code: Some("api_error".to_string()),
                retry_after: None,
            });
        }

        let response_json: Value = response.json().await.map_err(|e| ProviderError {
            message: format!("Failed to parse OpenRouter response: {}", e),
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
impl Provider for OpenRouterProvider {
    fn metadata() -> ProviderMetadata
    where
        Self: Sized,
    {
        ProviderMetadata {
            id: "openrouter".to_string(),
            name: "OpenRouter".to_string(),
            description: "OpenRouter API provider - unified access to multiple AI models".to_string(),
            supports_streaming: true,
            supports_tools: true,
            supports_images: true,
            supports_audio: false,
            max_tokens: Some(200000), // OpenRouter supports large context windows
            pricing: Some(PricingInfo {
                input_cost_per_1k: 0.001, // Varies by model, this is an average
                output_cost_per_1k: 0.002,
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
        self.call_openrouter_api(&request).await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        // For now, convert to non-streaming and return as a single chunk
        let response = self.call_openrouter_api(&request).await?;

        let converted_stream = async_stream::stream! {
            // If there's reasoning content, emit it first
            if let Some(reasoning) = &response.reasoning_content {
                yield Ok(ChatChunk {
                    content: None,
                    delta: Some(format!("🧠 **Thinking:**\n{}", reasoning)),
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
        // OpenRouter supports tool calling for most models
        !self.model_config.id.contains("llava") && !self.model_config.id.contains("stable-diffusion")
    }

    fn supports_streaming(&self) -> bool {
        // OpenRouter supports streaming
        true
    }

    fn supports_images(&self) -> bool {
        // OpenRouter supports vision for compatible models
        self.model_config.id.contains("vision") ||
        self.model_config.id.contains("claude-3") ||
        self.model_config.id.contains("gpt-4o") ||
        self.model_config.id.contains("gemini")
    }

    async fn count_tokens(&self, messages: &[Message]) -> Result<usize, ProviderError> {
        // Simple estimation: ~4 characters per token
        let total_chars: usize = messages.iter().map(|msg| self.extract_text_content(msg).len()).sum();
        Ok((total_chars + 3) / 4)
    }

    async fn list_models(&self) -> Vec<crate::chat_service::ModelConfig> {
        vec![
            // Popular OpenRouter models
            crate::chat_service::ModelConfig {
                id: "anthropic/claude-3.5-sonnet".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                provider: "openrouter".to_string(),
                description: Some("Anthropic's most intelligent model".to_string()),
                context_limit: Some(200000),
                supports_tools: true,
                supports_streaming: true,
                supports_vision: true,
                supports_function_calling: true,
                pricing: Some(crate::chat_service::ModelPricing {
                    input_tokens: 0.003,
                    output_tokens: 0.015,
                    currency: "USD".to_string(),
                }),
            },
            crate::chat_service::ModelConfig {
                id: "openai/gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                provider: "openrouter".to_string(),
                description: Some("OpenAI's flagship multimodal model".to_string()),
                context_limit: Some(128000),
                supports_tools: true,
                supports_streaming: true,
                supports_vision: true,
                supports_function_calling: true,
                pricing: Some(crate::chat_service::ModelPricing {
                    input_tokens: 0.005,
                    output_tokens: 0.015,
                    currency: "USD".to_string(),
                }),
            },
            crate::chat_service::ModelConfig {
                id: "google/gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                provider: "openrouter".to_string(),
                description: Some("Google's advanced multimodal model".to_string()),
                context_limit: Some(2000000), // 2M context window
                supports_tools: true,
                supports_streaming: true,
                supports_vision: true,
                supports_function_calling: true,
                pricing: Some(crate::chat_service::ModelPricing {
                    input_tokens: 0.00125,
                    output_tokens: 0.00375,
                    currency: "USD".to_string(),
                }),
            },
            crate::chat_service::ModelConfig {
                id: "meta-llama/llama-3.1-70b-instruct".to_string(),
                name: "Llama 3.1 70B Instruct".to_string(),
                provider: "openrouter".to_string(),
                description: Some("Meta's open-source instruction model".to_string()),
                context_limit: Some(131072),
                supports_tools: true,
                supports_streaming: true,
                supports_vision: false,
                supports_function_calling: true,
                pricing: Some(crate::chat_service::ModelPricing {
                    input_tokens: 0.00088,
                    output_tokens: 0.00088,
                    currency: "USD".to_string(),
                }),
            },
            // DeepSeek models via OpenRouter
            crate::chat_service::ModelConfig {
                id: "deepseek/deepseek-r1-distill-llama-70b".to_string(),
                name: "DeepSeek R1 Distill (via OpenRouter)".to_string(),
                provider: "openrouter".to_string(),
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