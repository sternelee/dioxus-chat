use crate::components::{
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogContent, DialogHeader, DialogTitle},
    dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger},
    input::Input,
    separator::Separator,
    switch::Switch,
    tabs::{TabContent, TabList, TabTrigger, Tabs},
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};
use api::{AgentConfig, GooseMode};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct SettingsPanelCoreProps {
    pub open: bool,
    pub on_open_change: EventHandler<bool>,
    pub active_tab: CoreSettingsTab,
    pub on_tab_change: EventHandler<CoreSettingsTab>,

    // Basic Settings
    pub theme: Option<Theme>,
    pub on_theme_change: EventHandler<Theme>,
    pub language: Option<String>,
    pub on_language_change: EventHandler<String>,
    pub spell_check_enabled: bool,
    pub on_spell_check_change: EventHandler<bool>,
    pub auto_save_enabled: bool,
    pub on_auto_save_change: EventHandler<bool>,

    // AI Provider Settings
    pub providers: Vec<AIProvider>,
    pub on_add_provider: EventHandler<AIProvider>,
    pub on_update_provider: EventHandler<AIProvider>,
    pub on_remove_provider: EventHandler<String>,
    pub selected_provider: Option<String>,
    pub on_select_provider: EventHandler<String>,

    // MCP Settings
    pub mcp_servers: Vec<MCPServer>,
    pub on_add_mcp_server: EventHandler<MCPServer>,
    pub on_update_mcp_server: EventHandler<MCPServer>,
    pub on_remove_mcp_server: EventHandler<String>,
    pub on_toggle_mcp_server: EventHandler<String>,

    // Agent Settings
    pub agent_config: Option<AgentConfig>,
    pub on_agent_config_change: EventHandler<AgentConfig>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum CoreSettingsTab {
    Basic,
    Providers,
    MCP,
    Agent,
}

#[derive(Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[derive(Clone, PartialEq)]
pub struct AIProvider {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<AIModel>,
    pub enabled: bool,
    pub rate_limit: Option<RateLimit>,
    pub custom_headers: std::collections::HashMap<String, String>,
}

#[derive(Clone, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Ollama,
    Local,
    Custom(String),
}

#[derive(Clone, PartialEq)]
pub struct AIModel {
    pub id: String,
    pub name: String,
    pub context_limit: Option<usize>,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_vision: bool,
    pub pricing: Option<ModelPricing>,
}

#[derive(Clone, PartialEq)]
pub struct ModelPricing {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub currency: String,
}

#[derive(Clone, PartialEq)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
}

#[derive(Clone, PartialEq)]
pub struct MCPServer {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub enabled: bool,
    pub tools: Vec<MCPTool>,
    pub status: MCPServerStatus,
}

#[derive(Clone, PartialEq, Debug)]
pub enum MCPServerStatus {
    Running,
    Stopped,
    Error(String),
}

