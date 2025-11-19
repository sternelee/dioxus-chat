// 工具集成演示
use api::{RigAgentService, Tool, CustomTool, ToolRegistry};
use async_trait::async_trait;
use rig::{completion::ToolDefinition, tool::Tool as RigTool};
use serde_json::json;
use std::collections::HashMap;

// 自定义工具示例
#[derive(Debug)]
pub struct CalculatorTool;

#[async_trait::async_trait]
impl RigTool for CalculatorTool {
    const NAME: &'static str = "calculator";
    type Error = anyhow::Error;
    type Args = CalculatorArgs;
    type Output = f64;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "calculator".to_string(),
            description: "执行基本的数学计算".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "要计算的数学表达式，例如 '2 + 3 * 4'"
                    }
                },
                "required": ["expression"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 简单的表达式计算（实际项目中应该使用更安全的解析器）
        match args.operation.as_str() {
            "+" => Ok(args.a + args.b),
            "-" => Ok(args.a - args.b),
            "*" => Ok(args.a * args.b),
            "/" => {
                if args.b != 0.0 {
                    Ok(args.a / args.b)
                } else {
                    Err(anyhow::anyhow!("除零错误"))
                }
            },
            _ => Err(anyhow::anyhow!("不支持的操作: {}", args.operation))
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CalculatorArgs {
    pub a: f64,
    pub b: f64,
    pub operation: String,
}

impl api::CustomTool for CalculatorTool {
    fn name(&self) -> &'static str {
        "calculator"
    }

    fn description(&self) -> &'static str {
        "执行基本的数学计算（加、减、乘、除）"
    }
}

// 文件操作工具示例
#[derive(Debug)]
pub struct FileOperationTool;

