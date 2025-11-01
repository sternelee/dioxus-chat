use crate::components::{
    button::{Button, ButtonVariant},
    dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger},
    input::Input,
    switch::Switch,
    separator::Separator,
    tabs::{Tabs, TabContent, TabList, TabTrigger},
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
    dialog::{Dialog, DialogContent, DialogHeader, DialogTitle},
};
use dioxus::prelude::*;
use api::{AgentConfig, GooseMode};

#[derive(Clone, PartialEq, Props)]
pub struct SettingsPanelProps {
    pub open: bool,
    pub on_open_change: EventHandler<bool>,
    pub active_tab: SettingsTab,
    pub on_tab_change: EventHandler<SettingsTab>,
    pub models: Vec<Model>,
    pub selected_model: Option<String>,
    pub on_select_model: EventHandler<String>,
    pub theme: Option<Theme>,
    pub on_theme_change: EventHandler<Theme>,
    pub agent_config: Option<AgentConfig>,
    pub on_agent_config_change: EventHandler<AgentConfig>,
    pub providers: Vec<Provider>,
    pub on_add_provider: EventHandler<Provider>,
    pub on_remove_provider: EventHandler<String>,
    pub on_update_provider: EventHandler<Provider>,
    pub shortcuts: Vec<Shortcut>,
    pub on_shortcut_change: EventHandler<(String, String)>,
    pub data_sources: Vec<DataSource>,
    pub on_add_data_source: EventHandler<DataSource>,
    pub on_remove_data_source: EventHandler<String>,
    pub performance_settings: PerformanceSettings,
    pub on_performance_change: EventHandler<PerformanceSettings>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SettingsTab {
    General,
    Appearance,
    Models,
    Agent,
    DataSources,
    Shortcuts,
    Performance,
    Advanced,
}

#[derive(Clone, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub context_limit: Option<usize>,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub pricing: Option<ModelPricing>,
}

#[derive(Clone, PartialEq)]
pub struct ModelPricing {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub currency: String,
}

#[derive(Clone, PartialEq)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<Model>,
    pub active: bool,
    pub rate_limit: Option<RateLimit>,
    pub custom_headers: std::collections::HashMap<String, String>,
}

#[derive(Clone, PartialEq)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
}

#[derive(Clone, PartialEq)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub type_: DataSourceType,
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub enabled: bool,
    pub last_sync: Option<String>,
    pub sync_frequency: SyncFrequency,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DataSourceType {
    LocalFile,
    GitHub,
    Confluence,
    Notion,
    WebScraping,
    Database,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SyncFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Manual,
}

