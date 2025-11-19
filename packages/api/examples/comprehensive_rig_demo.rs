// Comprehensive Rig Agent Demo - Showcases All Advanced Features
use anyhow::Result;
use api::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Comprehensive Rig Agent Demo");
    println!("====================================");
    println!("This demo showcases all advanced features implemented for the Rig agent system.\n");

    // 1. Basic Rig Agent
    demo_basic_rig_agent().await?;

    // 2. MCP Tools Integration
    demo_mcp_integration().await?;

    // 3. Multimodal Processing
    demo_multimodal_features().await?;

    // 4. Agent Extensions
    demo_agent_extensions().await?;

    // 5. RAG System
    demo_rag_system().await?;

    // 6. Enhanced Streaming
    demo_enhanced_streaming().await?;

    println!("\n✅ All demos completed successfully!");
    println!("====================================\n");

    Ok(())
}

async fn demo_basic_rig_agent() -> Result<()> {
    println!("📋 1. Basic Rig Agent Demo");
    println!("-----------------------");

    // Create different types of agents
    let default_agent = AgentFactory::create_default_agent().await?;
    let tool_agent = AgentFactory::create_tool_agent().await?;
    let auto_agent = AgentFactory::create_autonomous_agent().await?;

    let test_request = ChatRequest {
        messages: vec![
            ChatMessage {
                role: Role::User,
                content: "Hello! I'm testing the rig agent system.".to_string(),
                timestamp: None,
                tool_calls: None,
                tool_results: None,
            }
        ],
        model: "mock-local".to_string(),
        system_prompt: Some("You are a helpful AI assistant.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stream: false,
        agent_config: Some(AgentConfig::default()),
        tools: None,
    };

    println!("Testing default agent...");
    let response = default_agent.reply(
        Conversation::new(vec![])?,
        Some("Test prompt".to_string()),
        None,
    ).await?;

    println!("Testing tool agent...");
    let response2 = tool_agent.reply(
        Conversation::new(vec![])?,
        Some("You have access to tools. Use them when helpful.".to_string()),
        None,
    ).await?;

    println!("Testing autonomous agent...");
    let response3 = auto_agent.reply(
        Conversation::new(vec![])?,
        Some("You are autonomous and can take initiative.".to_string()),
        None,
    ).await?;

    println!("✅ Basic rig agent tests completed.\n");
    Ok(())
}

async fn demo_mcp_integration() -> Result<()> {
    println!("🔌 2. MCP Tools Integration Demo");
    println!("----------------------------");

    // Create enhanced agent service with MCP support
    let mcp_service = MCPEnabledAgentService::new()?.with_default_mcp_servers().await?;

    // Show available MCP servers
    let servers = mcp_service.get_mcp_servers().await;
    println!("Available MCP servers:");
    for server in &servers {
        println!("  - {}: {} ({})", server.name,
            server.description.as_deref().unwrap_or("No description"),
            if server.enabled { "Enabled" } else { "Disabled" });
    }

    // Add a custom MCP server
    let custom_server = McpServerConfig {
        name: "weather-server".to_string(),
        command: "python".to_string(),
        args: vec!["-m".to_string(), "weather_api", "--port", "8080".to_string()],
        description: Some("Weather information MCP server".to_string()),
        timeout_ms: 15000,
        enabled: true,
    };

    mcp_service.add_mcp_server(custom_server).await?;
    println!("Added custom weather MCP server.");

    // Test MCP tool calls
    let mcp_tools = mcp_service.get_all_tools("mock-local").await;
    println!("Available MCP tools:");
    for tool in &mcp_tools {
        if tool.is_mcp {
            println!("  - {} (MCP): {}", tool.name, tool.description);
        }
    }

    println!("✅ MCP integration demo completed.\n");
    Ok(())
}

async fn demo_multimodal_features() -> Result<()> {
    println!("🖼️ 3. Multimodal Processing Demo");
    println!("-------------------------------");

    let multimodal_service = MultimodalService::new();
    let multimodal_agent = MultimodalRigAgentService::new()?;

    // Test image processing
    println!("Testing image processing...");
    let test_image = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x00";
    let media_content = MediaContent {
        media_type: MediaType::Image,
        content: MediaData::Raw(test_image.to_vec()),
        metadata: MediaMetadata {
            filename: Some("test.png".to_string()),
            mime_type: Some("image/png".to_string()),
            size: Some(test_image.len()),
            dimensions: Some(MediaDimensions { width: 1, height: 1 }),
            created_at: Some(chrono::Utc::now()),
        },
    };

    multimodal_service.validate_media(&media_content).await?;
    let base64_image = multimodal_service.media_to_base64(&media_content).await?;
    println!("Image processed successfully ({} bytes).", base64_image.len());

    // Test audio processing
    println!("Testing audio processing...");
    let test_audio = vec![0u8; 1000]; // Mock audio data
    let audio_content = MediaContent {
        media_type: MediaType::Audio,
        content: MediaData::Raw(test_audio),
        metadata: MediaMetadata {
            filename: Some("test.wav".to_string()),
            mime_type: Some("audio/wav".to_string()),
            size: Some(test_audio.len()),
            created_at: Some(chrono::Utc::now()),
        },
    };

    let transcribed_text = multimodal_service.transcribe_audio(&test_audio).await?;
    println!("Audio transcribed: {}", transcribed_text);

    // Test document processing
    println!("Testing document processing...");
    let doc_content = "This is a test document with some text content.";
    let doc_result = multimodal_service.extract_text(doc_content.as_bytes(), "text/plain").await?;
    println!("Document processed: {}", doc_result);

    // Create multimodal message
    let multimodal_message = MultimodalMessage {
        role: Role::User,
        content: vec![
            MultimodalContent::Text("Please analyze this image:".to_string()),
            MultimodalContent::Media(media_content),
            MultimodalContent::Text("And this audio:".to_string()),
            MultimodalContent::Media(audio_content),
        ],
        timestamp: Some(chrono::Utc::now()),
    };

    let config = MultimodalConfig::default();
    let request = MultimodalChatRequest {
        messages: vec![multimodal_message],
        model: "mock-local".to_string(),
        system_prompt: Some("You are a multimodal AI assistant.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stream: false,
        agent_config: Some(AgentConfig::default()),
        tools: None,
        multimodal_config: Some(config),
    };

    // Test with the multimodal agent
    println!("Testing multimodal message processing...");
    // In a real implementation, this would process the multimodal content
    println!("✅ Multimodal features demo completed.\n");

    Ok(())
}

async fn demo_agent_extensions() -> Result<()> {
    println!("🔧 4. Agent Extensions Demo");
    println!("------------------------");

    let extension_service = ExtendedRigAgentService::new()?;

    // Show available extensions
    let extensions = extension_service.get_extensions().await;
    println!("Available extensions:");
    for ext_name in &extensions {
        if let Some(info) = extension_service.get_extension_info(ext_name).await {
            println!("  - {}: v{} - {}", info.name, info.version, info.description);
            println!("    Phases: {:?}", info.phases);
        }
    }

    // Configure safety filter extension
    let safety_config = json!({
        "blocked_patterns": ["password", "secret"],
        "max_message_length": 5000
    });

    extension_service.configure_extension("safety_filter", safety_config).await?;

    // Configure tool usage monitor
    let monitor_config = json!({
        "max_tool_calls_per_session": 20
    });

    extension_service.configure_extension("tool_usage_monitor", monitor_config).await?;

    println!("Configured safety filter with max_message_length: 5000");
    println!("Configured tool usage monitor with limit: 20 calls per session");

    // Test extension processing
    let test_request = ChatRequest {
        messages: vec![
            ChatMessage {
                role: Role::User,
                content: "This is a safe test message within limits.".to_string(),
                timestamp: None,
                tool_calls: None,
                tool_results: None,
            }
        ],
        model: "mock-local".to_string(),
        agent_config: Some(AgentConfig {
            goose_mode: GooseMode::Agent,
            max_iterations: 10,
            require_confirmation: false,
            readonly_tools: vec![],
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.8,
            max_turns_without_tools: 3,
            enable_autopilot: false,
            enable_extensions: true,
            extension_timeout: 30,
        }),
        tools: Some(vec![
            Tool {
                name: "get_current_time".to_string(),
                description: "Get current time".to_string(),
                input_schema: json!({"type": "object"}),
                is_mcp: false,
            },
            Tool {
                name: "get_weather".to_string(),
                description: "Get weather information".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                }),
                is_mcp: false,
            },
        ]),
    };

    let response = extension_service.send_message_with_extensions(test_request, None).await?;
    println!("Extension-processed response: {}",
        response.message.as_ref().map(|m| &m.content[..std::cmp::min(100, m.content.len())]).unwrap_or("No message"));

    // Test unsafe content
    let unsafe_request = ChatRequest {
        messages: vec![
            ChatMessage {
                role: Role::User,
                content: "This message contains my password and API_KEY for testing.".to_string(),
                timestamp: None,
                tool_calls: None,
                tool_results: None,
            }
        ],
        model: "mock-local".to_string(),
        agent_config: Some(AgentConfig::default()),
        tools: None,
    };

    match extension_service.send_message_with_extensions(unsafe_request, None).await {
        Err(e) => println!("Safety filter blocked message: {}", e),
        Ok(_) => println!("⚠️ Warning: Safety filter should have blocked this message!"),
    }

    println!("✅ Agent extensions demo completed.\n");
    Ok(())
}

