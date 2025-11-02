use super::base::*;
use crate::chat_service::{Message, MessageContent, ModelConfig, ProviderError, Role, Tool, Usage};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::{Client as HttpClient, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;
use tracing::{debug, error};

pub struct OllamaProvider {
    client: HttpClient,
    model_config: ModelConfig,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    images: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: Option<OllamaOptions>,
    tools: Option<Vec<OllamaTool>>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
    top_p: Option<f32>,
    num_predict: Option<u32>,
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OllamaToolFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaResponseMessage,
    done: bool,
    #[serde(default)]
    usage: OllamaUsage,
}

#[derive(Debug, Deserialize, Default)]
struct OllamaResponseMessage {
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Vec<OllamaToolCall>,
}

#[derive(Debug, Deserialize)]
struct OllamaToolCall {
    function: OllamaToolCallFunction,
}

#[derive(Debug, Deserialize)]
struct OllamaToolCallFunction {
    name: String,
    arguments: Value,
}

#[derive(Debug, Deserialize, Default)]
struct OllamaUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    #[serde(default)]
    message: OllamaResponseMessage,
    done: bool,
    #[serde(default)]
    usage: OllamaUsage,
}

impl OllamaProvider {
    pub fn new(base_url: String, model_config: ModelConfig) -> Self {
        let client = HttpClient::new();

        Self {
            client,
            model_config,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn from_config(model_config: ModelConfig) -> Result<Self, ProviderError> {
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        Ok(Self::new(base_url, model_config))
    }

    fn convert_message_to_ollama(&self, message: &Message) -> Result<OllamaMessage, ProviderError> {
        let role = match message.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
            Role::Tool => "tool",
        };

        let mut content = String::new();
        let mut images = Vec::new();

        match &message.content {
            MessageContent::Text { text } => {
                content.push_str(text);
            }
            MessageContent::ToolRequest {
                name, arguments, ..
            } => {
                // For Ollama, we include tool requests as text content
                let tool_json = json!({
                    "tool_call": {
                        "name": name,
                        "arguments": arguments
                    }
                });
                content.push_str(&format!("Tool call: {}", tool_json));
            }
            MessageContent::ToolResponse { result, .. } => {
                // Include tool results as text content
                content.push_str(&format!("Tool result: {}", result));
            }
            MessageContent::Image { url, .. } => {
                if url.starts_with("data:image/") {
                    if let Some(base64_data) = self.extract_base64_from_url(url) {
                        images.push(base64_data);
                    }
                }
            }
        }

        Ok(OllamaMessage {
            role: role.to_string(),
            content,
            images: if images.is_empty() {
                None
            } else {
                Some(images)
            },
        })
    }

    fn convert_tools_to_ollama(&self, tools: &[Tool]) -> Vec<OllamaTool> {
        tools
            .iter()
            .map(|tool| OllamaTool {
                tool_type: "function".to_string(),
                function: OllamaToolFunction {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.input_schema.clone(),
                },
            })
            .collect()
    }

    fn extract_base64_from_url(&self, url: &str) -> Option<String> {
        if url.starts_with("data:image/") {
            url.find(',')
                .map(|comma_pos| url[comma_pos + 1..].to_string())
        } else {
            None
        }
    }

    fn convert_response_from_ollama(&self, response: OllamaResponse) -> CompletionResponse {
        let tool_calls = if response.message.tool_calls.is_empty() {
            None
        } else {
            Some(
                response
                    .message
                    .tool_calls
                    .iter()
                    .map(|tc| {
                        ToolCall {
                            id: uuid::Uuid::new_v4().to_string(), // Ollama doesn't provide tool call IDs
                            name: tc.function.name.clone(),
                            arguments: tc.function.arguments.clone(),
                        }
                    })
                    .collect(),
            )
        };

        let usage = Usage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
        };

        CompletionResponse {
            content: response.message.content,
            usage,
            finish_reason: if response.done {
                Some("stop".to_string())
            } else {
                None
            },
            tool_calls,
            tool_results: None,
            reasoning_content: None,
        }
    }

    async fn call_ollama(&self, request: OllamaRequest) -> Result<OllamaResponse, ProviderError> {
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError {
                message: format!("Failed to call Ollama API: {}", e),
                code: Some("network_error".to_string()),
                retry_after: None,
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError {
                message: format!("Ollama API error: {} - {}", status, error_text),
                code: Some("api_error".to_string()),
                retry_after: None,
            });
        }