#[derive(Clone, PartialEq)]
pub struct Shortcut {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_keys: Vec<String>,
    pub current_keys: Vec<String>,
    pub category: ShortcutCategory,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ShortcutCategory {
    Navigation,
    Editing,
    Chat,
    Window,
    System,
}

#[derive(Clone, PartialEq)]
pub struct PerformanceSettings {
    pub max_concurrent_requests: u32,
    pub cache_size_mb: u32,
    pub streaming_buffer_size: usize,
    pub enable_gpu_acceleration: bool,
    pub memory_limit_mb: u32,
    pub network_timeout_seconds: u64,
    pub enable_compression: bool,
    pub log_level: LogLevel,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Auto,
    Custom(String),
}

#[component]
pub fn SettingsPanelComplete(props: SettingsPanelProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| props.on_open_change.call(false),

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-6xl w-full mx-4 max-h-[95vh] flex flex-col",
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

                // Search bar
                div {
                    class: "px-6 py-3 border-b border-gray-200 dark:border-gray-700",
                    div {
                        class: "relative max-w-md",
                        Input {
                            placeholder: "Search settings...",
                            class: "w-full pl-10",
                        }
                        div {
                            class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400",
                            "🔍"
                        }
                    }
                }

                // Tabs
                div {
                    class: "px-6 pt-4 flex-1 overflow-hidden",
                    Tabs {
                        value: format!("{:?}", props.active_tab),
                        on_value_change: move |value| {
                            match value.as_str() {
                                "General" => props.on_tab_change.call(SettingsTab::General),
                                "Appearance" => props.on_tab_change.call(SettingsTab::Appearance),
                                "Models" => props.on_tab_change.call(SettingsTab::Models),
                                "Agent" => props.on_tab_change.call(SettingsTab::Agent),
                                "DataSources" => props.on_tab_change.call(SettingsTab::DataSources),
                                "Shortcuts" => props.on_tab_change.call(SettingsTab::Shortcuts),
                                "Performance" => props.on_tab_change.call(SettingsTab::Performance),
                                "Advanced" => props.on_tab_change.call(SettingsTab::Advanced),
                                _ => {}
                            }
                        },

                        TabList {
                            class: "grid w-full grid-cols-4 gap-2",
                            TabTrigger {
                                value: "General",
                                "General"
                            }
                            TabTrigger {
                                value: "Appearance",
                                "Appearance"
                            }
                            TabTrigger {
                                value: "Models",
                                "Models"
                            }
                            TabTrigger {
                                value: "Agent",
                                "Agent"
                            }
                            TabTrigger {
                                value: "DataSources",
                                "Data Sources"
                            }
                            TabTrigger {
                                value: "Shortcuts",
                                "Shortcuts"
                            }
                            TabTrigger {
                                value: "Performance",
                                "Performance"
                            }
                            TabTrigger {
                                value: "Advanced",
                                "Advanced"
                            }
                        }

                        // Tab Contents
                        div {
                            class: "overflow-y-auto max-h-[60vh]",

                            // General Tab
                            TabContent {
                                value: "General",
                                GeneralSettings {}
                            }

                            // Appearance Tab
                            TabContent {
                                value: "Appearance",
                                AppearanceSettings {
                                    theme: props.theme.clone(),
                                    on_theme_change: props.on_theme_change,
                                }
                            }

                            // Models Tab
                            TabContent {
                                value: "Models",
                                ModelsSettingsComplete {
                                    models: props.models.clone(),
                                    selected_model: props.selected_model.clone(),
                                    on_select_model: props.on_select_model,
                                    providers: props.providers.clone(),
                                    on_add_provider: props.on_add_provider,
                                    on_remove_provider: props.on_remove_provider,
                                    on_update_provider: props.on_update_provider,
                                }
                            }

                            // Agent Tab
                            TabContent {
                                value: "Agent",
                                AgentSettingsComplete {
                                    agent_config: props.agent_config.clone(),
                                    on_agent_config_change: props.on_agent_config_change,
                                }
                            }

                            // Data Sources Tab
                            TabContent {
                                value: "DataSources",
                                DataSourcesSettings {
                                    data_sources: props.data_sources.clone(),
                                    on_add_data_source: props.on_add_data_source,
                                    on_remove_data_source: props.on_remove_data_source,
                                }
                            }

                            // Shortcuts Tab
                            TabContent {
                                value: "Shortcuts",
                                ShortcutsSettings {
                                    shortcuts: props.shortcuts.clone(),
                                    on_shortcut_change: props.on_shortcut_change,
                                }
                            }

                            // Performance Tab
                            TabContent {
                                value: "Performance",
                                PerformanceSettings {
                                    settings: props.performance_settings.clone(),
                                    on_change: props.on_performance_change,
                                }
                            }

                            // Advanced Tab
                            TabContent {
                                value: "Advanced",
                                AdvancedSettings {}
                            }
                        }
                    }
                }

                // Footer
                div {
                    class: "px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-between items-center",
                    div {
                        class: "text-sm text-gray-500 dark:text-gray-400",
                        "Settings are automatically saved"
                    }
                    div {
                        class: "flex gap-2",
                        Button {
                            onclick: move |_| {
                                // TODO: Reset to defaults
                            },
                            variant: ButtonVariant::Ghost,
                            "Reset to Defaults"
                        }
                        Button {
                            onclick: move |_| {
                                // TODO: Export settings
                            },
                            variant: ButtonVariant::Ghost,
                            "Export"
                        }
                        Button {
                            onclick: move |_| {
                                // TODO: Import settings
                            },
                            variant: ButtonVariant::Ghost,
                            "Import"
                        }
                        Button {
                            onclick: move |_| props.on_open_change.call(false),
                            variant: ButtonVariant::Primary,
                            "Done"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GeneralSettings() -> Element {
    rsx! {
        div {
            class: "space-y-8 p-4",

            // Language & Region
            div {
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                    "Language & Region"
                }
                div {
                    class: "space-y-4 grid grid-cols-2 gap-4",
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Language"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            option {
                                value: "en",
                                "English"
                            }
                            option {
                                value: "zh",
                                "中文"
                            }
                            option {
                                value: "ja",
                                "日本語"
                            }
                            option {
                                value: "ko",
                                "한국어"
                            }
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Time Zone"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            option {
                                value: "utc",
                                "UTC"
                            }
                            option {
                                value: "local",
                                "Local Time"
                            }
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Date Format"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            option {
                                value: "mdy",
                                "MM/DD/YYYY"
                            }
                            option {
                                value: "dmy",
                                "DD/MM/YYYY"
                            }
                            option {
                                value: "ymd",
                                "YYYY-MM-DD"
                            }
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Number Format"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            option {
                                value: "en",
                                "1,234.56"
                            }
                            option {
                                value: "eu",
                                "1.234,56"
                            }
                        }
                    }
                }
            }

            Separator {}

            // Conversation
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Conversation"
            }
            div {
                class: "space-y-4 grid grid-cols-2 gap-4",
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Auto-save conversations"
                        }
                        Switch {
                            checked: true,
                            on_checked_change: move |_checked| {
                                // TODO: Implement auto-save setting
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Automatically save conversation history"
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable conversation export"
                        }
                        Switch {
                            checked: true,
                            on_checked_change: move |_checked| {
                                // TODO: Implement export setting
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Allow exporting conversations to different formats"
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Conversation history limit"
                        }
                    }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                        option {
                            value: "100",
                            "100 conversations"
                        }
                        option {
                            value: "500",
                            "500 conversations"
                        }
                        option {
                            value: "1000",
                            "1000 conversations"
                        }
                        option {
                            value: "unlimited",
                            "Unlimited"
                        }
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Default conversation name"
                        }
                    }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                        option {
                            value: "first_message",
                            "First message"
                        }
                        option {
                            value: "datetime",
                            "Date and time"
                        }
                        option {
                            value: "custom",
                            "Custom name"
                        }
                    }
                }
            }

            Separator {}

            // Notifications
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Notifications"
            }
            div {
                class: "space-y-4",
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Desktop notifications"
                        }
                        Switch {
                            checked: true,
                            on_checked_change: move |_checked| {
                                // TODO: Implement notifications setting
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Show desktop notifications for new messages"
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Sound notifications"
                        }
                        Switch {
                            checked: false,
                            on_checked_change: move |_checked| {
                                // TODO: Implement sound notifications
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Play sound for new messages"
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Message completion notifications"
                        }
                        Switch {
                            checked: true,
                            on_checked_change: move |_checked| {
                                // TODO: Implement completion notifications
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Notify when responses are completed"
                    }
                }
            }
        }
    }
}