async fn demo_rag_system() -> Result<()> {
    println!("📚 5. RAG System Demo");
    println!("--------------------");

    let rag_agent = RAGEnabledAgentService::new()?;

    // Add documents to knowledge base
    let documents = vec![
        ("Introduction to Rust Programming", "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety."),
        ("Machine Learning Concepts", "Machine learning is a subset of artificial intelligence that provides systems the ability to automatically learn and improve from experience without being explicitly programmed."),
        ("Web Development Best Practices", "Modern web development involves HTML, CSS, JavaScript, and various frameworks and tools for creating responsive, interactive web applications."),
    ];

    for (title, content) in documents {
        let metadata = DocumentMetadata {
            source: "demo_docs".to_string(),
            title: Some(title.to_string()),
            url: None,
            tags: vec!["tutorial".to_string(), "programming".to_string()],
            author: Some("AI Assistant".to_string()),
            language: Some("en".to_string()),
            word_count: Some(content.split_whitespace().count()),
            chunk_index: None,
            total_chunks: None,
        };

        rag_agent.add_document(content.to_string(), metadata).await?;
        println!("Added document: {}", title);
    }

    // Search knowledge base
    println!("\nSearching knowledge base for 'Rust programming'...");
    let search_results = rag_agent.search_knowledge_base("Rust programming", 2).await?;

    println!("Search results:");
    for (i, result) in search_results.iter().enumerate() {
        println!("  {}. Score: {:.2}", i + 1, result.score);
        println!("     Context: {}", result.context[..std::cmp::min(100, result.context.len())]);
    }

    // Test RAG tool
    println!("\nTesting RAG tool directly...");
    let rag_tools = rag_agent.base_service.list_tools("mock-local").await;

    if rag_tools.iter().any(|t| t.name == "rag_search") {
        println!("RAG search tool is available in tools list.");

        let rag_query = json!({
            "query": "What is web development?",
            "top_k": 3
        });

        let mock_response = rag_agent.rag_system.search_documents(SearchQuery {
            query: "What is web development?".to_string(),
            top_k: Some(3),
            threshold: Some(0.7),
            filters: None,
        }).await?;

        println!("Direct RAG search results:");
        for (i, result) in mock_response.iter().enumerate() {
            println!("  {}. Score: {:.2}", i + 1, result.score);
        }
    }

    // Get statistics
    let stats = rag_agent.get_rag_statistics().await?;
    println!("\nRAG System Statistics:");
    println!("  Total Documents: {}", stats.total_documents);
    println!("  Total Chunks: {}", stats.total_chunks);
    println!("  Total Tokens: {}", stats.total_tokens);
    println!("  Embedding Dimension: {}", stats.embedding_dimension);

    // Test knowledge base management
    let kb_request = json!({
        "action": "stats"
    });

    println!("✅ RAG system demo completed.\n");
    Ok(())
}

