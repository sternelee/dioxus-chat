// Streaming Service using Rig patterns
use anyhow::Result;
use chrono::Utc;
use futures::{Stream, StreamExt};
use serde_json::json;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;

use crate::chat_service_simple::{
    ChatMessage, ChatRequest, ChatResponse, Role, StreamChunk, TokenUsage,
};
use crate::rig_agent_service::RigAgentService;

/// Streaming configuration
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub chunk_delay_ms: u64,
    pub chunk_size: usize,
    pub enable_thinking_stream: bool,
    pub enable_tool_call_stream: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_delay_ms: 50,
            chunk_size: 10,
            enable_thinking_stream: true,
            enable_tool_call_stream: true,
        }
    }
}

/// Enhanced streaming chunk with more metadata
#[derive(Debug, Clone)]
pub struct EnhancedStreamChunk {
    pub base: StreamChunk,
    pub chunk_type: ChunkType,
    pub metadata: StreamMetadata,
}

#[derive(Debug, Clone)]
pub enum ChunkType {
    Content,
    Thinking,
    ToolCall,
    ToolResult,
    Metadata,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamMetadata {
    pub agent_name: String,
    pub iteration: usize,
    pub timestamp: chrono::DateTime<Utc>,
    pub agent_mode: String,
}

/// Streaming Agent Service
pub struct StreamingAgentService {
    agent_service: RigAgentService,
    config: StreamingConfig,
}

impl StreamingAgentService {
    pub fn new(agent_service: RigAgentService) -> Self {
        Self {
            agent_service,
            config: StreamingConfig::default(),
        }
    }

    pub fn with_config(mut self, config: StreamingConfig) -> Self {
        self.config = config;
        self
    }