#[component]
fn AppearanceSettings(
    theme: Option<Theme>,
    on_theme_change: EventHandler<Theme>,
) -> Element {
    rsx! {
        div {
            class: "space-y-8 p-4",

            // Theme Selection
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Theme"
            }
            div {
                class: "grid grid-cols-4 gap-4",
                Button {
                    onclick: move |_| on_theme_change.call(Theme::Light),
                    variant: if matches!(theme, Some(Theme::Light)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: "flex flex-col items-center gap-2 h-24",
                    div {
                        class: "text-3xl",
                        "☀️"
                    }
                    span {
                        class: "text-sm",
                        "Light"
                    }
                }
                Button {
                    onclick: move |_| on_theme_change.call(Theme::Dark),
                    variant: if matches!(theme, Some(Theme::Dark)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: "flex flex-col items-center gap-2 h-24",
                    div {
                        class: "text-3xl",
                        "🌙"
                    }
                    span {
                        class: "text-sm",
                        "Dark"
                    }
                }
                Button {
                    onclick: move |_| on_theme_change.call(Theme::Auto),
                    variant: if matches!(theme, Some(Theme::Auto)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: "flex flex-col items-center gap-2 h-24",
                    div {
                        class: "text-3xl",
                        "🔄"
                    }
                    span {
                        class: "text-sm",
                        "Auto"
                    }
                }
                Button {
                    onclick: move |_| on_theme_change.call(Theme::Custom("high_contrast".to_string())),
                    variant: if matches!(theme, Some(Theme::Custom(_))) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: "flex flex-col items-center gap-2 h-24",
                    div {
                        class: "text-3xl",
                        "🎨"
                    }
                    span {
                        class: "text-sm",
                        "Custom"
                    }
                }
            }

            Separator {}

            // Chat Appearance
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Chat Appearance"
            }
            div {
                class: "space-y-4 grid grid-cols-2 gap-4",
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Font Size"
                    }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                        option {
                            value: "small",
                            "Small"
                        }
                        option {
                            value: "medium",
                            "Medium"
                        }
                        option {
                            value: "large",
                            "Large"
                        }
                        option {
                            value: "extra_large",
                            "Extra Large"
                        }
                    }
                }
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Font Family"
                    }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                        option {
                            value: "system",
                            "System Default"
                        }
                        option {
                            value: "sans",
                            "Sans Serif"
                        }
                        option {
                            value: "serif",
                            "Serif"
                        }
                        option {
                            value: "mono",
                            "Monospace"
                        }
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Show timestamps"
                        }
                        Switch {
                            checked: true,
                            on_checked_change: move |_checked| {
                                // TODO: Implement timestamps setting
                            },
                        }
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Show avatars"
                        }
                        Switch {
                            checked: true,
                            on_checked_change: move |_checked| {
                                // TODO: Implement avatars setting
                            },
                        }
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Compact mode"
                        }
                        Switch {
                            checked: false,
                            on_checked_change: move |_checked| {
                                // TODO: Implement compact mode
                            },
                        }
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Code syntax highlighting"
                        }
                        Switch {
                            checked: true,
                            on_checked_change: move |_checked| {
                                // TODO: Implement syntax highlighting
                            },
                        }
                    }
                }
            }

            Separator {}

            // Colors
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Colors"
            }
            div {
                class: "space-y-4 grid grid-cols-2 gap-4",
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Accent Color"
                    }
                    div {
                        class: "flex items-center gap-2",
                        input {
                            r#type: "color",
                            class: "w-8 h-8 border border-gray-300 rounded",
                            value: "#3B82F6",
                        }
                        Input {
                            value: "#3B82F6",
                            class: "flex-1",
                        }
                    }
                }
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Background Color"
                    }
                    div {
                        class: "flex items-center gap-2",
                        input {
                            r#type: "color",
                            class: "w-8 h-8 border border-gray-300 rounded",
                            value: "#FFFFFF",
                        }
                        Input {
                            value: "#FFFFFF",
                            class: "flex-1",
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ModelsSettingsComplete(
    models: Vec<Model>,
    selected_model: Option<String>,
    on_select_model: EventHandler<String>,
    providers: Vec<Provider>,
    on_add_provider: EventHandler<Provider>,
    on_remove_provider: EventHandler<String>,
    on_update_provider: EventHandler<Provider>,
) -> Element {
    let show_add_provider: Signal<bool> = use_signal(|| false);

    rsx! {
        div {
            class: "space-y-6 p-4",

            // Active Model
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Active Model"
            }
            div {
                class: "space-y-4",
                select {
                    class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                    onchange: move |evt| on_select_model.call(evt.value()),
                    option {
                        value: "",
                        selected: selected_model.is_none(),
                        disabled: true,
                        "Select a model"
                    }
                    for model in &models {
                        option {
                            value: model.id.clone(),
                            selected: selected_model.as_ref() == Some(&model.id),
                            "{model.name} - {model.provider}"
                        }
                    }
                }

                if let Some(model_id) = &selected_model {
                    if let Some(model) = models.iter().find(|m| &m.id == model_id) {
                        div {
                            class: "bg-gray-50 dark:bg-gray-900 rounded-lg p-4",
                            h4 {
                                class: "font-medium text-gray-900 dark:text-gray-100 mb-2",
                                "{model.name}"
                            }
                            if let Some(description) = &model.description {
                                p {
                                    class: "text-sm text-gray-600 dark:text-gray-400 mb-3",
                                    "{description}"
                                }
                            }
                            div {
                                class: "grid grid-cols-3 gap-4 text-sm mb-3",
                                if let Some(context_limit) = model.context_limit {
                                    div {
                                        span {
                                            class: "text-gray-500 dark:text-gray-400",
                                            "Context: "
                                        }
                                        span {
                                            class: "text-gray-900 dark:text-gray-100",
                                            "{context_limit} tokens"
                                        }
                                    }
                                }
                                div {
                                    span {
                                        class: "text-gray-500 dark:text-gray-400",
                                        "Tools: "
                                    }
                                    span {
                                        class: "text-gray-900 dark:text-gray-100",
                                        "{if model.supports_tools { 'Yes' } else { 'No' }}"
                                    }
                                }
                                div {
                                    span {
                                        class: "text-gray-500 dark:text-gray-400",
                                        "Streaming: "
                                    }
                                    span {
                                        class: "text-gray-900 dark:text-gray-100",
                                        "{if model.supports_streaming { 'Yes' } else { 'No' }}"
                                    }
                                }
                                div {
                                    span {
                                        class: "text-gray-500 dark:text-gray-400",
                                        "Vision: "
                                    }
                                    span {
                                        class: "text-gray-900 dark:text-gray-100",
                                        "{if model.supports_vision { 'Yes' } else { 'No' }}"
                                    }
                                }
                                div {
                                    span {
                                        class: "text-gray-500 dark:text-gray-400",
                                        "Function Calling: "
                                    }
                                    span {
                                        class: "text-gray-900 dark:text-gray-100",
                                        "{if model.supports_function_calling { 'Yes' } else { 'No' }}"
                                    }
                                }
                            }
                            if let Some(pricing) = &model.pricing {
                                div {
                                    class: "text-xs text-gray-500 dark:text-gray-400",
                                    "Pricing: ${pricing.input_tokens}/1K input tokens, ${pricing.output_tokens}/1K output tokens ({pricing.currency})"
                                }
                            }
                        }
                    }
                }
            }

            Separator {}

            // Providers
            div {
                class: "flex items-center justify-between mb-4",
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100",
                    "Providers"
                }
                Button {
                    onclick: move |_| show_add_provider.set(true),
                    variant: ButtonVariant::Primary,
                    "+ Add Provider"
                }
            }
            div {
                class: "space-y-3",
                {providers.iter().map(|provider| {
                    rsx! {
                        ProviderItemComplete {
                            provider: provider.clone(),
                            on_remove: props.on_remove_provider,
                            on_update: props.on_update_provider,
                        }
                    }
                })}
            }

            // Add Provider Dialog
            if *show_add_provider.read() {
                AddProviderDialog {
                    open: true,
                    on_open_change: move |open| show_add_provider.set(open),
                    on_add_provider: props.on_add_provider,
                }
            }
        }
    }
}

