// Multimodal Support for Rig Agents
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use rig::{
    completion::ToolDefinition,
    tool::Tool,
};
use base64::{Engine as _, engine::general_purpose};
use tokio::fs;
use std::path::PathBuf;

use crate::{ChatMessage, MessageContent, Role, ToolCall, ToolResult, Tool as ApiTool};

/// Supported media types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MediaType {
    Image,
    Audio,
    Video,
    Document,
}

/// Media content with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaContent {
    pub media_type: MediaType,
    pub content: MediaData,
    pub metadata: MediaMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaData {
    Base64(String),
    Url(String),
    FilePath(PathBuf),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaMetadata {
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<usize>,
    pub dimensions: Option<MediaDimensions>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaDimensions {
    pub width: u32,
    pub height: u32,
}

/// Multimodal message content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultimodalMessage {
    pub role: Role,
    pub content: Vec<MultimodalContent>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MultimodalContent {
    Text(String),
    Media(MediaContent),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
}

/// Multimodal processing service
#[derive(Debug)]
pub struct MultimodalService {
    supported_formats: HashMap<MediaType, Vec<String>>,
    max_file_size: usize,
}

impl MultimodalService {
    pub fn new() -> Self {
        let mut supported_formats = HashMap::new();

        supported_formats.insert(MediaType::Image, vec![
            "image/jpeg".to_string(),
            "image/png".to_string(),
            "image/gif".to_string(),
            "image/webp".to_string(),
            "image/svg+xml".to_string(),
        ]);

        supported_formats.insert(MediaType::Audio, vec![
            "audio/mpeg".to_string(),
            "audio/wav".to_string(),
            "audio/ogg".to_string(),
            "audio/mp3".to_string(),
        ]);

        supported_formats.insert(MediaType::Video, vec![
            "video/mp4".to_string(),
            "video/webm".to_string(),
            "video/ogg".to_string(),
        ]);

        supported_formats.insert(MediaType::Document, vec![
            "application/pdf".to_string(),
            "text/plain".to_string(),
            "text/markdown".to_string(),
            "application/json".to_string(),
        ]);

        Self {
            supported_formats,
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Validate media content
    pub async fn validate_media(&self, content: &MediaContent) -> Result<()> {
        match &content.content {
            MediaData::Raw(data) => {
                if data.len() > self.max_file_size {
                    return Err(anyhow::anyhow!("File size exceeds limit of {} bytes", self.max_file_size));
                }
            },
            MediaData::FilePath(path) => {
                let metadata = fs::metadata(path).await?;
                if metadata.len() > self.max_file_size as u64 {
                    return Err(anyhow::anyhow!("File size exceeds limit"));
                }
            },
            _ => {}
        }

        // Validate MIME type if provided
        if let Some(ref mime_type) = content.metadata.mime_type {
            let supported = self.supported_formats
                .get(&content.media_type)
                .map(|formats| formats.contains(mime_type))
                .unwrap_or(false);

            if !supported {
                return Err(anyhow::anyhow!("Unsupported media type: {}", mime_type));
            }
        }

        Ok(())
    }

    /// Convert media to base64 for API transmission
    pub async fn to_base64(&self, content: &MediaContent) -> Result<String> {
        let data = match &content.content {
            MediaData::Base64(data) => return Ok(data.clone()),
            MediaData::Raw(raw) => raw.clone(),
            MediaData::FilePath(path) => fs::read(path).await?,
            MediaData::Url(url) => {
                // For URLs, we would need to download the content
                // For now, return the URL as-is
                return Ok(url.clone());
            }
        };

        let engine = general_purpose::STANDARD;
        Ok(engine.encode(&data))
    }

    /// Detect media type from content
    pub fn detect_media_type(&self, data: &[u8]) -> Option<(MediaType, String)> {
        // Basic magic number detection
        if data.len() < 4 {
            return None;
        }

        match &data[0..4] {
            [0xFF, 0xD8, 0xFF] => Some((MediaType::Image, "image/jpeg".to_string())),
            [0x89, 0x50, 0x4E, 0x47] => Some((MediaType::Image, "image/png".to_string())),
            [0x47, 0x49, 0x46] => Some((MediaType::Image, "image/gif".to_string())),
            [0x25, 0x50, 0x44, 0x46] => Some((MediaType::Document, "application/pdf".to_string())),
            [0x49, 0x44, 0x33] => Some((MediaType::Audio, "audio/mp3".to_string())),
            _ => None,
        }
    }

    /// Process image with AI vision models
    pub async fn analyze_image(&self, image_data: &[u8], prompt: &str) -> Result<String> {
        // This would integrate with vision-capable models like GPT-4V, Claude 3.5, etc.
        // For now, return a mock response
        Ok(format!("Analyzed image ({} bytes) with prompt: '{}'. Vision analysis would be implemented with actual AI model.", image_data.len(), prompt))
    }

    /// Transcribe audio to text
    pub async fn transcribe_audio(&self, audio_data: &[u8]) -> Result<String> {
        // This would integrate with speech-to-text APIs
        // For now, return a mock response
        Ok(format!("Transcribed audio ({} bytes). Speech-to-text would be implemented with actual audio processing.", audio_data.len()))
    }

    /// Extract text from document
    pub async fn extract_text(&self, document_data: &[u8], mime_type: &str) -> Result<String> {
        match mime_type {
            "text/plain" | "text/markdown" => {
                Ok(String::from_utf8(document_data.to_vec())?)
            },
            "application/pdf" => {
                // This would use a PDF extraction library
                Ok(format!("Extracted text from PDF ({} bytes). PDF processing would be implemented with actual PDF library.", document_data.len()))
            },
            _ => Ok(format!("Document processing for {} not implemented yet.", mime_type))
        }
    }
}

impl Default for MultimodalService {
    fn default() -> Self {
        Self::new()
    }
}

/// Vision analysis tool
#[derive(Debug)]
pub struct VisionAnalysisTool {
    multimodal_service: MultimodalService,
}

#[async_trait]
impl Tool for VisionAnalysisTool {
    const NAME: &'static str = "vision_analyze";
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = VisionAnalysisArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> Result<ToolDefinition, Self::Error> {
        Ok(ToolDefinition {
            name: "vision_analyze".to_string(),
            description: "Analyze images and provide visual descriptions".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "image": {
                        "type": "string",
                        "description": "Base64 encoded image data or file path"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "What to analyze in the image"
                    }
                },
                "required": ["image", "prompt"]
            }),
        })
    }

    async fn call(&self, args: serde_json::Value) -> Result<Self::Output, Self::Error> {
        let args: VisionAnalysisArgs = serde_json::from_value(args)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let image_data = if args.image.starts_with('/') {
            // File path
            fs::read(&args.image).await?
        } else if args.image.contains(',') {
            // Base64 data URL
            let parts: Vec<&str> = args.image.split(',').collect();
            if parts.len() != 2 {
                return Err("Invalid data URL format".into());
            }
            use base64::{Engine as _, engine::general_purpose};
            general_purpose::STANDARD.decode(parts[1])?
        } else {
            // Assume base64 string
            use base64::{Engine as _, engine::general_purpose};
            general_purpose::STANDARD.decode(&args.image)?
        };

        self.multimodal_service.analyze_image(&image_data, &args.prompt).await.map_err(|e| e.into())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct VisionAnalysisArgs {
    pub image: String,
    pub prompt: String,
}

/// Speech-to-text tool
#[derive(Debug)]
pub struct SpeechToTextTool {
    multimodal_service: MultimodalService,
}

#[async_trait]
impl Tool for SpeechToTextTool {
    const NAME: &'static str = "speech_to_text";
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = SpeechToTextArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> Result<ToolDefinition, Self::Error> {
        Ok(ToolDefinition {
            name: "speech_to_text".to_string(),
            description: "Transcribe audio to text".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "audio": {
                        "type": "string",
                        "description": "Base64 encoded audio data or file path"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language code (e.g., en, zh, es)",
                        "default": "en"
                    }
                },
                "required": ["audio"]
            }),
        })
    }

    async fn call(&self, args: serde_json::Value) -> Result<Self::Output, Self::Error> {
        let args: SpeechToTextArgs = serde_json::from_value(args)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let audio_data = if args.audio.starts_with('/') {
            fs::read(&args.audio).await?
        } else {
            use base64::{Engine as _, engine::general_purpose};
            general_purpose::STANDARD.decode(&args.audio)?
        };

        self.multimodal_service.transcribe_audio(&audio_data).await.map_err(|e| e.into())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SpeechToTextArgs {
    pub audio: String,
    pub language: Option<String>,
}

/// Document processing tool
#[derive(Debug)]
pub struct DocumentProcessorTool {
    multimodal_service: MultimodalService,
}

#[async_trait]
impl Tool for DocumentProcessorTool {
    const NAME: &'static str = "document_process";
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = DocumentProcessorArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> Result<ToolDefinition, Self::Error> {
        Ok(ToolDefinition {
            name: "document_process".to_string(),
            description: "Extract text and analyze documents".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "document": {
                        "type": "string",
                        "description": "Base64 encoded document data or file path"
                    },
                    "mime_type": {
                        "type": "string",
                        "description": "MIME type of the document"
                    },
                    "extract_images": {
                        "type": "boolean",
                        "description": "Extract images from documents",
                        "default": false
                    }
                },
                "required": ["document"]
            }),
        })
    }

    async fn call(&self, args: serde_json::Value) -> Result<Self::Output, Self::Error> {
        let args: DocumentProcessorArgs = serde_json::from_value(args)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let document_data = if args.document.starts_with('/') {
            fs::read(&args.document).await?
        } else {
            use base64::{Engine as _, engine::general_purpose};
            general_purpose::STANDARD.decode(&args.document)?
        };

        let mime_type = args.mime_type.as_deref().unwrap_or("text/plain");
        let text = self.multimodal_service.extract_text(&document_data, mime_type).await?;

        if args.extract_images.unwrap_or(false) {
            Ok(format!("Extracted text: {}\n\nImage extraction would be implemented with document processing libraries.", text))
        } else {
            Ok(text)
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct DocumentProcessorArgs {
    pub document: String,
    pub mime_type: Option<String>,
    pub extract_images: Option<bool>,
}

/// Enhanced ChatRequest with multimodal support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalChatRequest {
    pub messages: Vec<MultimodalMessage>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stream: bool,
    pub agent_config: Option<crate::AgentConfig>,
    pub tools: Option<Vec<ApiTool>>,
    pub multimodal_config: Option<MultimodalConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalConfig {
    pub enable_vision: bool,
    pub enable_audio: bool,
    pub enable_document_processing: bool,
    pub max_media_size: Option<usize>,
    pub supported_formats: Option<HashMap<String, Vec<String>>>,
}

impl Default for MultimodalConfig {
    fn default() -> Self {
        Self {
            enable_vision: true,
            enable_audio: true,
            enable_document_processing: true,
            max_media_size: Some(10 * 1024 * 1024), // 10MB
            supported_formats: None,
        }
    }
}

/// Convert MultimodalMessage to ChatMessage for compatibility
impl From<MultimodalMessage> for ChatMessage {
    fn from(msg: MultimodalMessage) -> Self {
        let content: Vec<MessageContent> = msg.content.into_iter().map(|c| match c {
            MultimodalContent::Text(text) => MessageContent::Text { text },
            MultimodalContent::Media(media) => {
                MessageContent::Image {
                    url: match media.content {
                        MediaData::Url(url) => url,
                        MediaData::Base64(data) => format!("data:{};base64,{}",
                            media.metadata.mime_type.as_deref().unwrap_or("application/octet-stream"), data),
                        MediaData::FilePath(path) => format!("file://{}", path.display()),
                        MediaData::Raw(_) => "data:application/octet-stream;base64,".to_string(),
                    },
                    description: media.metadata.filename,
                }
            },
            MultimodalContent::ToolCall(tc) => MessageContent::ToolRequest {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arguments: tc.arguments.clone(),
            },
            MultimodalContent::ToolResult(tr) => MessageContent::ToolResponse {
                id: tr.tool_call_id.clone(),
                name: "tool_result".to_string(),
                result: tr.result.clone(),
            },
        }).collect();

        let combined_content = content.into_iter().fold(String::new(), |mut acc, c| {
            match c {
                MessageContent::Text { text } => acc.push_str(&text),
                MessageContent::Image { url, .. } => acc.push_str(&format!("[Image: {}]", url)),
                MessageContent::ToolRequest { name, arguments, .. } => {
                    acc.push_str(&format!("[Tool Call: {} with args: {}]", name, arguments));
                }
                MessageContent::ToolResponse { result, .. } => {
                    acc.push_str(&format!("[Tool Result: {}]", result));
                }
                _ => {}
            }
            acc
        });

        ChatMessage {
            role: msg.role,
            content: combined_content,
            timestamp: msg.timestamp,
            tool_calls: None,
            tool_results: None,
        }
    }
}

/// Multimodal-enhanced agent service
pub struct MultimodalRigAgentService {
    base_service: crate::RigAgentService,
    multimodal_service: MultimodalService,
}

impl MultimodalRigAgentService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            base_service: crate::RigAgentService::new()?,
            multimodal_service: MultimodalService::new(),
        })
    }

    /// Process multimodal chat request
    pub async fn process_multimodal_message(&self, request: MultimodalChatRequest) -> Result<crate::ChatResponse> {
        let multimodal_config = request.multimodal_config.unwrap_or_default();

        // Add multimodal tools if enabled
        let mut tools = request.tools.unwrap_or_default();

        if multimodal_config.enable_vision && !tools.iter().any(|t| t.name == "vision_analyze") {
            tools.push(ApiTool {
                name: "vision_analyze".to_string(),
                description: "Analyze images and provide visual descriptions".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "image": {"type": "string"},
                        "prompt": {"type": "string"}
                    },
                    "required": ["image", "prompt"]
                }),
                is_mcp: false,
            });
        }

        if multimodal_config.enable_audio && !tools.iter().any(|t| t.name == "speech_to_text") {
            tools.push(ApiTool {
                name: "speech_to_text".to_string(),
                description: "Transcribe audio to text".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "audio": {"type": "string"},
                        "language": {"type": "string"}
                    },
                    "required": ["audio"]
                }),
                is_mcp: false,
            });
        }

        if multimodal_config.enable_document_processing && !tools.iter().any(|t| t.name == "document_process") {
            tools.push(ApiTool {
                name: "document_process".to_string(),
                description: "Extract text and analyze documents".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "document": {"type": "string"},
                        "mime_type": {"type": "string"}
                    },
                    "required": ["document"]
                }),
                is_mcp: false,
            });
        }

        // Convert multimodal messages to standard format
        let chat_messages: Vec<ChatMessage> = request.messages.into_iter()
            .map(|msg| msg.into())
            .collect();

        // Create standard chat request
        let chat_request = crate::ChatRequest {
            messages: chat_messages,
            model: request.model,
            system_prompt: request.system_prompt,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stream: request.stream,
            agent_config: request.agent_config,
            tools: Some(tools),
        };

        self.base_service.send_message(chat_request).await
    }

    /// Validate multimodal content
    pub async fn validate_multimodal_content(&self, content: &MediaContent) -> Result<()> {
        self.multimodal_service.validate_media(content).await
    }

    /// Convert media to base64
    pub async fn media_to_base64(&self, content: &MediaContent) -> Result<String> {
        self.multimodal_service.to_base64(content).await
    }

    /// Detect media type
    pub fn detect_media_type(&self, data: &[u8]) -> Option<(MediaType, String)> {
        self.multimodal_service.detect_media_type(data)
    }
}

impl Default for MultimodalRigAgentService {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

// Implement delegation for base service methods
impl std::ops::Deref for MultimodalRigAgentService {
    type Target = crate::RigAgentService;

    fn deref(&self) -> &Self::Target {
        &self.base_service
    }
}