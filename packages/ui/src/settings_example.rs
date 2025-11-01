use crate::{
    settings_panel_complete::{
        SettingsPanelComplete, SettingsTabComplete, SettingsModelComplete, Provider,
        DataSource, Shortcut, PerformanceSettings, ModelPricing, RateLimit,
        DataSourceType, SyncFrequency, ShortcutCategory, LogLevel
    },
    settings_menu::Theme,
};
use dioxus::prelude::*;
use api::{AgentConfig, GooseMode};

#[component]
pub fn SettingsExample() -> Element {
    let mut settings_open = use_signal(|| false);
    let mut active_tab = use_signal(SettingsTabComplete::General);
    let mut selected_model = use_signal(|| Some("gpt-4".to_string()));
    let mut theme = use_signal(|| Some(Theme::Auto));

    // Sample data
    let models = vec![
        SettingsModelComplete {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
            description: Some("Most capable GPT-4 model with advanced reasoning".to_string()),
            capabilities: vec!["chat".to_string(), "tools".to_string(), "vision".to_string(), "function_calling".to_string()],
            context_limit: Some(128000),
            supports_tools: true,
            supports_streaming: true,
            supports_vision: true,
            supports_function_calling: true,
            pricing: Some(ModelPricing {
                input_tokens: 0.03,
                output_tokens: 0.06,
                currency: "USD".to_string(),
            }),
        },
        SettingsModelComplete {
            id: "claude-3-opus".to_string(),
            name: "Claude 3 Opus".to_string(),
            provider: "Anthropic".to_string(),
            description: Some("Most intelligent Claude model for complex tasks".to_string()),
            capabilities: vec!["chat".to_string(), "tools".to_string(), "vision".to_string(), "analysis".to_string()],
            context_limit: Some(200000),
            supports_tools: true,
            supports_streaming: true,
            supports_vision: true,
            supports_function_calling: true,
            pricing: Some(ModelPricing {
                input_tokens: 0.015,
                output_tokens: 0.075,
                currency: "USD".to_string(),
            }),
        },
        SettingsModelComplete {
            id: "llama-3-70b".to_string(),
            name: "Llama 3 70B".to_string(),
            provider: "Meta".to_string(),
            description: Some("Open-source model with excellent performance".to_string()),
            capabilities: vec!["chat".to_string(), "tools".to_string()],
            context_limit: Some(8192),
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            supports_function_calling: true,
            pricing: None,
        },
    ];

    let providers = vec![
        Provider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            api_key: Some("sk-...".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            models: models.iter().filter(|m| m.provider == "OpenAI").cloned().collect(),
            active: true,
            rate_limit: Some(RateLimit {
                requests_per_minute: 5000,
                tokens_per_minute: 160000,
            }),
            custom_headers: std::collections::HashMap::new(),
        },
        Provider {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            api_key: Some("sk-ant-...".to_string()),
            base_url: Some("https://api.anthropic.com".to_string()),
            models: models.iter().filter(|m| m.provider == "Anthropic").cloned().collect(),
            active: true,
            rate_limit: Some(RateLimit {
                requests_per_minute: 1000,
                tokens_per_minute: 40000,
            }),
            custom_headers: std::collections::HashMap::new(),
        },
    ];

    let data_sources = vec![
        DataSource {
            id: "docs".to_string(),
            name: "Documentation".to_string(),
            type_: DataSourceType::LocalFile,
            url: Some("/path/to/docs".to_string()),
            api_key: None,
            enabled: true,
            last_sync: Some("2 hours ago".to_string()),
            sync_frequency: SyncFrequency::Daily,
        },
        DataSource {
            id: "github-repo".to_string(),
            name: "GitHub Repository".to_string(),
            type_: DataSourceType::GitHub,
            url: Some("https://github.com/user/repo".to_string()),
            api_key: Some("ghp_...".to_string()),
            enabled: true,
            last_sync: Some("1 day ago".to_string()),
            sync_frequency: SyncFrequency::Hourly,
        },
        DataSource {
            id: "notion-db".to_string(),
            name: "Notion Database".to_string(),
            type_: DataSourceType::Notion,
            url: Some("https://notion.so/...".to_string()),
            api_key: Some("secret_...".to_string()),
            enabled: false,
            last_sync: None,
            sync_frequency: SyncFrequency::Manual,
        },
    ];

    let shortcuts = vec![
        Shortcut {
            id: "new_chat".to_string(),
            name: "New Chat".to_string(),
            description: "Start a new conversation".to_string(),
            default_keys: vec!["Ctrl+N".to_string()],
            current_keys: vec!["Ctrl+N".to_string()],
            category: ShortcutCategory::Navigation,
        },
        Shortcut {
            id: "send_message".to_string(),
            name: "Send Message".to_string(),
            description: "Send the current message".to_string(),
            default_keys: vec!["Ctrl+Enter".to_string()],
            current_keys: vec!["Ctrl+Enter".to_string()],
            category: ShortcutCategory::Chat,
        },
        Shortcut {
            id: "toggle_sidebar".to_string(),
            name: "Toggle Sidebar".to_string(),
            description: "Show or hide the sidebar".to_string(),
            default_keys: vec!["Ctrl+B".to_string()],
            current_keys: vec!["Ctrl+B".to_string()],
            category: ShortcutCategory::Window,
        },
        Shortcut {
            id: "settings".to_string(),
            name: "Open Settings".to_string(),
            description: "Open the settings panel".to_string(),
            default_keys: vec!["Ctrl+,".to_string()],
            current_keys: vec!["Ctrl+,".to_string()],
            category: ShortcutCategory::System,
        },
    ];

    let performance_settings = PerformanceSettings {
        max_concurrent_requests: 5,
        cache_size_mb: 512,
        streaming_buffer_size: 8192,
        enable_gpu_acceleration: false,
        memory_limit_mb: 2048,
        network_timeout_seconds: 30,
        enable_compression: true,
        log_level: LogLevel::Info,
    };

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
                        "Dioxus Chat Settings Demo"
                    }
                    p {
                        class: "text-gray-600 dark:text-gray-400 mb-6",
                        "Complete settings panel with all advanced features"
                    }
                    Button {
                        onclick: move |_| settings_open.set(true),
                        variant: crate::components::button::ButtonVariant::Primary,
                        "Open Settings"
                    }
                }

                // Feature cards
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8",

                    // Models Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "🤖"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Models & Providers"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Configure AI models, providers, and advanced model settings including pricing and capabilities."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "{models.len()} models • {providers.len()} providers"
                        }
                    }

                    // Data Sources Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "📊"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Data Sources"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Connect external data sources like GitHub, Notion, databases, and local files."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "{data_sources.len()} data sources configured"
                        }
                    }

                    // Performance Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "⚡"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Performance"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Optimize performance with caching, GPU acceleration, and advanced memory management."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "Cache: {performance_settings.cache_size_mb}MB • Limit: {performance_settings.memory_limit_mb}MB"
                        }
                    }

                    // Shortcuts Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "⌨️"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Keyboard Shortcuts"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Customize keyboard shortcuts for navigation, editing, and system operations."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "{shortcuts.len()} shortcuts configured"
                        }
                    }

                    // Agent Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "🧠"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Agent Configuration"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Advanced agent settings including planning, memory, and tool usage optimization."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "Mode: {agent_config.goose_mode:?} • Iterations: {agent_config.max_iterations}"
                        }
                    }

                    // Appearance Card
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm",
                        div {
                            class: "text-2xl mb-3",
                            "🎨"
                        }
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                            "Appearance"
                        }
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                            "Customize themes, fonts, colors, and visual preferences for optimal user experience."
                        }
                        div {
                            class: "text-xs text-gray-500 dark:text-gray-500",
                            "Theme: {theme.as_ref().map_or(String::new(), |t| format!("{:?}", t))}"
                        }
                    }
                }
            }
        }

        // Settings Modal
        SettingsPanelComplete {
            open: *settings_open.read(),
            on_open_change: move |open| settings_open.set(open),
            active_tab: active_tab.read().clone(),
            on_tab_change: move |tab| active_tab.set(tab),
            models,
            selected_model: selected_model.read().clone(),
            on_select_model: move |model_id| selected_model.set(Some(model_id)),
            theme: theme.read().clone(),
            on_theme_change: move |new_theme| theme.set(Some(new_theme)),
            agent_config: Some(agent_config),
            on_agent_config_change: move |config| {
                // TODO: Handle agent config change
            },
            providers,
            on_add_provider: move |provider| {
                // TODO: Handle provider addition
            },
            on_remove_provider: move |provider_id| {
                // TODO: Handle provider removal
            },
            on_update_provider: move |provider| {
                // TODO: Handle provider update
            },
            shortcuts,
            on_shortcut_change: move |(id, keys)| {
                // TODO: Handle shortcut change
            },
            data_sources,
            on_add_data_source: move |source| {
                // TODO: Handle data source addition
            },
            on_remove_data_source: move |source_id| {
                // TODO: Handle data source removal
            },
            performance_settings,
            on_performance_change: move |settings| {
                // TODO: Handle performance settings change
            },
        }
    }
}