#[component]
fn ProviderItemComplete(
    provider: Provider,
    on_remove: EventHandler<String>,
    on_update: EventHandler<Provider>,
) -> Element {
    let show_edit: Signal<bool> = use_signal(|| false);

    rsx! {
        div {
            class: "border border-gray-200 dark:border-gray-700 rounded-lg p-4",
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
                            "{provider.models.len()} models • {if provider.active { 'Active' } else { 'Inactive' }}"
                        }
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    div {
                        class: "w-2 h-2 rounded-full",
                        class: if provider.active { "bg-green-500" } else { "bg-gray-400" }
                    }
                    Button {
                        onclick: move |_| show_edit.set(true),
                        variant: ButtonVariant::Ghost,
                        class: "text-sm",
                        "Edit"
                    }
                    Button {
                        onclick: move |_| on_remove.call(provider.id.clone()),
                        variant: ButtonVariant::Ghost,
                        class: "text-sm text-red-500",
                        "Remove"
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
            div {
                class: "mt-3",
                p {
                    class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                    "Available Models:"
                }
                div {
                    class: "grid grid-cols-2 gap-2",
                    {provider.models.iter().take(6).map(|model| {
                        rsx! {
                            div {
                                class: "text-xs bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded",
                                "{model.name}"
                            }
                        }
                    })}
                    if provider.models.len() > 6 {
                        div {
                            class: "text-xs bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded",
                            "+{provider.models.len() - 6} more"
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
                on_update: on_update,
            }
        }
    }
}

#[component]
fn AddProviderDialog(
    open: bool,
    on_open_change: EventHandler<bool>,
    on_add_provider: EventHandler<Provider>,
) -> Element {
    let mut name: Signal<String> = use_signal(|| String::new());
    let mut base_url: Signal<String> = use_signal(|| String::new());
    let mut api_key: Signal<String> = use_signal(|| String::new());

    rsx! {
        Dialog {
            open,
            on_open_change,
            DialogContent {
                class: "max-w-md",
                DialogHeader {
                    DialogTitle {
                        "Add Provider"
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
                            oninput: move |evt| name.set(evt.value()),
                            placeholder: "e.g., OpenAI, Anthropic",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Base URL (Optional)"
                        }
                        Input {
                            value: base_url.read().clone(),
                            oninput: move |evt| base_url.set(evt.value()),
                            placeholder: "https://api.openai.com/v1",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "API Key (Optional)"
                        }
                        Input {
                            r#type: "password",
                            value: api_key.read().clone(),
                            oninput: move |evt| api_key.set(evt.value()),
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
                                on_add_provider.call(Provider {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: name.read().clone(),
                                    base_url: Some(base_url.read().clone()),
                                    api_key: Some(api_key.read().clone()),
                                    models: vec![],
                                    active: true,
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
    provider: Provider,
    on_update: EventHandler<Provider>,
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
                        "Edit Provider"
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
                            oninput: move |evt| name.set(evt.value()),
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Base URL"
                        }
                        Input {
                            value: base_url.read().clone(),
                            oninput: move |evt| base_url.set(evt.value()),
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
                            oninput: move |evt| api_key.set(evt.value()),
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
                            updated_provider.base_url = Some(base_url.read().clone());
                            updated_provider.api_key = Some(api_key.read().clone());
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
fn AgentSettingsComplete(
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
                        oninput: move |event| {
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

                // Compact Threshold
                if config_signal.read().enable_auto_compact {
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Compact Threshold (%)"
                        }
                        Input {
                            value: (config_signal.read().compact_threshold * 100.0).to_string(),
                            oninput: move |event| {
                                if let Ok(threshold) = event.value().parse::<f32>() {
                                    let mut config = config_signal.read().clone();
                                    config.compact_threshold = threshold / 100.0;
                                    config_signal.set(config.clone());
                                    on_agent_config_change.call(config);
                                }
                            },
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                        }
                        p {
                            class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                            "Percentage of context limit at which to trigger compaction"
                        }
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
                        oninput: move |event| {
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

                // Enable Extensions
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable Extensions"
                        }
                        Switch {
                            checked: config_signal.read().enable_extensions,
                            on_checked_change: move |checked| {
                                let mut config = config_signal.read().clone();
                                config.enable_extensions = checked;
                                config_signal.set(config.clone());
                                on_agent_config_change.call(config);
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Enable agent extensions and plugins"
                    }
                }

                // Extension Timeout
                if config_signal.read().enable_extensions {
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Extension Timeout (seconds)"
                        }
                        Input {
                            value: config_signal.read().extension_timeout.to_string(),
                            oninput: move |event| {
                                if let Ok(timeout) = event.value().parse::<u64>() {
                                    let mut config = config_signal.read().clone();
                                    config.extension_timeout = timeout;
                                    config_signal.set(config.clone());
                                    on_agent_config_change.call(config);
                                }
                            },
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                        }
                        p {
                            class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                            "Maximum time to wait for extension responses"
                        }
                    }
                }

                // Enable Autopilot
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable Autopilot"
                        }
                        Switch {
                            checked: config_signal.read().enable_autopilot,
                            on_checked_change: move |checked| {
                                let mut config = config_signal.read().clone();
                                config.enable_autopilot = checked;
                                config_signal.set(config.clone());
                                on_agent_config_change.call(config);
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Allow agent to operate autonomously without user intervention"
                    }
                }
            }
        }
    }
}

#[component]
fn DataSourcesSettings(
    data_sources: Vec<DataSource>,
    on_add_data_source: EventHandler<DataSource>,
    on_remove_data_source: EventHandler<String>,
) -> Element {
    let mut show_add_source: Signal<bool> = use_signal(|| false);

    rsx! {
        div {
            class: "space-y-6 p-4",

            // Data Sources Header
            div {
                class: "flex items-center justify-between mb-4",
                h3 {
                    class: "text-lg font-medium text-gray-900 dark:text-gray-100",
                    "Data Sources"
                }
                Button {
                    onclick: move |_| show_add_source.set(true),
                    variant: ButtonVariant::Primary,
                    "+ Add Data Source"
                }
            }

            // Data Sources List
            div {
                class: "space-y-3",
                if data_sources.is_empty() {
                    div {
                        class: "text-center py-8 text-gray-500 dark:text-gray-400",
                        div {
                            class: "text-4xl mb-2",
                            "📊"
                        }
                        p {
                            class: "text-sm",
                            "No data sources configured"
                        }
                        p {
                            class: "text-xs mt-1",
                            "Add data sources to give your agent access to external information"
                        }
                    }
                } else {
                    {data_sources.iter().map(|source| {
                        rsx! {
                            DataSourceItem {
                                source: source.clone(),
                                on_remove: on_remove_data_source,
                            }
                        }
                    })}
                }
            }

            // Add Data Source Dialog
            if *show_add_source.read() {
                AddDataSourceDialog {
                    open: true,
                    on_open_change: move |open| show_add_source.set(open),
                    on_add_data_source,
                }
            }
        }
    }
}

#[component]
fn DataSourceItem(
    source: DataSource,
    on_remove: EventHandler<String>,
) -> Element {
    let type_icon = match source.type_ {
        DataSourceType::LocalFile => "📁",
        DataSourceType::GitHub => "🐙",
        DataSourceType::Confluence => "📄",
        DataSourceType::Notion => "📝",
        DataSourceType::WebScraping => "🌐",
        DataSourceType::Database => "🗄️",
    };

    rsx! {
        div {
            class: "border border-gray-200 dark:border-gray-700 rounded-lg p-4",
            div {
                class: "flex items-center justify-between mb-3",
                div {
                    class: "flex items-center gap-3",
                    div {
                        class: "text-2xl",
                        "{type_icon}"
                    }
                    div {
                        h4 {
                            class: "font-medium text-gray-900 dark:text-gray-100",
                            "{source.name}"
                        }
                        p {
                            class: "text-sm text-gray-500 dark:text-gray-400",
                            "{source.type_:?} • {if source.enabled { 'Active' } else { 'Inactive' }}"
                        }
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    div {
                        class: "w-2 h-2 rounded-full",
                        class: if source.enabled { "bg-green-500" } else { "bg-gray-400" }
                    }
                    Button {
                        onclick: move |_| on_remove.call(source.id.clone()),
                        variant: ButtonVariant::Ghost,
                        class: "text-sm text-red-500",
                        "Remove"
                    }
                }
            }

            // Source details
            if let Some(url) = &source.url {
                div {
                    class: "text-sm text-gray-600 dark:text-gray-400 mb-2",
                    "URL: {url}"
                }
            }

            if let Some(last_sync) = &source.last_sync {
                div {
                    class: "text-sm text-gray-600 dark:text-gray-400 mb-2",
                    "Last sync: {last_sync}"
                }
            }

            div {
                class: "text-sm text-gray-600 dark:text-gray-400",
                "Sync frequency: {source.sync_frequency:?}"
            }
        }
    }
}

#[component]
fn AddDataSourceDialog(
    open: bool,
    on_open_change: EventHandler<bool>,
    on_add_data_source: EventHandler<DataSource>,
) -> Element {
    let name: Signal<String> = use_signal(|| String::new());
    let source_type: Signal<DataSourceType> = use_signal(|| DataSourceType::LocalFile);
    let url: Signal<String> = use_signal(|| String::new());
    let api_key: Signal<String> = use_signal(|| String::new());
    let sync_frequency: Signal<SyncFrequency> = use_signal(|| SyncFrequency::Manual);

    rsx! {
        Dialog {
            open,
            on_open_change,
            DialogContent {
                class: "max-w-md",
                DialogHeader {
                    DialogTitle {
                        "Add Data Source"
                    }
                }
                div {
                    class: "space-y-4",
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Source Name"
                        }
                        Input {
                            value: name.read().clone(),
                            oninput: move |evt| name.set(evt.value()),
                            placeholder: "e.g., My Documents",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Source Type"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            onchange: move |evt| {
                                source_type.set(match evt.value().as_str() {
                                    "LocalFile" => DataSourceType::LocalFile,
                                    "GitHub" => DataSourceType::GitHub,
                                    "Confluence" => DataSourceType::Confluence,
                                    "Notion" => DataSourceType::Notion,
                                    "WebScraping" => DataSourceType::WebScraping,
                                    "Database" => DataSourceType::Database,
                                    _ => DataSourceType::LocalFile,
                                });
                            },
                            option {
                                value: "LocalFile",
                                selected: matches!(*source_type.read(), DataSourceType::LocalFile),
                                "Local Files"
                            }
                            option {
                                value: "GitHub",
                                selected: matches!(*source_type.read(), DataSourceType::GitHub),
                                "GitHub Repository"
                            }
                            option {
                                value: "Confluence",
                                selected: matches!(*source_type.read(), DataSourceType::Confluence),
                                "Confluence"
                            }
                            option {
                                value: "Notion",
                                selected: matches!(*source_type.read(), DataSourceType::Notion),
                                "Notion"
                            }
                            option {
                                value: "WebScraping",
                                selected: matches!(*source_type.read(), DataSourceType::WebScraping),
                                "Web Scraping"
                            }
                            option {
                                value: "Database",
                                selected: matches!(*source_type.read(), DataSourceType::Database),
                                "Database"
                            }
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "URL / Path"
                        }
                        Input {
                            value: url.read().clone(),
                            oninput: move |evt| url.set(evt.value()),
                            placeholder: "https://github.com/user/repo or /path/to/files",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "API Key (Optional)"
                        }
                        Input {
                            r#type: "password",
                            value: api_key.read().clone(),
                            oninput: move |evt| api_key.set(evt.value()),
                            placeholder: "Enter API key if required",
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                            "Sync Frequency"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                            onchange: move |evt| {
                                sync_frequency.set(match evt.value().as_str() {
                                    "RealTime" => SyncFrequency::RealTime,
                                    "Hourly" => SyncFrequency::Hourly,
                                    "Daily" => SyncFrequency::Daily,
                                    "Weekly" => SyncFrequency::Weekly,
                                    "Manual" => SyncFrequency::Manual,
                                    _ => SyncFrequency::Manual,
                                });
                            },
                            option {
                                value: "Manual",
                                selected: matches!(*sync_frequency.read(), SyncFrequency::Manual),
                                "Manual"
                            }
                            option {
                                value: "Hourly",
                                selected: matches!(*sync_frequency.read(), SyncFrequency::Hourly),
                                "Hourly"
                            }
                            option {
                                value: "Daily",
                                selected: matches!(*sync_frequency.read(), SyncFrequency::Daily),
                                "Daily"
                            }
                            option {
                                value: "Weekly",
                                selected: matches!(*sync_frequency.read(), SyncFrequency::Weekly),
                                "Weekly"
                            }
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
                                on_add_data_source.call(DataSource {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: name.read().clone(),
                                    type_: source_type.read().clone(),
                                    url: Some(url.read().clone()),
                                    api_key: Some(api_key.read().clone()),
                                    enabled: true,
                                    last_sync: None,
                                    sync_frequency: sync_frequency.read().clone(),
                                });
                                name.set(String::new());
                                url.set(String::new());
                                api_key.set(String::new());
                                on_open_change.call(false);
                            }
                        },
                        variant: ButtonVariant::Primary,
                        "Add Data Source"
                    }
                }
            }
        }
    }
}

#[component]
fn ShortcutsSettings(
    shortcuts: Vec<Shortcut>,
    on_shortcut_change: EventHandler<(String, String)>,
) -> Element {
    rsx! {
        div {
            class: "space-y-6 p-4",

            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Keyboard Shortcuts"
            }

            // Shortcuts by category
            for category in [ShortcutCategory::Navigation, ShortcutCategory::Editing, ShortcutCategory::Chat, ShortcutCategory::Window, ShortcutCategory::System] {
                div {
                    h4 {
                        class: "text-md font-medium text-gray-800 dark:text-gray-200 mb-3",
                        "{category:?}"
                    }
                    div {
                        class: "space-y-2",
                        {shortcuts.iter()
                            .filter(|s| s.category == category)
                            .map(|shortcut| {
                                rsx! {
                                    ShortcutItem {
                                        shortcut: shortcut.clone(),
                                        on_change: on_shortcut_change,
                                    }
                                }
                            })}
                    }
                }
            }
        }
    }
}

#[component]
fn ShortcutItem(
    shortcut: Shortcut,
    on_change: EventHandler<(String, String)>,
) -> Element {
    let is_editing: Signal<bool> = use_signal(|| false);
    let current_keys: Signal<String> = use_signal(|| shortcut.current_keys.join(", "));

    rsx! {
        div {
            class: "flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg",
            div {
                class: "flex-1",
                div {
                    class: "font-medium text-gray-900 dark:text-gray-100",
                    "{shortcut.name}"
                }
                div {
                    class: "text-sm text-gray-500 dark:text-gray-400",
                    "{shortcut.description}"
                }
            }
            div {
                class: "flex items-center gap-2",
                if *is_editing.read() {
                    Input {
                        value: current_keys.read().clone(),
                        oninput: move |evt: dioxus::prelude::Event<FormEvent>| current_keys.set(evt.value()),
                        class: "w-32 px-2 py-1 text-sm",
                        placeholder: "e.g., Ctrl+N",
                    }
                    Button {
                        onclick: move |_| {
                            on_change.call((shortcut.id.clone(), current_keys.read().clone()));
                            is_editing.set(false);
                        },
                        variant: ButtonVariant::Primary,
                        class: "text-xs px-2 py-1",
                        "Save"
                    }
                    Button {
                        onclick: move |_| is_editing.set(false),
                        variant: ButtonVariant::Ghost,
                        class: "text-xs px-2 py-1",
                        "Cancel"
                    }
                } else {
                    div {
                        class: "bg-gray-100 dark:bg-gray-700 px-3 py-1 rounded text-sm font-mono",
                        "{shortcut.current_keys.join(\", \")}"
                    }
                    Button {
                        onclick: move |_| is_editing.set(true),
                        variant: ButtonVariant::Ghost,
                        class: "text-xs px-2 py-1",
                        "Edit"
                    }
                }
            }
        }
    }
}

#[component]
fn PerformanceSettings(
    settings: PerformanceSettings,
    on_change: EventHandler<PerformanceSettings>,
) -> Element {
    let mut settings_signal: Signal<PerformanceSettings> = use_signal(|| settings);

    rsx! {
        div {
            class: "space-y-6 p-4",

            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Performance Settings"
            }

            div {
                class: "space-y-4 grid grid-cols-2 gap-4",
                // Max Concurrent Requests
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Max Concurrent Requests"
                    }
                    Input {
                        value: settings_signal.read().max_concurrent_requests.to_string(),
                        oninput: move |evt: dioxus::prelude::Event<FormEvent>| {
                            if let Ok(value) = evt.value().parse::<u32>() {
                                let mut settings = settings_signal.read().clone();
                                settings.max_concurrent_requests = value;
                                settings_signal.set(settings.clone());
                                on_change.call(settings);
                            }
                        },
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                    }
                }

                // Cache Size
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Cache Size (MB)"
                    }
                    Input {
                        value: settings_signal.read().cache_size_mb.to_string(),
                        oninput: move |evt| {
                            if let Ok(value) = evt.value().parse::<u32>() {
                                let mut settings = settings_signal.read().clone();
                                settings.cache_size_mb = value;
                                settings_signal.set(settings.clone());
                                on_change.call(settings);
                            }
                        },
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                    }
                }

                // Streaming Buffer Size
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Streaming Buffer Size"
                    }
                    Input {
                        value: settings_signal.read().streaming_buffer_size.to_string(),
                        oninput: move |evt| {
                            if let Ok(value) = evt.value().parse::<usize>() {
                                let mut settings = settings_signal.read().clone();
                                settings.streaming_buffer_size = value;
                                settings_signal.set(settings.clone());
                                on_change.call(settings);
                            }
                        },
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                    }
                }

                // Memory Limit
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Memory Limit (MB)"
                    }
                    Input {
                        value: settings_signal.read().memory_limit_mb.to_string(),
                        oninput: move |evt| {
                            if let Ok(value) = evt.value().parse::<u32>() {
                                let mut settings = settings_signal.read().clone();
                                settings.memory_limit_mb = value;
                                settings_signal.set(settings.clone());
                                on_change.call(settings);
                            }
                        },
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                    }
                }

                // Network Timeout
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Network Timeout (seconds)"
                    }
                    Input {
                        value: settings_signal.read().network_timeout_seconds.to_string(),
                        oninput: move |evt| {
                            if let Ok(value) = evt.value().parse::<u64>() {
                                let mut settings = settings_signal.read().clone();
                                settings.network_timeout_seconds = value;
                                settings_signal.set(settings.clone());
                                on_change.call(settings);
                            }
                        },
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800"
                    }
                }

                // Log Level
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Log Level"
                    }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800",
                        onchange: move |evt| {
                            let mut settings = settings_signal.read().clone();
                            settings.log_level = match evt.value().as_str() {
                                "Error" => LogLevel::Error,
                                "Warn" => LogLevel::Warn,
                                "Info" => LogLevel::Info,
                                "Debug" => LogLevel::Debug,
                                "Trace" => LogLevel::Trace,
                                _ => LogLevel::Info,
                            };
                            settings_signal.set(settings.clone());
                            on_change.call(settings);
                        },
                        option {
                            value: "Error",
                            selected: matches!(settings_signal.read().log_level, LogLevel::Error),
                            "Error"
                        }
                        option {
                            value: "Warn",
                            selected: matches!(settings_signal.read().log_level, LogLevel::Warn),
                            "Warning"
                        }
                        option {
                            value: "Info",
                            selected: matches!(settings_signal.read().log_level, LogLevel::Info),
                            "Info"
                        }
                        option {
                            value: "Debug",
                            selected: matches!(settings_signal.read().log_level, LogLevel::Debug),
                            "Debug"
                        }
                        option {
                            value: "Trace",
                            selected: matches!(settings_signal.read().log_level, LogLevel::Trace),
                            "Trace"
                        }
                    }
                }
            }

            // Boolean settings
            div {
                class: "space-y-4",
                // GPU Acceleration
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable GPU Acceleration"
                        }
                        Switch {
                            checked: settings_signal.read().enable_gpu_acceleration,
                            on_checked_change: move |checked| {
                                let mut settings = settings_signal.read().clone();
                                settings.enable_gpu_acceleration = checked;
                                settings_signal.set(settings.clone());
                                on_change.call(settings);
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Use GPU acceleration for supported operations"
                    }
                }

                // Compression
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable Compression"
                        }
                        Switch {
                            checked: settings_signal.read().enable_compression,
                            on_checked_change: move |checked| {
                                let mut settings = settings_signal.read().clone();
                                settings.enable_compression = checked;
                                settings_signal.set(settings.clone());
                                on_change.call(settings);
                            },
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Compress network requests to reduce bandwidth usage"
                    }
                }
            }
        }
    }
}

