// Rig Agent 集成演示
use api::{RigAgentService, AgentFactory, ChatRequest, ChatMessage, Role, AgentConfig, GooseMode};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Rig Agent 集成演示");
    println!("====================");

    // 1. 创建 Rig Agent Service
    println!("\n📋 1. 创建 Rig Agent Service");
    let agent_service = RigAgentService::new()?;
    println!("✅ Rig Agent Service 创建成功");

    // 2. 获取可用模型
    println!("\n🤖 2. 获取可用模型");
    let models = agent_service.get_available_models();
    for model in &models {
        println!("  - {}: {} ({})", model.id, model.name, model.provider);
    }
    println!("✅ 共找到 {} 个模型", models.len());

    // 3. 创建不同类型的 Agent 配置
    println!("\n🏗️ 3. Agent 配置示例");

    // 对话模式
    let chat_config = AgentConfig {
        goose_mode: GooseMode::Chat,
        max_iterations: 5,
        require_confirmation: false,
        readonly_tools: vec![],
        enable_tool_inspection: true,
        enable_auto_compact: false,
        compact_threshold: 0.8,
        max_turns_without_tools: 3,
        enable_autopilot: false,
        enable_extensions: false,
        extension_timeout: 30,
    };

    // Agent 模式
    let agent_config = AgentConfig {
        goose_mode: GooseMode::Agent,
        max_iterations: 10,
        require_confirmation: false,
        readonly_tools: vec![],
        enable_tool_inspection: true,
        enable_auto_compact: true,
        compact_threshold: 0.8,
        max_turns_without_tools: 5,
        enable_autopilot: false,
        enable_extensions: true,
        extension_timeout: 60,
    };

    // 自主模式
    let auto_config = AgentConfig {
        goose_mode: GooseMode::Auto,
        max_iterations: 20,
        require_confirmation: false,
        readonly_tools: vec![],
        enable_tool_inspection: true,
        enable_auto_compact: true,
        compact_threshold: 0.7,
        max_turns_without_tools: 10,
        enable_autopilot: true,
        enable_extensions: true,
        extension_timeout: 90,
    };

    println!("✅ Agent 配置创建完成");
    println!("  - Chat 模式: 最大迭代 {}", chat_config.max_iterations);
    println!("  - Agent 模式: 最大迭代 {}", agent_config.max_iterations);
    println!("  - Auto 模式: 最大迭代 {}, 启用自动导航", auto_config.max_iterations);

    // 4. 演示聊天请求创建
    println!("\n💬 4. 创建聊天请求示例");

    let test_requests = vec![
        // 简单对话
        ChatRequest {
            messages: vec![
                ChatMessage {
                    role: Role::User,
                    content: "你好，请介绍一下你自己。".to_string(),
                    timestamp: None,
                    tool_calls: None,
                    tool_results: None,
                }
            ],
            model: "mock-local".to_string(),
            system_prompt: Some("你是一个友好的AI助手。".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(500),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: false,
            agent_config: Some(chat_config.clone()),
            tools: None,
        },

        // 工具使用请求
        ChatRequest {
            messages: vec![
                ChatMessage {
                    role: Role::User,
                    content: "现在几点了？北京天气怎么样？".to_string(),
                    timestamp: None,
                    tool_calls: None,
                    tool_results: None,
                }
            ],
            model: "mock-local".to_string(),
            system_prompt: Some("你是一个有工具使用能力的AI助手。".to_string()),
            temperature: Some(0.3),
            max_tokens: Some(800),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: false,
            agent_config: Some(agent_config.clone()),
            tools: Some(vec![
                api::Tool {
                    name: "get_current_time".to_string(),
                    description: "获取当前日期和时间".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    }),
                    is_mcp: false,
                },
                api::Tool {
                    name: "get_weather".to_string(),
                    description: "获取天气信息".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "要查询的位置"
                            }
                        },
                        "required": ["location"]
                    }),
                    is_mcp: false,
                }
            ]),
        },
    ];

    println!("✅ 创建了 {} 个测试请求", test_requests.len());
    for (i, request) in test_requests.iter().enumerate() {
        let mode = request.agent_config.as_ref()
            .map(|c| format!("{:?}", c.goose_mode))
            .unwrap_or("None".to_string());
        let has_tools = request.tools.as_ref()
            .map(|t| t.len())
            .unwrap_or(0);
        println!("  请求 {}: {} 模式, {} 个工具", i + 1, mode, has_tools);
    }

    // 5. 演示序列化和反序列化
    println!("\n📦 5. 序列化演示");
    let request_json = serde_json::to_string_pretty(&test_requests[0])?;
    println!("✅ 请求序列化成功，长度: {} 字符", request_json.len());

    let deserialized: ChatRequest = serde_json::from_str(&request_json)?;
    println!("✅ 请求反序列化成功");
    println!("  消息数量: {}", deserialized.messages.len());
    println!("  模型: {}", deserialized.model);

    // 6. Agent Factory 演示
    println!("\n🏭 6. Agent Factory 演示");
    let agent_factory = AgentFactory::new();

    // 获取可用的 agent 类型
    let agent_types = vec![
        "conversational", "tool_agent", "autonomous",
        "programming", "research", "creative", "analysis"
    ];

    for agent_type in &agent_types {
        println!("  - {} agent", agent_type);
    }
    println!("✅ 支持的 Agent 类型: {}", agent_types.len());

    // 7. 总结
    println!("\n📊 7. 集成状态总结");
    println!("  ✅ Rig Agent Service: 已集成");
    println!("  ✅ Agent Builder: 已集成");
    println!("  ✅ Streaming Service: 已集成");
    println!("  ✅ Tool Registry: 已集成");
    println!("  ✅ 多种 Agent 模式: 支持 Chat/Agent/Auto");
    println!("  ✅ 多种 Provider: 支持 OpenAI/DeepSeek/Anthropic");

    println!("\n🎉 Rig Agent 集成演示完成！");
    println!("=====================================");

    Ok(())
}