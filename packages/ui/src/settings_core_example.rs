use crate::{
    settings_panel_core::{
        SettingsPanelCore, CoreSettingsTab, AIProvider, MCPServer, Theme, AgentConfig,
        ProviderType, AIModel, RateLimit, MCPServerStatus
    },
};
use dioxus::prelude::*;
use api::{AgentConfig, GooseMode};

#[component]
pub fn SettingsCoreExample() -> Element {
    let mut settings_open = use_signal(|| false);
    let mut active_tab = use_signal(|| CoreSettingsTab::Basic);
    let mut theme = use_signal(|| Some(Theme::Auto));
    let mut language = use_signal(|| Some("en".to_string()));
    let mut spell_check_enabled = use_signal(|| true);
    let mut auto_save_enabled = use_signal(|| true);

    // Sample AI Providers
    let providers = vec![
        AIProvider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: Some("sk-...".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            models: vec![
                AIModel {
                    id: "gpt-4".to_string(),
                    name: "GPT-4".to_string(),
                    context_limit: Some(128000),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: true,
                    pricing: Some(crate::settings_panel_core::ModelPricing {
                        input_tokens: 0.03,
                        output_tokens: 0.06,
                        currency: "USD".to_string(),
                    }),
                },
                AIModel {
                    id: "gpt-3.5-turbo".to_string(),
                    name: "GPT-3.5 Turbo".to_string(),
                    context_limit: Some(4096),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: false,
                    pricing: Some(crate::settings_panel_core::ModelPricing {
                        input_tokens: 0.001,
                        output_tokens: 0.002,
                        currency: "USD".to_string(),
                    }),
                },
            ],
            enabled: true,
            rate_limit: Some(RateLimit {
                requests_per_minute: 5000,
                tokens_per_minute: 160000,
            }),
            custom_headers: std::collections::HashMap::new(),
        },
        AIProvider {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            provider_type: ProviderType::Anthropic,
            api_key: Some("sk-ant-...".to_string()),
            base_url: Some("https://api.anthropic.com".to_string()),
            models: vec![
                AIModel {
                    id: "claude-3-opus".to_string(),
                    name: "Claude 3 Opus".to_string(),
                    context_limit: Some(200000),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: true,
                    pricing: Some(crate::settings_panel_core::ModelPricing {
                        input_tokens: 0.015,
                        output_tokens: 0.075,
                        currency: "USD".to_string(),
                    }),
                },
                AIModel {
                    id: "claude-3-sonnet".to_string(),
                    name: "Claude 3 Sonnet".to_string(),
                    context_limit: Some(200000),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: true,
                    pricing: Some(crate::settings_panel_core::ModelPricing {
                        input_tokens: 0.003,
                        output_tokens: 0.015,
                        currency: "USD".to_string(),
                    }),
                },
            ],
            enabled: true,
            rate_limit: Some(RateLimit {
                requests_per_minute: 1000,
                tokens_per_minute: 40000,
            }),
            custom_headers: std::collections::HashMap::new(),
        },
        AIProvider {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            provider_type: ProviderType::Ollama,
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            models: vec![
                AIModel {
                    id: "llama-3-70b".to_string(),
                    name: "Llama 3 70B".to_string(),
                    context_limit: Some(8192),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: false,
                    pricing: None,
                },
                AIModel {
                    id: "codellama-34b".to_string(),
                    name: "CodeLlama 34B".to_string(),
                    context_limit: Some(16384),
                    supports_tools: true,
                    supports_streaming: true,
                    supports_vision: false,
                    pricing: None,
                },
            ],
            enabled: false,
            rate_limit: None,
            custom_headers: std::collections::HashMap::new(),
        },
    ];

    // Sample MCP Servers
    let mcp_servers = vec![
        MCPServer {
            id: "filesystem".to_string(),
            name: "Filesystem Server".to_string(),
            command: "npx".to_string(),
            args: vec![
                "@modelcontextprotocol/server-filesystem".to_string(),
                "/path/to/data".to_string()
            ],
            env: std::collections::HashMap::new(),
            enabled: true,
            tools: vec![
                crate::settings_panel_core::MCPTool {
                    name: "read_file".to_string(),
                    description: "Read a file from the filesystem".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string"
                            }
                        },
                        "required": ["path"]
                    }),
                },
                crate::settings_panel_core::MCPTool {
                    name: "write_file".to_string(),
                    description: "Write content to a file".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string"
                            },
                            "content": {
                                "type": "string"
                            }
                        },
                        "required": ["path", "content"]
                    }),
                },
                crate::settings_panel_core::MCPTool {
                    name: "list_directory".to_string(),
                    description: "List directory contents".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            ],
            status: MCPServerStatus::Running,
        },
        MCPServer {
            id: "git".to_string(),
            name: "Git Repository".to_string(),
            command: "git".to_string(),
            args: vec!["status".to_string(), "--porcelain".to_string()],
            env: std::collections::HashMap::new(),
            enabled: false,
            tools: vec![
                crate::settings_panel_core::MCPTool {
                    name: "git_status".to_string(),
                    description: "Get git repository status".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                },
                crate::settings_panel_core::MCPTool {
                    name: "git_diff".to_string(),
                    description: "Show git diff".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "files": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                }
                            }
                        },
                        "required": []
                    }),
                },
            ],
            status: MCPServerStatus::Stopped,
        },
        MCPServer {
            id: "memory".to_string(),
            name: "Memory Server".to_string(),
            command: "python".to_string(),
            args: vec!["memory_server.py".to_string()],
            env: std::collections::HashMap::new(),
            enabled: false,
            tools: vec![
                crate::settings_panel_core::MCPTool {
                    name: "create_memory".to_string(),
                    description: "Create a new memory entry".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string"
                            },
                            "metadata": {
                                "type": "object"
                            }
                        },
                        "required": ["content"]
                    }),
                },
                crate::settings_panel_core::MCPTool {
                    name: "search_memory".to_string(),
                    description: "Search through memory entries".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string"
                            }
                        },
                        "required": ["query"]
                    }),
                },
            ],
            status: MCPServerStatus::Stopped,
        },
    ];

    let agent_config = AgentConfig {
        max_iterations: 10,
        require_confirmation: true,
        readonly_tools: vec![],
        enable_tool_inspection: true,
        enable_auto_compact: true,
        compact_threshold: 0.8,
        max_turns_without_tools: 3,
        enable_autopilot: false,
        enable_extensions: true,
        extension_timeout: 30,
        goose_mode: GooseMode::Agent,
    };

    rsx! {
        div {
            class: "min-h-screen bg-gray-50 dark:bg-gray-900 p-8",
            div {
                class: "max-w-4xl mx-auto",
                div {
                    class: "text-center mb-8",
                    h1 {
                        class: "text-3xl font-bold text-gray-900 dark:text-gray-100 mb-4",
                        "Dioxus Chat Core Settings"
                    }
                    p {
                        class: "text-gray-600 dark:text-gray-400 mb-6",
                        "Complete settings panel with Basic, AI Provider, MCP, and Agent configuration"
                    }
                    Button {
                        onclick: move |_| settings_open.set(true),
                        variant: crate::components::button::ButtonVariant::Primary,
                        "Open Settings"
                    }
                }

                // Feature Overview Cards
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-6 mb-8",

                    // Basic Settings Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "⚙️"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Basic Settings"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Configure theme, language, spell checking, and conversation settings."
                        }
                        div {
                            class: "grid grid-cols-2 gap-4 text-xs",
                            div {
                                span {
                                    class: "text-gray-500 dark:text-gray-500",
                                    "Theme:"
                                }
                                span {
                                    class: "text-gray-700 dark:text-gray-300",
                                    "{theme.as_ref().map_or(String::new(), |t| format!("{:?}", t))}"
                                }
                            }
                            div {
                                span {
                                    class: "text-gray-500 dark:text-gray-500",
                                    "Language:"
                                }
                                span {
                                    class: "text-gray-700 dark:text-300",
                                    "{language.as_ref().unwrap_or_default()}"
                                }
                            }
                            div {
                                span {
                                    class: "text-gray-500 dark:text-gray-500",
                                    "Spell Check:"
                                }
                                span {
                                    class: "text-gray-700 dark:text-gray-300",
                                    "{if *spell_check_enabled.read() { 'Enabled' } else { 'Disabled' }}"
                                }
                            }
                            div {
                                span {
                                    class: "text-gray-500 dark:text-gray-500",
                                    "Auto Save:"
                                }
                                span {
                                    class: "text-gray-700 dark:text-gray-300",
                                    "{if *auto_save_enabled.read() { 'Enabled' } else { 'Disabled' }}"
                                }
                            }
                        }
                    }

                    // AI Providers Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "🤖"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "AI Providers"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Manage AI model providers, API keys, and model configurations."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "{providers.len()} providers • {providers.iter().map(|p| p.models.len()).sum::<usize>()} models"
                        }
                    }

                    // MCP Servers Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "🔌"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "MCP Servers"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Configure Model Context Protocol servers for tool functionality."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "{mcp_servers.len()} servers • {mcp_servers.iter().map(|s| s.tools.len()).sum::<usize>()} tools"
                        }
                    }

                    // Agent Settings Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "🧠"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Agent Settings"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Configure agent behavior, tool usage, and advanced agent features."
                        }
                        div {
                            class: "grid grid-cols-2 gap-4 text-xs",
                            div {
                                span {
                                    class: "text-gray-500 dark:text-gray-500",
                                    "Mode:"
                                }
                                span {
                                    class: "text-gray-700 dark:text-gray-300",
                                    "{agent_config.goose_mode:?}"
                                }
                            }
                            div {
                                span {
                                    class: "text-gray-500 dark:text-gray-500",
                                    "Max Iterations:"
                                }
                                span {
                                    class: "text-gray-700 dark:text-gray-300",
                                    "{agent_config.max_iterations}"
                                }
                            }
                            div {
                                span {
                                    class: "text-gray-500 dark:text-gray-500",
                                    "Tool Confirmation:"
                                }
                                span {
                                    class: "text-gray-700 dark:text-gray-300",
                                    "{if agent_config.require_confirmation { 'Required' } else { 'Not Required' }}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Settings Modal
        SettingsPanelCore {
            open: *settings_open.read(),
            on_open_change: move |open| settings_open.set(open),
            active_tab: active_tab.read().clone(),
            on_tab_change: move |tab| active_tab.set(tab),

            // Basic Settings Props
            theme: theme.read().clone(),
            on_theme_change: move |new_theme| theme.set(Some(new_theme)),
            language: language.read().clone(),
            on_language_change: move |new_lang| language.set(Some(new_lang)),
            spell_check_enabled: *spell_check_enabled.read(),
            on_spell_check_change: move |enabled| spell_check_enabled.set(enabled),
            auto_save_enabled: *auto_save_enabled.read(),
            on_auto_save_change: move |enabled| auto_save_enabled.set(enabled),

            // AI Provider Props
            providers: providers.clone(),
            on_add_provider: move |provider| {
                // TODO: Handle provider addition
            },
            on_update_provider: move |provider| {
                // TODO: Handle provider update
            },
            on_remove_provider: move |provider_id| {
                // TODO: Handle provider removal
            },
            selected_provider: None,
            on_select_provider: move |provider_id| {
                // TODO: Handle provider selection
            },

            // MCP Server Props
            mcp_servers: mcp_servers.clone(),
            on_add_mcp_server: move |server| {
                // TODO: Handle MCP server addition
            },
            on_update_mcp_server: move |server| {
                // TODO: Handle MCP server update
            },
            on_remove_mcp_server: move |server_id| {
                // TODO: Handle MCP server removal
            },
            on_toggle_mcp_server: move |server_id| {
                // TODO: Handle MCP server toggle
            },

            // Agent Settings Props
            agent_config: Some(agent_config),
            on_agent_config_change: move |config| {
                // TODO: Handle agent config change
            },
        }
    }
}