        response
            .json::<OllamaResponse>()
            .await
            .map_err(|e| ProviderError {
                message: format!("Failed to parse Ollama response: {}", e),
                code: Some("parse_error".to_string()),
                retry_after: None,
            })
    }

    async fn stream_ollama(
        &self,
        request: OllamaRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError {
                message: format!("Failed to start Ollama stream: {}", e),
                code: Some("network_error".to_string()),
                retry_after: None,
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError {
                message: format!("Ollama streaming error: {} - {}", status, error_text),
                code: Some("stream_error".to_string()),
                retry_after: None,
            });
        }

        let byte_stream = response.bytes_stream();
        let converted_stream = async_stream::stream! {
            use futures::StreamExt;
            let mut stream = byte_stream;
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&chunk_str);

                        // Process complete JSON objects
                        while let Some(newline_pos) = buffer.find('\n') {
                            let json_str = &buffer[..newline_pos].trim();
                            let remaining = buffer[newline_pos + 1..].to_string();

                            if json_str.is_empty() {
                                buffer = remaining;
                                continue;
                            }

                            match serde_json::from_str::<OllamaStreamChunk>(json_str) {
                                Ok(ollama_chunk) => {
                                    let tool_calls = if ollama_chunk.message.tool_calls.is_empty() {
                                        None
                                    } else {
                                        Some(
                                            ollama_chunk.message.tool_calls.iter().map(|tc| {
                                                ToolCall {
                                                    id: uuid::Uuid::new_v4().to_string(),
                                                    name: tc.function.name.clone(),
                                                    arguments: tc.function.arguments.clone(),
                                                }
                                            }).collect()
                                        )
                                    };

                                    yield Ok(ChatChunk {
                                        content: Some(ollama_chunk.message.content.clone()),
                                        delta: Some(ollama_chunk.message.content.clone()),
                                        tool_calls,
                                        finish_reason: if ollama_chunk.done { Some("stop".to_string()) } else { None },
                                        usage: if ollama_chunk.done {
                                            Some(Usage {
                                                prompt_tokens: ollama_chunk.usage.prompt_tokens,
                                                completion_tokens: ollama_chunk.usage.completion_tokens,
                                                total_tokens: ollama_chunk.usage.total_tokens,
                                            })
                                        } else { None },
                                    });

                                    if ollama_chunk.done {
                                        return;
                                    }
                                    buffer = remaining;
                                }
                                Err(e) => {
                                    error!("Failed to parse Ollama chunk: {} - JSON: {}", e, json_str);
                                    yield Err(ProviderError {
                                        message: format!("Failed to parse chunk: {}", e),
                                        code: Some("parse_error".to_string()),
                                        retry_after: None,
                                    });
                                    buffer = remaining;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Ollama stream error: {}", e);
                        yield Err(ProviderError {
                            message: format!("Stream error: {}", e),
                            code: Some("stream_error".to_string()),
                            retry_after: None,
                        });
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(converted_stream))
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn metadata() -> ProviderMetadata
    where
        Self: Sized,
    {
        ProviderMetadata {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            description: "Local Ollama server for running open-source models".to_string(),
            supports_streaming: true,
            supports_tools: true,
            supports_images: true,
            supports_audio: false,
            max_tokens: None, // Depends on the model
            pricing: None,    // Free and local
        }
    }

    fn model_config(&self) -> &ModelConfig {
        &self.model_config
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        debug!(
            "Ollama completion request for model: {}",
            self.model_config.id
        );

        let messages: Result<Vec<_>, _> = request
            .messages
            .iter()
            .map(|msg| self.convert_message_to_ollama(msg))
            .collect();
        let messages = messages?;

        let mut options = OllamaOptions {
            temperature: request.temperature,
            top_p: request.top_p,
            num_predict: request.max_tokens,
            stop: request.stop,
        };

        // Only include non-None options
        let options_param = if options.temperature.is_some()
            || options.top_p.is_some()
            || options.num_predict.is_some()
            || options.stop.is_some()
        {
            Some(options)
        } else {
            None
        };

        let mut ollama_request = OllamaRequest {
            model: self.model_config.id.clone(),
            messages,
            stream: false,
            options: options_param,
            tools: None,
        };

        // Add system message
        if let Some(system) = request.system {
            ollama_request.messages.insert(
                0,
                OllamaMessage {
                    role: "system".to_string(),
                    content: system,
                    images: None,
                },
            );
        }

        // Add tools if provided
        if let Some(tools) = request.tools {
            if !tools.is_empty() {
                ollama_request.tools = Some(self.convert_tools_to_ollama(&tools));
            }
        }

        let response = self.call_ollama(ollama_request).await?;
        Ok(self.convert_response_from_ollama(response))
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        debug!(
            "Ollama stream request for model: {}",
            self.model_config.id
        );

        let messages: Result<Vec<_>, _> = request
            .messages
            .iter()
            .map(|msg| self.convert_message_to_ollama(msg))
            .collect();
        let messages = messages?;

        let mut options = OllamaOptions {
            temperature: request.temperature,
            top_p: request.top_p,
            num_predict: request.max_tokens,
            stop: request.stop,
        };

        // Only include non-None options
        let options_param = if options.temperature.is_some()
            || options.top_p.is_some()
            || options.num_predict.is_some()
            || options.stop.is_some()
        {
            Some(options)
        } else {
            None
        };

        let mut ollama_request = OllamaRequest {
            model: self.model_config.id.clone(),
            messages,
            stream: true,
            options: options_param,
            tools: None,
        };

        // Add system message
        if let Some(system) = request.system {
            ollama_request.messages.insert(
                0,
                OllamaMessage {
                    role: "system".to_string(),
                    content: system,
                    images: None,
                },
            );
        }

        // Add tools if provided
        if let Some(tools) = request.tools {
            if !tools.is_empty() {
                ollama_request.tools = Some(self.convert_tools_to_ollama(&tools));
            }
        }

        self.stream_ollama(ollama_request).await
    }

    async fn validate_request(&self, request: &CompletionRequest) -> Result<(), ProviderError> {
        // Check if Ollama server is available
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => Ok(()),
            Ok(_) => Err(ProviderError {
                message: "Ollama server is not responding correctly".to_string(),
                code: Some("server_unavailable".to_string()),
                retry_after: None,
            }),
            Err(e) => Err(ProviderError {
                message: format!("Cannot connect to Ollama server: {}", e),
                code: Some("connection_error".to_string()),
                retry_after: None,
            }),
        }
    }
}

