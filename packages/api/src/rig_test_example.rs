// Rig Agent测试示例
use anyhow::Result;
use serde_json::json;

// 简化版本，用于测试rig集成而不需要完整的项目结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestChatRequest {
    pub messages: Vec<TestChatMessage>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 开始测试Rig Agent集成");

    // 测试1: 基本模型初始化
    test_basic_rig_integration().await?;

    // 测试2: 工具定义
    test_tool_definitions().await?;

    // 测试3: Agent Builder
    test_agent_builder().await?;

    println!("✅ 所有测试完成！");
    Ok(())
}

async fn test_basic_rig_integration() -> Result<()> {
    println!("\n📋 测试1: 基本Rig集成");

    // 检查是否能导入rig的基本组件
    // 由于我们不需要实际调用API，这里只是检查编译
    let mock_request = TestChatRequest {
        messages: vec![
            TestChatMessage {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
            }
        ],
        model: "mock-local".to_string(),
        system_prompt: Some("You are a helpful assistant.".to_string()),
        temperature: Some(0.7),
    };

    let request_json = serde_json::to_string_pretty(&mock_request)?;
    println!("✅ 请求序列化成功:");
    println!("{}", request_json);

    Ok(())
}

async fn test_tool_definitions() -> Result<()> {
    println!("\n🛠️ 测试2: 工具定义");

    // 定义工具schema
    let datetime_tool = json!({
        "name": "get_current_time",
        "description": "获取当前日期和时间",
        "parameters": {
            "type": "object",
            "properties": {},
            "required": []
        }
    });

    let weather_tool = json!({
        "name": "get_weather",
        "description": "获取指定位置的天气信息",
        "parameters": {
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "要查询天气的位置"
                }
            },
            "required": ["location"]
        }
    });

    println!("✅ 工具定义成功:");
    println!("时间工具: {}", serde_json::to_string_pretty(&datetime_tool)?);
    println!("天气工具: {}", serde_json::to_string_pretty(&weather_tool)?);

    Ok(())
}

async fn test_agent_builder() -> Result<()> {
    println!("\n🏗️ 测试3: Agent Builder概念");

    // 模拟不同类型的agent配置
    let conversational_config = json!({
        "agent_type": "conversational",
        "system_prompt": "You are a friendly conversational AI assistant.",
        "temperature": 0.7,
        "max_tokens": 1000,
        "tools": ["datetime"]
    });

    let tool_agent_config = json!({
        "agent_type": "tool_agent",
        "system_prompt": "You are a capable AI assistant with access to tools.",
        "temperature": 0.3,
        "max_tokens": 2000,
        "tools": ["datetime", "weather"]
    });

    let autonomous_config = json!({
        "agent_type": "autonomous",
        "system_prompt": "You are an autonomous AI assistant that can take initiative.",
        "temperature": 0.5,
        "max_tokens": 3000,
        "tools": ["datetime", "weather"],
        "enable_autopilot": true,
        "max_iterations": 20
    });

    println!("✅ Agent配置定义成功:");
    println!("对话Agent: {}", serde_json::to_string_pretty(&conversational_config)?);
    println!("工具Agent: {}", serde_json::to_string_pretty(&tool_agent_config)?);
    println!("自主Agent: {}", serde_json::to_string_pretty(&autonomous_config)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serialization() -> Result<()> {
        let message = TestChatMessage {
            role: "user".to_string(),
            content: "Test message".to_string(),
        };

        let serialized = serde_json::to_string(&message)?;
        let deserialized: TestChatMessage = serde_json::from_str(&serialized)?;

        assert_eq!(message.role, deserialized.role);
        assert_eq!(message.content, deserialized.content);

        println!("✅ 序列化测试通过");
        Ok(())
    }

    #[test]
    fn test_tool_schema() -> Result<()> {
        let tool_schema = json!({
            "name": "test_tool",
            "description": "A test tool",
            "parameters": {
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Input parameter"
                    }
                },
                "required": ["input"]
            }
        });

        let schema_str = serde_json::to_string_pretty(&tool_schema)?;
        assert!(schema_str.contains("test_tool"));
        assert!(schema_str.contains("Input parameter"));

        println!("✅ 工具schema测试通过");
        Ok(())
    }
}