#[component]
fn AdvancedSettings() -> Element {
    rsx! {
        div {
            class: "space-y-6 p-4",

            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Advanced Settings"
            }

            // Developer Settings
            div {
                h4 {
                    class: "text-md font-medium text-gray-800 dark:text-gray-200 mb-3",
                    "Developer Options"
                }
                div {
                    class: "space-y-4",
                    div {
                        div {
                            class: "flex items-center justify-between",
                            label {
                                class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                "Enable Developer Mode"
                            }
                            Switch {
                                checked: false,
                                on_checked_change: move |_checked| {
                                    // TODO: Implement developer mode
                                },
                            }
                        }
                        p {
                            class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                            "Enable advanced developer features and debugging tools"
                        }
                    }
                    div {
                        div {
                            class: "flex items-center justify-between",
                            label {
                                class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                "Show Debug Information"
                            }
                            Switch {
                                checked: false,
                                on_checked_change: move |_checked| {
                                    // TODO: Implement debug info
                                },
                            }
                        }
                        p {
                            class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                            "Display detailed debug information in the console"
                        }
                    }
                }
            }

            Separator {}

            // Data Management
            h4 {
                class: "text-md font-medium text-gray-800 dark:text-gray-200 mb-3",
                "Data Management"
            }
            div {
                class: "space-y-4",
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Clear Cache"
                        }
                        Button {
                            onclick: move |_| {
                                // TODO: Implement cache clearing
                            },
                            variant: ButtonVariant::Ghost,
                            "Clear"
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Clear all cached data"
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Reset All Settings"
                        }
                        Button {
                            onclick: move |_| {
                                // TODO: Implement settings reset
                            },
                            variant: ButtonVariant::Ghost,
                            class: "text-red-500",
                            "Reset"
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Reset all settings to default values"
                    }
                }
                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Export All Data"
                        }
                        Button {
                            onclick: move |_| {
                                // TODO: Implement data export
                            },
                            variant: ButtonVariant::Ghost,
                            "Export"
                        }
                    }
                    p {
                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        "Export all conversations, settings, and data"
                    }
                }
            }

            Separator {}

            // About
            h4 {
                class: "text-md font-medium text-gray-800 dark:text-gray-200 mb-3",
                "About"
            }
            div {
                class: "bg-gray-50 dark:bg-gray-900 rounded-lg p-4",
                div {
                    class: "space-y-2 text-sm",
                    div {
                        span {
                            class: "font-medium text-gray-700 dark:text-gray-300",
                            "Version: "
                        }
                        span {
                            class: "text-gray-600 dark:text-gray-400",
                            "1.0.0"
                        }
                    }
                    div {
                        span {
                            class: "font-medium text-gray-700 dark:text-gray-300",
                            "Build: "
                        }
                        span {
                            class: "text-gray-600 dark:text-gray-400",
                            "2024.01.15"
                        }
                    }
                    div {
                        span {
                            class: "font-medium text-gray-700 dark:text-gray-300",
                            "Data Directory: "
                        }
                        span {
                            class: "text-gray-600 dark:text-gray-400",
                            "~/.dioxus-chat"
                        }
                    }
                    div {
                        span {
                            class: "font-medium text-gray-700 dark:text-gray-300",
                            "License: "
                        }
                        span {
                            class: "text-gray-600 dark:text-gray-400",
                            "MIT"
                        }
                    }
                }
            }
        }
    }
}