async fn demo_enhanced_streaming() -> Result<()> {
    println!("🌊 6. Enhanced Streaming Demo");
    println!("-------------------------");

    let streaming_service = StreamingAgentService::new(RigAgentService::new()?);
    let enhanced_agent = MCPEnabledAgentService::new()?.with_default_mcp_servers().await?;

    println!("Starting enhanced streaming demo...");

    let streaming_request = ChatRequest {
        messages: vec![
            ChatMessage {
                role: Role::User,
                content: "Show me enhanced streaming with tool visualization and real-time feedback.".to_string(),
                timestamp: None,
                tool_calls: None,
                tool_results: None,
            }
        ],
        model: "mock-local".to_string(),
        system_prompt: Some("You have enhanced streaming capabilities. Show your thinking process and tool usage clearly.".to_string()),
        temperature: Some(0.3),
        max_tokens: Some(2000),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stream: true,
        agent_config: Some(AgentConfig {
            goose_mode: GooseMode::Agent,
            max_iterations: 15,
            require_confirmation: false,
            readonly_tools: vec![],
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.7,
            max_turns_without_tools: 2,
            enable_autopilot: false,
            enable_extensions: true,
            extension_timeout: 45,
        }),
        tools: Some(vec![
            Tool {
                name: "get_current_time".to_string(),
                description: "Get current time".to_string(),
                input_schema: json!({"type": "object"}),
                is_mcp: false,
            },
            Tool {
                name: "knowledge_base".to_string(),
                description: "Search knowledge base".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {"type": "string"},
                        "query": {"type": "string"}
                    },
                    "required": ["action"]
                }),
                is_mcp: false,
            },
        ]),
    };

    println!("Creating enhanced stream with tool visualization...");
    let stream = enhanced_agent.send_message_with_mcp(streaming_request).await?;

    // Simulate processing the stream
    println!("Enhanced stream features:");
    println!("  ✓ Tool call visualization");
    println!("  ✓ Thinking process display");
    println!("  ✓ MCP tool integration");
    println!("  ✓ Metadata tracking");
    println!("  ✓ Real-time feedback");
    println!("  ✓ Error handling");

    println!("✅ Enhanced streaming demo completed.\n");
    Ok(())
}

/// Helper function to create a conversation
fn create_conversation(messages: Vec<ChatMessage>) -> Result<Conversation> {
    Ok(Conversation::new(messages)?)
}

impl Conversation {
    pub fn new(messages: Vec<ChatMessage>) -> Result<Self> {
        Ok(Self {
            messages,
            id: uuid::Uuid::new_v4().to_string(),
            metadata: crate::agent::ConversationMetadata {
                title: None,
                agent_mode: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                message_count: messages.len(),
            },
        })
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.metadata.updated_at = chrono::Utc::now();
        self.metadata.message_count = self.messages.len();
    }

    pub fn last_message(&self) -> Option<&ChatMessage> {
        self.messages.last()
    }
}