#[derive(Clone, PartialEq)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[component]
pub fn SettingsPanelCore(props: SettingsPanelCoreProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| props.on_open_change.call(false),

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-5xl w-full mx-4 max-h-[90vh] flex flex-col",
                onclick: move |e: dioxus::prelude::Event<MouseData>| e.stop_propagation(),

                // Header
                div {
                    class: "px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between",
                    div {
                        class: "flex items-center gap-3",
                        h2 {
                            class: "text-xl font-semibold text-gray-900 dark:text-gray-100",
                            "Settings"
                        }
                        span {
                            class: "text-sm text-gray-500 dark:text-gray-400",
                            "Configure your chat experience"
                        }
                    }
                    Button {
                        onclick: move |_| props.on_open_change.call(false),
                        variant: ButtonVariant::Ghost,
                        class: "w-8 h-8 p-0",
                        "×"
                    }
                }

                // Tabs
                div {
                    class: "px-6 pt-4 flex-1 overflow-hidden",
                    Tabs {
                        value: format!("{:?}", props.active_tab),
                        on_value_change: move |value: String| {
                            match value.as_str() {
                                "Basic" => props.on_tab_change.call(CoreSettingsTab::Basic),
                                "Providers" => props.on_tab_change.call(CoreSettingsTab::Providers),
                                "MCP" => props.on_tab_change.call(CoreSettingsTab::MCP),
                                "Agent" => props.on_tab_change.call(CoreSettingsTab::Agent),
                                _ => {}
                            }
                        },

                        TabsList {
                            TabsTrigger {
                                value: "Basic",
                                "Basic"
                            }
                            TabsTrigger {
                                value: "Providers",
                                "AI Providers"
                            }
                            TabsTrigger {
                                value: "MCP",
                                "MCP Servers"
                            }
                            TabsTrigger {
                                value: "Agent",
                                "Agent"
                            }
                        }

                        // Tab Contents
                        div {
                            class: "overflow-y-auto max-h-[60vh]",

                            // Basic Settings Tab
                            TabsContent {
                                value: "Basic",
                                BasicSettingsTab {
                                    theme: props.theme.clone(),
                                    on_theme_change: props.on_theme_change,
                                    language: props.language.clone(),
                                    on_language_change: props.on_language_change,
                                    spell_check_enabled: props.spell_check_enabled,
                                    on_spell_check_change: props.on_spell_check_change,
                                    auto_save_enabled: props.auto_save_enabled,
                                    on_auto_save_change: props.on_auto_save_change,
                                }
                            }

                            // AI Providers Tab
                            TabsContent {
                                value: "Providers",
                                AIProvidersTab {
                                    providers: props.providers.clone(),
                                    selected_provider: props.selected_provider.clone(),
                                    on_select_provider: props.on_select_provider,
                                    on_add_provider: props.on_add_provider,
                                    on_update_provider: props.on_update_provider,
                                    on_remove_provider: props.on_remove_provider,
                                }
                            }

                            // MCP Servers Tab
                            TabsContent {
                                value: "MCP",
                                MCPServersTab {
                                    servers: props.mcp_servers.clone(),
                                    on_add_server: props.on_add_mcp_server,
                                    on_update_server: props.on_update_mcp_server,
                                    on_remove_server: props.on_remove_mcp_server,
                                    on_toggle_server: props.on_toggle_mcp_server,
                                }
                            }

                            // Agent Settings Tab
                            TabsContent {
                                value: "Agent",
                                AgentSettingsTab {
                                    agent_config: props.agent_config.clone(),
                                    on_agent_config_change: props.on_agent_config_change,
                                }
                            }
                        }
                    }
                }

                // Footer
                div {
                    class: "px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-2",
                    Button {
                        onclick: move |_| props.on_open_change.call(false),
                        variant: ButtonVariant::Ghost,
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| props.on_open_change.call(false),
                        variant: ButtonVariant::Primary,
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn BasicSettingsTab(
    theme: Option<Theme>,
    on_theme_change: EventHandler<Theme>,
    language: Option<String>,
    on_language_change: EventHandler<String>,
    spell_check_enabled: bool,
    on_spell_check_change: EventHandler<bool>,
    auto_save_enabled: bool,
    on_auto_save_change: EventHandler<bool>,
) -> Element {
    rsx! {
        div {
            class: "space-y-6 p-4",

            // Appearance Section
            div {
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                    "Appearance"
                }
                div {
                    class: "space-y-4",
                    // Theme
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "Theme"
                        }
                        div {
                            class: "flex gap-2",
                            Button {
                                onclick: move |_| on_theme_change.call(Theme::Light),
                                variant: if matches!(theme, Some(Theme::Light)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                                "Light"
                            }
                            Button {
                                onclick: move |_| on_theme_change.call(Theme::Dark),
                                variant: if matches!(theme, Some(Theme::Dark)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                                "Dark"
                            }
                            Button {
                                onclick: move |_| on_theme_change.call(Theme::Auto),
                                variant: if matches!(theme, Some(Theme::Auto)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                                "Auto"
                            }
                        }
                    }
                }
            }

            Separator {}

            // Language Section
            div {
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                    "Language & Region"
                }
                div {
                    class: "space-y-4",
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Language"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            onchange: move |evt| on_language_change.call(evt.value()),
                            option {
                                value: "en",
                                selected: language.as_ref().map_or(false, |l| l == "en"),
                                "English"
                            }
                            option {
                                value: "zh",
                                selected: language.as_ref().map_or(false, |l| l == "zh"),
                                "中文"
                            }
                            option {
                                value: "ja",
                                selected: language.as_ref().map_or(false, |l| l == "ja"),
                                "日本語"
                            }
                        }
                    }
                }
            }

            Separator {}

            // Conversation Section
            div {
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                    "Conversation"
                }
                div {
                    class: "space-y-4",
                    // Spell Check
                    div {
                        div {
                            class: "flex items-center justify-between",
                            label {
                                class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                "Spell Check"
                            }
                            Switch {
                                checked: spell_check_enabled,
                                on_checked_change: on_spell_check_change,
                            }
                        }
                        p {
                            class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                            "Enable spell checking for chat input"
                        }
                    }
                    // Auto Save
                    div {
                        div {
                            class: "flex items-center justify-between",
                            label {
                                class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                "Auto-save Conversations"
                            }
                            Switch {
                                checked: auto_save_enabled,
                                on_checked_change: on_auto_save_change,
                            }
                        }
                        p {
                            class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                            "Automatically save conversation history"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AIProvidersTab(
    providers: Vec<AIProvider>,
    selected_provider: Option<String>,
    on_select_provider: EventHandler<String>,
    on_add_provider: EventHandler<AIProvider>,
    on_update_provider: EventHandler<AIProvider>,
    on_remove_provider: EventHandler<String>,
) -> Element {
    let mut show_add_provider = use_signal(|| false);

    rsx! {
        div {
            class: "space-y-6 p-4",

            // Header
            div {
                class: "flex items-center justify-between",
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100",
                    "AI Providers"
                }
                Button {
                    onclick: move |_| show_add_provider.set(true),
                    variant: ButtonVariant::Primary,
                    "+ Add Provider"
                }
            }

            // Providers List
            div {
                class: "space-y-4",
                if providers.is_empty() {
                    div {
                        class: "text-center py-8 text-gray-500 dark:text-gray-400",
                        div {
                            class: "text-4xl mb-2",
                            "🤖"
                        }
                        p {
                            class: "text-sm",
                            "No AI providers configured"
                        }
                        p {
                            class: "text-xs mt-1",
                            "Add your first AI provider to start chatting"
                        }
                    }
                } else {
                    {providers.iter().map(|provider| {
                        rsx! {
                            ProviderCard {
                                provider: provider.clone(),
                                selected: selected_provider.as_ref() == Some(&provider.id),
                                on_select: on_select_provider,
                                on_update: on_update_provider,
                                on_remove: on_remove_provider,
                            }
                        }
                    })}
                }
            }

            // Add Provider Dialog
            if *show_add_provider.read() {
                AddProviderDialog {
                    open: true,
                    on_open_change: move |open| show_add_provider.set(open),
                    on_add_provider,
                }
            }
        }
    }
}

#[component]
fn ProviderCard(
    provider: AIProvider,
    selected: bool,
    on_select: EventHandler<String>,
    on_update: EventHandler<AIProvider>,
    on_remove: EventHandler<String>,
) -> Element {
    let mut show_edit = use_signal(|| false);
    let provider_id = provider.id.clone();

    rsx! {
        div {
            class: "border border-gray-200 dark:border-gray-700 rounded-lg p-4 {
                if selected { 'bg-blue-50 dark:bg-blue-900/20' } else { '' }
            }",
            div {
                class: "flex items-center justify-between mb-3",
                div {
                    class: "flex items-center gap-3",
                    div {
                        class: "w-10 h-10 bg-blue-500 rounded-lg flex items-center justify-center text-white text-sm font-bold",
                        "{provider.name.chars().next().unwrap_or('P')}"
                    }
                    div {
                        h4 {
                            class: "font-medium text-gray-900 dark:text-gray-100",
                            "{provider.name}"
                        }
                        p {
                            class: "text-sm text-gray-500 dark:text-gray-400",
                            "{provider.provider_type:?} • {provider.models.len()} models • {if provider.enabled { 'Active' } else { 'Inactive' }}"
                        }
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    div {
                        class: "w-2 h-2 rounded-full",
                        class: if provider.enabled { "bg-green-500" } else { "bg-gray-400" }
                    }
                    DropdownMenu {
                        DropdownMenuTrigger {
                            Button {
                                variant: ButtonVariant::Ghost,
                                class: "text-sm",
                                "⋯"
                            }
                        }
                        DropdownMenuContent {
                            DropdownMenuItem::<String> {
                                value: "edit".to_string(),
                                index: 0usize,
                                on_select: move |_: String| show_edit.set(true),
                                "Edit"
                            }
                            DropdownMenuItem::<String> {
                                value: "remove".to_string(),
                                index: 1usize,
                                on_select: move |_: String| on_remove.call(provider_id.clone()),
                                class: "text-red-600 dark:text-red-400",
                                "Remove"
                            }
                        }
                    }
                }

                // Provider details
                if let Some(base_url) = &provider.base_url {
                    div {
                        class: "text-sm text-gray-600 dark:text-gray-400 mb-2",
                        "Base URL: {base_url}"
                    }
                }

                if let Some(rate_limit) = &provider.rate_limit {
                    div {
                        class: "text-sm text-gray-600 dark:text-gray-400 mb-2",
                        "Rate Limit: {rate_limit.requests_per_minute} req/min, {rate_limit.tokens_per_minute} tokens/min"
                    }
                }

                // Models list
                if !provider.models.is_empty() {
                    div {
                        class: "mt-3",
                        p {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Available Models:"
                        }
                        div {
                            class: "grid grid-cols-2 gap-2",
                            {provider.models.iter().map(|model| {
                                rsx! {
                                    div {
                                        class: "text-xs bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded",
                                        title: if let Some(limit) = model.context_limit {
                                            format!("Context: {} tokens", limit)
                                        } else {
                                            "No context limit".to_string()
                                        },
                                        "{model.name}"
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }

        // Edit Dialog
        if *show_edit.read() {
            EditProviderDialog {
                open: true,
                on_open_change: move |open| show_edit.set(open),
                provider: provider.clone(),
                on_update,
            }
        }
    }
}

#[component]
fn AddProviderDialog(
    open: bool,
    on_open_change: EventHandler<bool>,
    on_add_provider: EventHandler<AIProvider>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut provider_type = use_signal(|| ProviderType::OpenAI);
    let mut base_url = use_signal(|| String::new());
    let mut api_key = use_signal(|| String::new());

    rsx! {
        Dialog {
            open,
            on_open_change,
            DialogContent {
                class: "max-w-md",
                DialogHeader {
                    DialogTitle {
                        "Add AI Provider"
                    }
                }
                div {
                    class: "space-y-4",
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Provider Name"
                        }
                        Input {
                            value: name.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| name.set(evt.value()),
                            placeholder: "e.g., OpenAI, Anthropic",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Provider Type"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            onchange: move |evt| {
                                provider_type.set(match evt.value().as_str() {
                                    "OpenAI" => ProviderType::OpenAI,
                                    "Anthropic" => ProviderType::Anthropic,
                                    "Ollama" => ProviderType::Ollama,
                                    "Local" => ProviderType::Local,
                                    _ => ProviderType::Custom(evt.value()),
                                });
                            },
                            option {
                                value: "OpenAI",
                                selected: matches!(*provider_type.read(), ProviderType::OpenAI),
                                "OpenAI"
                            }
                            option {
                                value: "Anthropic",
                                selected: matches!(*provider_type.read(), ProviderType::Anthropic),
                                "Anthropic"
                            }
                            option {
                                value: "Ollama",
                                selected: matches!(*provider_type.read(), ProviderType::Ollama),
                                "Ollama"
                            }
                            option {
                                value: "Local",
                                selected: matches!(*provider_type.read(), ProviderType::Local),
                                "Local"
                            }
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Base URL (Optional)"
                        }
                        Input {
                            value: base_url.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| base_url.set(evt.value()),
                            placeholder: "https://api.openai.com/v1",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "API Key"
                        }
                        Input {
                            r#type: "password",
                            value: api_key.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| api_key.set(evt.value()),
                            placeholder: "sk-...",
                        }
                    }
                }
                div {
                    class: "flex justify-end gap-2 pt-4",
                    Button {
                        onclick: move |_| on_open_change.call(false),
                        variant: ButtonVariant::Ghost,
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            if !name.read().is_empty() {
                                on_add_provider.call(AIProvider {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: name.read().clone(),
                                    provider_type: provider_type.read().clone(),
                                    base_url: if base_url.read().is_empty() { None } else { Some(base_url.read().clone()) },
                                    api_key: if api_key.read().is_empty() { None } else { Some(api_key.read().clone()) },
                                    models: vec![],
                                    enabled: true,
                                    rate_limit: None,
                                    custom_headers: std::collections::HashMap::new(),
                                });
                                name.set(String::new());
                                base_url.set(String::new());
                                api_key.set(String::new());
                                on_open_change.call(false);
                            }
                        },
                        variant: ButtonVariant::Primary,
                        "Add Provider"
                    }
                }
            }
        }
    }
}

#[component]
fn EditProviderDialog(
    open: bool,
    on_open_change: EventHandler<bool>,
    provider: AIProvider,
    on_update: EventHandler<AIProvider>,
) -> Element {
    let mut name: Signal<String> = use_signal(|| provider.name.clone());
    let mut base_url: Signal<String> = use_signal(|| provider.base_url.clone().unwrap_or_default());
    let mut api_key: Signal<String> = use_signal(|| provider.api_key.clone().unwrap_or_default());

    rsx! {
        Dialog {
            open,
            on_open_change,
            DialogContent {
                class: "max-w-md",
                DialogHeader {
                    DialogTitle {
                        "Edit AI Provider"
                    }
                }
                div {
                    class: "space-y-4",
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Provider Name"
                        }
                        Input {
                            value: name.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| name.set(evt.value()),
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Base URL"
                        }
                        Input {
                            value: base_url.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| base_url.set(evt.value()),
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "API Key"
                        }
                        Input {
                            r#type: "password",
                            value: api_key.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| api_key.set(evt.value()),
                        }
                    }
                }
                div {
                    class: "flex justify-end gap-2 pt-4",
                    Button {
                        onclick: move |_| on_open_change.call(false),
                        variant: ButtonVariant::Ghost,
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            let mut updated_provider = provider.clone();
                            updated_provider.name = name.read().clone();
                            updated_provider.base_url = if base_url.read().is_empty() { None } else { Some(base_url.read().clone()) };
                            updated_provider.api_key = if api_key.read().is_empty() { None } else { Some(api_key.read().clone()) };
                            on_update.call(updated_provider);
                            on_open_change.call(false);
                        },
                        variant: ButtonVariant::Primary,
                        "Save Changes"
                    }
                }
            }
        }
    }
}

#[component]
fn MCPServersTab(
    servers: Vec<MCPServer>,
    on_add_server: EventHandler<MCPServer>,
    on_update_server: EventHandler<MCPServer>,
    on_remove_server: EventHandler<String>,
    on_toggle_server: EventHandler<String>,
) -> Element {
    let mut show_add_server = use_signal(|| false);

    rsx! {
        div {
            class: "space-y-6 p-4",

            // Header
            div {
                class: "flex items-center justify-between",
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100",
                    "MCP Servers"
                }
                Button {
                    onclick: move |_| show_add_server.set(true),
                    variant: ButtonVariant::Primary,
                    "+ Add Server"
                }
            }

            // Servers List
            div {
                class: "space-y-4",
                if servers.is_empty() {
                    div {
                        class: "text-center py-8 text-gray-500 dark:text-gray-400",
                        div {
                            class: "text-4xl mb-2",
                            "🔌"
                        }
                        p {
                            class: "text-sm",
                            "No MCP servers configured"
                        }
                        p {
                            class: "text-xs mt-1",
                            "Add MCP servers to enable tool functionality"
                        }
                    }
                } else {
                    {servers.iter().map(|server| {
                        rsx! {
                            MCPServerCard {
                                server: server.clone(),
                                on_toggle: on_toggle_server,
                                on_update: on_update_server,
                                on_remove: on_remove_server,
                            }
                        }
                    })}
                }
            }

            // Add Server Dialog
            if *show_add_server.read() {
                AddMCPServerDialog {
                    open: true,
                    on_open_change: move |open| show_add_server.set(open),
                    on_add_server,
                }
            }
        }
    }
}

#[component]
fn MCPServerCard(
    server: MCPServer,
    on_toggle: EventHandler<String>,
    on_update: EventHandler<MCPServer>,
    on_remove: EventHandler<String>,
) -> Element {
    let mut show_edit = use_signal(|| false);
    let toggle_server_id = server.id.clone();
    let remove_server_id = server.id.clone();

    rsx! {
        div {
            class: "border border-gray-200 dark:border-gray-700 rounded-lg p-4",
            div {
                class: "flex items-center justify-between mb-3",
                div {
                    class: "flex items-center gap-3",
                    div {
                        class: "w-10 h-10 bg-purple-500 rounded-lg flex items-center justify-center text-white text-sm font-bold",
                        "MCP"
                    }
                    div {
                        h4 {
                            class: "font-medium text-gray-900 dark:text-gray-100",
                            "{server.name}"
                        }
                        p {
                            class: "text-sm text-gray-500 dark:text-gray-400",
                            "{server.tools.len()} tools • Status: {server.status:?}"
                        }
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    Switch {
                        checked: matches!(server.status, MCPServerStatus::Running),
                        on_checked_change: move |enabled| {
                            if enabled {
                                on_toggle.call(toggle_server_id.clone());
                            } else {
                                on_toggle.call(toggle_server_id.clone());
                            }
                        },
                    }
                    DropdownMenu {
                        DropdownMenuTrigger {
                            Button {
                                variant: ButtonVariant::Ghost,
                                class: "text-sm",
                                "⋯"
                            }
                        }
                        DropdownMenuContent {
                            DropdownMenuItem::<String> {
                                value: "edit".to_string(),
                                index: 0usize,
                                on_select: move |_: String| show_edit.set(true),
                                "Edit"
                            }
                            DropdownMenuItem::<String> {
                                value: "remove".to_string(),
                                index: 1usize,
                                on_select: move |_: String| on_remove.call(remove_server_id.clone()),
                                class: "text-red-600 dark:text-red-400",
                                "Remove"
                            }
                        }
                    }
                }

                // Server details
                div {
                    class: "text-sm text-gray-600 dark:text-gray-400 mb-2",
                    "Command: {server.command}"
                }
                if !server.args.is_empty() {
                    div {
                        class: "text-sm text-gray-600 dark:text-gray-400 mb-2",
                        "Args: {server.args.join(\" \")}"
                    }
                }

                // Tools list
                if !server.tools.is_empty() {
                    div {
                        class: "mt-3",
                        p {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Available Tools:"
                        }
                        div {
                            class: "grid grid-cols-1 gap-1",
                            {server.tools.iter().map(|tool| {
                                rsx! {
                                    div {
                                        class: "text-xs bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded",
                                        title: tool.description.clone(),
                                        "{tool.name}"
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }

        // Edit Dialog
        if *show_edit.read() {
            EditMCPServerDialog {
                open: true,
                on_open_change: move |open| show_edit.set(open),
                server: server.clone(),
                on_update,
            }
        }
    }
}

#[component]
fn AddMCPServerDialog(
    open: bool,
    on_open_change: EventHandler<bool>,
    on_add_server: EventHandler<MCPServer>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut command = use_signal(|| String::new());
    let mut args = use_signal(|| String::new());

    rsx! {
        Dialog {
            open,
            on_open_change,
            DialogContent {
                class: "max-w-md",
                DialogHeader {
                    DialogTitle {
                        "Add MCP Server"
                    }
                }
                div {
                    class: "space-y-4",
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Server Name"
                        }
                        Input {
                            value: name.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| name.set(evt.value()),
                            placeholder: "e.g., Filesystem Server",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Command"
                        }
                        Input {
                            value: command.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| command.set(evt.value()),
                            placeholder: "npx",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Arguments (Optional)"
                        }
                        Input {
                            value: args.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| args.set(evt.value()),
                            placeholder: "--arg1 value1 --arg2 value2",
                        }
                    }
                }
                div {
                    class: "flex justify-end gap-2 pt-4",
                    Button {
                        onclick: move |_| on_open_change.call(false),
                        variant: ButtonVariant::Ghost,
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            if !name.read().is_empty() && !command.read().is_empty() {
                                on_add_server.call(MCPServer {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: name.read().clone(),
                                    command: command.read().clone(),
                                    args: if args.read().is_empty() { vec![] } else { args.read().split_whitespace().map(|s| s.to_string()).collect() },
                                    env: std::collections::HashMap::new(),
                                    enabled: true,
                                    tools: vec![],
                                    status: MCPServerStatus::Stopped,
                                });
                                name.set(String::new());
                                command.set(String::new());
                                args.set(String::new());
                                on_open_change.call(false);
                            }
                        },
                        variant: ButtonVariant::Primary,
                        "Add Server"
                    }
                }
            }
        }
    }
}

#[component]
fn EditMCPServerDialog(
    open: bool,
    on_open_change: EventHandler<bool>,
    server: MCPServer,
    on_update: EventHandler<MCPServer>,
) -> Element {
    let mut name = use_signal(|| server.name.clone());
    let mut command = use_signal(|| server.command.clone());
    let mut args = use_signal(|| server.args.join(" "));

    rsx! {
        Dialog {
            open,
            on_open_change,
            DialogContent {
                class: "max-w-md",
                DialogHeader {
                    DialogTitle {
                        "Edit MCP Server"
                    }
                }
                div {
                    class: "space-y-4",
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Server Name"
                        }
                        Input {
                            value: name.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| name.set(evt.value()),
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Command"
                        }
                        Input {
                            value: command.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| command.set(evt.value()),
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Arguments"
                        }
                        Input {
                            value: args.read().clone(),
                            oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| args.set(evt.value()),
                        }
                    }
                }
                div {
                    class: "flex justify-end gap-2 pt-4",
                    Button {
                        onclick: move |_| on_open_change.call(false),
                        variant: ButtonVariant::Ghost,
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            let mut updated_server = server.clone();
                            updated_server.name = name.read().clone();
                            updated_server.command = command.read().clone();
                            updated_server.args = args.read().split_whitespace().map(|s| s.to_string()).collect();
                            on_update.call(updated_server);
                            on_open_change.call(false);
                        },
                        variant: ButtonVariant::Primary,
                        "Save Changes"
                    }
                }
            }
        }
    }
}

#[component]
fn AgentSettingsTab(
    agent_config: Option<AgentConfig>,
    on_agent_config_change: EventHandler<AgentConfig>,
) -> Element {
    let config = agent_config.unwrap_or_default();
    let mut config_signal = use_signal(|| config);

    rsx! {
        div {
            class: "space-y-6 p-4",

            // Agent Mode
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Agent Configuration"
            }
            div {
                class: "space-y-4",

                // Goose Mode
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Agent Mode"
                    }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                        onchange: move |evt| {
                            let mut config = config_signal.read().clone();
                            config.goose_mode = match evt.value().as_str() {
                                "Chat" => GooseMode::Chat,
                                "Agent" => GooseMode::Agent,
                                "Auto" => GooseMode::Auto,
                                _ => GooseMode::Agent,
                            };
                            config_signal.set(config.clone());
                            on_agent_config_change.call(config);
                        },
                        option {
                            value: "Chat",
                            selected: matches!(config_signal.read().goose_mode, GooseMode::Chat),
                            "Chat - Simple conversation mode"
                        }
                        option {
                            value: "Agent",
                            selected: matches!(config_signal.read().goose_mode, GooseMode::Agent),
                            "Agent - Full agent capabilities with tools"
                        }
                        option {
                            value: "Auto",
                            selected: matches!(config_signal.read().goose_mode, GooseMode::Auto),
                            "Auto - Automatically choose best mode"
                        }
                    }
                }

                // Max Iterations
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Max Iterations"
                    }
                    Input {
                        value: config_signal.read().max_iterations.to_string(),
                        oninput: move |event: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                            if let Ok(iterations) = event.value().parse::<usize>() {
                                let mut config = config_signal.read().clone();
                                config.max_iterations = iterations;
                                config_signal.set(config.clone());
                                on_agent_config_change.call(config);
                            }
                        },
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Maximum number of agent iterations before stopping"
                    }
                }

                // Require Confirmation
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Require Tool Confirmation"
                        }
                        Switch {
                            checked: config_signal.read().require_confirmation,
                            on_checked_change: move |checked| {
                                let mut config = config_signal.read().clone();
                                config.require_confirmation = checked;
                                config_signal.set(config.clone());
                                on_agent_config_change.call(config);
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Require user confirmation before executing tools"
                    }
                }

                // Tool Inspection
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable Tool Inspection"
                        }
                        Switch {
                            checked: config_signal.read().enable_tool_inspection,
                            on_checked_change: move |checked| {
                                let mut config = config_signal.read().clone();
                                config.enable_tool_inspection = checked;
                                config_signal.set(config.clone());
                                on_agent_config_change.call(config);
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Enable security and repetition inspection for tool calls"
                    }
                }

                // Auto Compact
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable Auto Compact"
                        }
                        Switch {
                            checked: config_signal.read().enable_auto_compact,
                            on_checked_change: move |checked| {
                                let mut config = config_signal.read().clone();
                                config.enable_auto_compact = checked;
                                config_signal.set(config.clone());
                                on_agent_config_change.call(config);
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Automatically compact conversation history when context limit is approached"
                    }
                }

                // Max Turns Without Tools
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Max Turns Without Tools"
                    }
                    Input {
                        value: config_signal.read().max_turns_without_tools.to_string(),
                        oninput: move |event: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                            if let Ok(turns) = event.value().parse::<usize>() {
                                let mut config = config_signal.read().clone();
                                config.max_turns_without_tools = turns;
                                config_signal.set(config.clone());
                                on_agent_config_change.call(config);
                            }
                        },
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Maximum number of consecutive turns without tool usage before stopping"
                    }
                }
            }
        }
    }
}