    /// Stream a chat response with enhanced features
    pub async fn stream_chat_response(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = EnhancedStreamChunk> + Send>>> {
        let agent_name = self.get_agent_name(&request);
        let agent_mode = self.get_agent_mode(&request);
        let model_id = request.model.clone();

        // Generate the full response using the agent service
        let full_response = match self.agent_service.send_message(request).await {
            Ok(response) => response,
            Err(e) => {
                return Ok(Box::pin(futures::stream::once(async move {
                    EnhancedStreamChunk {
                        base: StreamChunk {
                            content: Some(format!("Error: {}", e)),
                            delta: None,
                            token_usage: None,
                            model: model_id,
                            finish_reason: Some("error".to_string()),
                            is_complete: true,
                        },
                        chunk_type: ChunkType::Error,
                        metadata: StreamMetadata {
                            agent_name,
                            iteration: 0,
                            timestamp: Utc::now(),
                            agent_mode,
                        },
                    }
                })));
            }
        };

        // Extract content and metadata
        let content = full_response
            .message
            .as_ref()
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        let thinking_content = full_response.thinking_content.clone();

        // Create streaming chunks
        let chunks = self
            .create_enhanced_chunks(
                content,
                thinking_content,
                agent_name,
                agent_mode,
                model_id,
                full_response.token_usage,
            )
            .await;

        Ok(Box::pin(futures::stream::iter(chunks)))
    }

    /// Stream with Server-Sent Events format
    pub async fn stream_chat_sse(
        &self,
        request: ChatRequest,
    ) -> Result<impl Stream<Item = String>> {
        let stream = self.stream_chat_response(request).await?;

        let sse_stream = stream.map(|chunk| match chunk.chunk_type {
            ChunkType::Content => {
                format!(
                    "data: {}\n\n",
                    serde_json::json!({
                        "type": "content",
                        "content": chunk.base.content,
                        "delta": chunk.base.delta,
                        "finish_reason": chunk.base.finish_reason,
                        "is_complete": chunk.base.is_complete,
                        "metadata": chunk.metadata
                    })
                )
            }
            ChunkType::Thinking => {
                format!(
                    "data: {}\n\n",
                    serde_json::json!({
                        "type": "thinking",
                        "content": chunk.base.content,
                        "metadata": chunk.metadata
                    })
                )
            }
            ChunkType::ToolCall => {
                format!(
                    "data: {}\n\n",
                    serde_json::json!({
                        "type": "tool_call",
                        "content": chunk.base.content,
                        "metadata": chunk.metadata
                    })
                )
            }
            ChunkType::ToolResult => {
                format!(
                    "data: {}\n\n",
                    serde_json::json!({
                        "type": "tool_result",
                        "content": chunk.base.content,
                        "metadata": chunk.metadata
                    })
                )
            }
            ChunkType::Metadata => {
                format!(
                    "data: {}\n\n",
                    serde_json::json!({
                        "type": "metadata",
                        "metadata": chunk.metadata
                    })
                )
            }
            ChunkType::Error => {
                format!(
                    "data: {}\n\n",
                    serde_json::json!({
                        "type": "error",
                        "content": chunk.base.content,
                        "metadata": chunk.metadata
                    })
                )
            }
        });

        Ok(sse_stream)
    }

    /// Stream with tool execution visualization
    pub async fn stream_chat_with_tools(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = EnhancedStreamChunk> + Send>>> {
        let mut chunks = Vec::new();

        // Start with agent metadata
        let agent_name = self.get_agent_name(&request);
        let agent_mode = self.get_agent_mode(&request);

        chunks.push(EnhancedStreamChunk {
            base: StreamChunk {
                content: Some(format!(
                    "Starting {} agent in {} mode",
                    agent_name, agent_mode
                )),
                delta: None,
                token_usage: None,
                model: request.model.clone(),
                finish_reason: None,
                is_complete: false,
            },
            chunk_type: ChunkType::Metadata,
            metadata: StreamMetadata {
                agent_name: agent_name.clone(),
                iteration: 0,
                timestamp: Utc::now(),
                agent_mode: agent_mode.clone(),
            },
        });

        // If tools are available, add tool information
        if request.tools.as_ref().map_or(false, |t| !t.is_empty()) {
            let tool_names: Vec<String> = request
                .tools
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|t| t.name.clone())
                .collect();

            chunks.push(EnhancedStreamChunk {
                base: StreamChunk {
                    content: Some(format!("Available tools: {}", tool_names.join(", "))),
                    delta: None,
                    token_usage: None,
                    model: request.model.clone(),
                    finish_reason: None,
                    is_complete: false,
                },
                chunk_type: ChunkType::Metadata,
                metadata: StreamMetadata {
                    agent_name,
                    iteration: 1,
                    timestamp: Utc::now(),
                    agent_mode,
                },
            });
        }

        // Continue with regular streaming
        let mut stream = self.stream_chat_response(request).await?;
        while let Some(chunk) = stream.next().await {
            chunks.push(chunk);
        }

        Ok(Box::pin(futures::stream::iter(chunks)))
    }

    async fn create_enhanced_chunks(
        &self,
        content: String,
        thinking_content: Option<String>,
        agent_name: String,
        agent_mode: String,
        model_id: String,
        token_usage: Option<TokenUsage>,
    ) -> Vec<EnhancedStreamChunk> {
        let mut chunks = Vec::new();
        let mut iteration = 0;

        // Add thinking chunks if available and enabled
        if let Some(ref thinking) = thinking_content {
            if self.config.enable_thinking_stream {
                let thinking_words: Vec<String> =
                    thinking.split_whitespace().map(|s| s.to_string()).collect();

                for (i, word) in thinking_words.iter().enumerate() {
                    let is_complete_thinking = i == thinking_words.len() - 1;
                    chunks.push(EnhancedStreamChunk {
                        base: StreamChunk {
                            content: Some(format!("{} ", word)),
                            delta: Some(format!("{} ", word)),
                            token_usage: None,
                            model: model_id.clone(),
                            finish_reason: None,
                            is_complete: false,
                        },
                        chunk_type: ChunkType::Thinking,
                        metadata: StreamMetadata {
                            agent_name: agent_name.clone(),
                            iteration,
                            timestamp: Utc::now(),
                            agent_mode: agent_mode.clone(),
                        },
                    });
                }

                // Add transition marker
                chunks.push(EnhancedStreamChunk {
                    base: StreamChunk {
                        content: Some("\n\n--- Response ---\n\n".to_string()),
                        delta: Some("\n\n--- Response ---\n\n".to_string()),
                        token_usage: None,
                        model: model_id.clone(),
                        finish_reason: None,
                        is_complete: false,
                    },
                    chunk_type: ChunkType::Metadata,
                    metadata: StreamMetadata {
                        agent_name: agent_name.clone(),
                        iteration,
                        timestamp: Utc::now(),
                        agent_mode: agent_mode.clone(),
                    },
                });
            }
        }

        // Split content into chunks for streaming
        let words: Vec<String> = content.split_whitespace().map(|s| s.to_string()).collect();
        let words_len = words.len();

        for (index, word) in words.into_iter().enumerate() {
            iteration += 1;
            let is_complete = index == words_len - 1;

            chunks.push(EnhancedStreamChunk {
                base: StreamChunk {
                    content: Some(format!("{} ", word)),
                    delta: Some(format!("{} ", word)),
                    token_usage: if is_complete {
                        token_usage.clone()
                    } else {
                        None
                    },
                    model: model_id.clone(),
                    finish_reason: if is_complete {
                        Some("stop".to_string())
                    } else {
                        None
                    },
                    is_complete,
                },
                chunk_type: ChunkType::Content,
                metadata: StreamMetadata {
                    agent_name: agent_name.clone(),
                    iteration,
                    timestamp: Utc::now(),
                    agent_mode: agent_mode.clone(),
                },
            });
        }

        chunks
    }

    fn get_agent_name(&self, request: &ChatRequest) -> String {
        if let Some(ref agent_config) = request.agent_config {
            match agent_config.goose_mode {
                crate::chat_service_simple::GooseMode::Agent => "Agent".to_string(),
                crate::chat_service_simple::GooseMode::Chat => "Chat".to_string(),
                crate::chat_service_simple::GooseMode::Auto => "Auto".to_string(),
            }
        } else {
            "Assistant".to_string()
        }
    }

    fn get_agent_mode(&self, request: &ChatRequest) -> String {
        if let Some(ref agent_config) = request.agent_config {
            format!("{:?}", agent_config.goose_mode)
        } else {
            "Chat".to_string()
        }
    }

    /// Create a delayed stream for realistic typing effect
    async fn create_delayed_stream(
        chunks: Vec<EnhancedStreamChunk>,
        delay_ms: u64,
    ) -> impl Stream<Item = EnhancedStreamChunk> {
        futures::stream::iter(chunks)
            .map(move |chunk| async move {
                sleep(Duration::from_millis(delay_ms)).await;
                chunk
            })
            .buffer_unordered(1)
    }

    /// Stream with real-time delay
    pub async fn stream_with_delay(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = EnhancedStreamChunk> + Send>>> {
        let chunks = self
            .create_enhanced_chunks(
                "Response will be streamed with delay".to_string(),
                None,
                self.get_agent_name(&request),
                self.get_agent_mode(&request),
                request.model.clone(),
                None,
            )
            .await;

        Ok(Box::pin(
            Self::create_delayed_stream(chunks, self.config.chunk_delay_ms).await,
        ))
    }
}

impl Clone for StreamingAgentService {
    fn clone(&self) -> Self {
        Self {
            agent_service: self.agent_service.clone(),
            config: self.config.clone(),
        }
    }
}