#[async_trait::async_trait]
impl RigTool for FileOperationTool {
    const NAME: &'static str = "file_operations";
    type Error = anyhow::Error;
    type Args = FileOperationArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "file_operations".to_string(),
            description: "执行基本文件操作（读取、写入、列出文件）".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "list"],
                        "description": "文件操作类型"
                    },
                    "path": {
                        "type": "string",
                        "description": "文件路径"
                    },
                    "content": {
                        "type": "string",
                        "description": "要写入的内容（仅用于写操作）"
                    }
                },
                "required": ["operation", "path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.operation.as_str() {
            "read" => {
                // 模拟文件读取
                Ok(format!("读取文件 {} 的内容: 这是一个模拟的文件内容", args.path))
            },
            "write" => {
                // 模拟文件写入
                Ok(format!("成功将内容写入文件 {}", args.path))
            },
            "list" => {
                // 模拟文件列表
                Ok(format!("列出目录 {} 的文件: file1.txt, file2.txt, subdirectory/", args.path))
            },
            _ => Err(anyhow::anyhow!("不支持的文件操作: {}", args.operation))
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FileOperationArgs {
    pub operation: String,
    pub path: String,
    pub content: Option<String>,
}

impl api::CustomTool for FileOperationTool {
    fn name(&self) -> &'static str {
        "file_operations"
    }

    fn description(&self) -> &'static str {
        "执行基本文件操作（读取、写入、列出文件）"
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔧 工具集成演示");
    println!("================");

    // 1. 创建工具注册表
    println!("\n📋 1. 创建工具注册表");
    let mut tool_registry = ToolRegistry::new();

    // 注册自定义工具
    tool_registry.register_tool("calculator".to_string(), CalculatorTool);
    tool_registry.register_tool("file_operations".to_string(), FileOperationTool);

    println!("✅ 工具注册表创建成功");
    println!("注册的工具:");
    for tool_name in tool_registry.list_tools() {
        println!("  - {}", tool_name);
    }

    // 2. 演示内置工具
    println!("\n🛠️ 2. 内置工具演示");

    let builtin_tools = vec![
        Tool {
            name: "get_current_time".to_string(),
            description: "获取当前日期和时间".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
            is_mcp: false,
        },
        Tool {
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
    ];

    for tool in &builtin_tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // 3. 演示工具调用示例
    println!("\n📞 3. 工具调用示例");

    let tool_call_examples = vec![
        json!({
            "tool": "get_current_time",
            "args": {}
        }),
        json!({
            "tool": "get_weather",
            "args": {
                "location": "北京"
            }
        }),
        json!({
            "tool": "calculator",
            "args": {
                "a": 10.0,
                "b": 5.0,
                "operation": "+"
            }
        }),
        json!({
            "tool": "file_operations",
            "args": {
                "operation": "read",
                "path": "/path/to/file.txt"
            }
        })
    ];

    for (i, example) in tool_call_examples.iter().enumerate() {
        let tool_name = example.get("tool").unwrap().as_str().unwrap();
        let args = example.get("args").unwrap();
        println!("  {}. 调用工具: {}", i + 1, tool_name);
        println!("     参数: {}", serde_json::to_string_pretty(args)?);

        // 模拟工具调用结果
        match tool_name {
            "get_current_time" => {
                println!("     结果: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
            },
            "get_weather" => {
                let location = args.get("location").unwrap().as_str().unwrap();
                println!("     结果: {} 天气晴朗，温度 25°C", location);
            },
            "calculator" => {
                let a = args.get("a").unwrap().as_f64().unwrap();
                let b = args.get("b").unwrap().as_f64().unwrap();
                let op = args.get("operation").unwrap().as_str().unwrap();
                match op {
                    "+" => println!("     结果: {} + {} = {}", a, b, a + b),
                    "-" => println!("     结果: {} - {} = {}", a, b, a - b),
                    "*" => println!("     结果: {} × {} = {}", a, b, a * b),
                    "/" => println!("     结果: {} ÷ {} = {}", a, b, a / b),
                    _ => println!("     结果: 不支持的操作"),
                }
            },
            "file_operations" => {
                let operation = args.get("operation").unwrap().as_str().unwrap();
                let path = args.get("path").unwrap().as_str().unwrap();
                match operation {
                    "read" => println!("     结果: 模拟读取文件 {}", path),
                    "write" => println!("     结果: 模拟写入文件 {}", path),
                    "list" => println!("     结果: 模拟列出目录 {}", path),
                    _ => println!("     结果: 不支持的操作"),
                }
            },
            _ => println!("     结果: 未知工具"),
        }
    }

    // 4. 集成到 Agent Service
    println!("\n🤖 4. 集成到 Agent Service");
    let agent_service = RigAgentService::new()?;

    // 测试工具列表
    let models = agent_service.get_available_models();
    if let Some(model) = models.first() {
        println!("测试模型: {}", model.id);
        let tools = agent_service.list_tools(&model.id).await;
        println!("可用工具数量: {}", tools.len());
        for tool in &tools {
            println!("  - {}: {}", tool.name, tool.description);
        }
    }

    // 5. 创建工具配置示例
    println!("\n⚙️ 5. 工具配置示例");

    let tool_configurations = vec![
        json!({
            "name": "minimal_tools",
            "description": "最小工具集",
            "tools": ["get_current_time"]
        }),
        json!({
            "name": "full_toolset",
            "description": "完整工具集",
            "tools": ["get_current_time", "get_weather", "calculator", "file_operations"]
        }),
        json!({
            "name": "development_tools",
            "description": "开发工具集",
            "tools": ["calculator", "file_operations"]
        })
    ];

    for config in &tool_configurations {
        let name = config.get("name").unwrap().as_str().unwrap();
        let description = config.get("description").unwrap().as_str().unwrap();
        let tools: Vec<String> = config.get("tools")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect();

        println!("  配置: {} - {}", name, description);
        println!("    工具: {}", tools.join(", "));
    }

    println!("\n📊 6. 工具集成状态总结");
    println!("  ✅ 内置工具: 时间查询、天气查询");
    println!("  ✅ 自定义工具: 计算器、文件操作");
    println!("  ✅ 工具注册表: 动态工具管理");
    println!("  ✅ Agent 集成: 工具调用和结果处理");
    println!("  ✅ 配置管理: 灵活的工具配置");

    println!("\n🎉 工具集成演示完成！");
    println!("========================");

    Ok(())
}