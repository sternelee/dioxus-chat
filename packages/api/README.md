# Dioxus Chat API - Rig Agent 集成

这是基于 `rig` 框架重构后的聊天 API，提供了强大的智能体功能和工具集成能力。

## 🚀 功能特性

### 核心服务
- **RigAgentService**: 基于 rig 的智能体服务
- **StreamingAgentService**: 增强的流式响应服务
- **AgentFactory**: 智能体工厂，支持多种预配置类型
- **ToolRegistry**: 工具注册表，支持动态工具管理

### 智能体模式
- **Chat**: 对话模式，专注于自然对话
- **Agent**: 代理模式，具有工具使用能力
- **Auto**: 自主模式，主动帮助用户，可使用工具

### 支持的 AI 提供商
- OpenAI (GPT-4o, GPT-3.5-turbo)
- DeepSeek (deepseek-chat, deepseek-r1)
- Anthropic (Claude 3.5 Sonnet)
- Mock (用于测试)

### 内置工具
- **DateTimeTool**: 获取当前时间
- **WeatherTool**: 获取天气信息
- **CalculatorTool**: 数学计算
- **FileOperationTool**: 文件操作

## 🛠️ 快速开始

### 1. 基本使用

```rust
use api::{RigAgentService, ChatRequest, ChatMessage, Role};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建智能体服务
    let agent_service = RigAgentService::new()?;

    // 创建聊天请求
    let request = ChatRequest {
        messages: vec![
            ChatMessage {
                role: Role::User,
                content: "你好！".to_string(),
                timestamp: None,
                tool_calls: None,
                tool_results: None,
            }
        ],
        model: "openai/gpt-4o".to_string(),
        system_prompt: Some("你是一个友好的AI助手。".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stream: false,
        agent_config: None,
        tools: None,
    };

    // 发送消息
    let response = agent_service.send_message(request).await?;
    println!("回复: {}", response.message.unwrap().content);

    Ok(())
}
```

### 2. 使用智能体配置

```rust
use api::{AgentConfig, GooseMode};

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

let request = ChatRequest {
    // ... 其他字段
    agent_config: Some(agent_config),
    tools: Some(vec![
        api::Tool {
            name: "get_current_time".to_string(),
            description: "获取当前时间".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            is_mcp: false,
        }
    ]),
    // ...
};
```

### 3. 流式响应

```rust
use api::StreamingAgentService;
use futures::StreamExt;

let agent_service = RigAgentService::new()?;
let streaming_service = StreamingAgentService::new(agent_service);

let request = ChatRequest { /* ... */ };
let mut stream = streaming_service.stream_chat_response(request).await?;

while let Some(chunk) = stream.next().await {
    match chunk.chunk_type {
        api::ChunkType::Content => {
            print!("{}", chunk.base.content.unwrap_or_default());
        },
        api::ChunkType::ToolCall => {
            println!("[工具调用] {}", chunk.base.content.unwrap_or_default());
        },
        api::ChunkType::Thinking => {
            println!("[思考] {}", chunk.base.content.unwrap_or_default());
        },
        _ => {}
    }
}
```

### 4. 使用 Agent Factory

```rust
use api::AgentFactory;

let factory = AgentFactory::new();

// 创建对话型智能体
let conversational_agent = factory
    .create_conversational_agent("openai/gpt-4o")
    .with_system_prompt("你是一个友好的助手。".to_string())
    .with_temperature(0.8);

// 创建工具型智能体
let tool_agent = factory
    .create_tool_agent("openai/gpt-4o")
    .with_tools(vec![datetime_tool, weather_tool]);

// 创建自主型智能体
let auto_agent = factory
    .create_autonomous_agent("openai/gpt-4o")
    .with_max_iterations(20);
```

## 🌐 API 端点

### 基础端点
- `POST /api/models` - 获取可用模型列表
- `POST /api/chat` - 发送聊天消息
- `POST /api/chat/stream` - 流式聊天响应
- `POST /api/tools` - 获取可用工具

### 增强端点
- `POST /api/agents/create` - 创建专用智能体
- `POST /api/agents/types` - 获取智能体类型
- `POST /api/chat/stream/enhanced` - 增强流式聊天（带工具可视化）

## 📚 示例

项目包含多个示例：

1. **基础演示** (`examples/rig_agent_demo.rs`): 展示基本的智能体使用
2. **工具集成** (`examples/tool_integration_demo.rs`): 演示工具集成和自定义工具
3. **测试示例** (`src/rig_test_example.rs`): 基本功能测试

运行示例：
```bash
# 运行基础演示
cargo run --example rig_agent_demo

# 运行工具集成演示
cargo run --example tool_integration_demo
```

## 🔑 环境变量

需要设置以下环境变量来使用相应的 AI 提供商：

```bash
# OpenAI
export OPENAI_API_KEY="your-openai-api-key"

# DeepSeek
export DEEPSEEK_API_KEY="your-deepseek-api-key"

# Anthropic
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

## 🏗️ 架构

### 核心组件

1. **RigAgentService**
   - 智能体管理和缓存
   - 多提供商支持
   - 工具集成

2. **StreamingAgentService**
   - 增强流式响应
   - 元数据追踪
   - 工具调用可视化

3. **AgentBuilder**
   - 灵活的智能体配置
   - 预设模板
   - 自定义扩展

4. **ToolRegistry**
   - 动态工具注册
   - 工具发现
   - 类型安全

## 🔄 迁移指南

### 从 SimpleChatService 迁移

旧的：
```rust
let service = SimpleChatService::new()?;
let response = service.send_message(request).await?;
```

新的：
```rust
let service = RigAgentService::new()?;
let response = service.send_message(request).await?;
```

主要变化：
- 更强的智能体功能
- 内置工具支持
- 增强的流式响应
- 多提供商支持

## 📈 性能优化

- **智能体缓存**: 相同配置的智能体会被缓存重用
- **流式响应**: 支持实时响应，减少等待时间
- **工具并行**: 支持并行工具调用
- **内存管理**: 自动清理未使用的智能体

---

## 服务器架构说明

这个 crate 包含所有共享的全栈服务器函数。这是一个放置您想要在多个平台（如数据库访问或邮件发送）暴露的服务器逻辑的好地方。

这个 crate 将构建两次：
1. 一次为服务器构建，启用 `dioxus/server` 功能
2. 一次为客户端构建，禁用客户端功能

在服务器构建期间，服务器函数将被收集并托管在公共 API 上供客户端调用。在客户端构建期间，服务器函数将被编译到客户端构建中。

大多数服务器依赖（如 sqlx 和 tokio）将无法在像 WASM 这样的客户端平台上编译。为了避免在客户端上构建服务器依赖，您应该在 [Cargo.toml](./Cargo.toml) 文件中的 `server` 功能下添加平台特定的依赖。
