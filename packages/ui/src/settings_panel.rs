use crate::components::{
    button::{Button, ButtonVariant},
    dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger},
    input::Input,
    switch::Switch,
      separator::Separator,
    tabs::{Tabs, TabContent, TabList, TabTrigger},
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
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
}

#[derive(Clone, PartialEq, Debug)]
pub enum SettingsTab {
    General,
    Appearance,
    Models,
    Agent,
    DataSources,
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
}

#[derive(Clone, PartialEq)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<Model>,
    pub active: bool,
}

#[derive(Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[component]
pub fn SettingsPanel(props: SettingsPanelProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| props.on_open_change.call(false),

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col",
                onclick: move |e: dioxus::prelude::Event<MouseData>| e.stop_propagation(),

                // Header
                div {
                    class: "px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between",
                    h2 {
                        class: "text-xl font-semibold text-gray-900 dark:text-gray-100",
                        "Settings"
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
                    class: "px-6 pt-4",
                    Tabs {
                        value: format!("{:?}", props.active_tab),
                        on_value_change: move |value| {
                            match value.as_str() {
                                "General" => props.on_tab_change.call(SettingsTab::General),
                                "Appearance" => props.on_tab_change.call(SettingsTab::Appearance),
                                "Models" => props.on_tab_change.call(SettingsTab::Models),
                                "Agent" => props.on_tab_change.call(SettingsTab::Agent),
                                "DataSources" => props.on_tab_change.call(SettingsTab::DataSources),
                                "Advanced" => props.on_tab_change.call(SettingsTab::Advanced),
                                _ => {}
                            }
                        },

                        TabList {
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
                                value: "Advanced",
                                "Advanced"
                            }
                        }

                        // General Tab
                        TabContent {
                            value: "General",
                            GeneralSettings {
                                agent_config: props.agent_config.clone(),
                                on_agent_config_change: props.on_agent_config_change,
                            }
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
                            ModelsSettings {
                                models: props.models.clone(),
                                selected_model: props.selected_model.clone(),
                                on_select_model: props.on_select_model,
                                providers: props.providers.clone(),
                                on_add_provider: props.on_add_provider,
                                on_remove_provider: props.on_remove_provider,
                            }
                        }

                        // Agent Tab
                        TabContent {
                            value: "Agent",
                            AgentSettings {
                                agent_config: props.agent_config.clone(),
                                on_agent_config_change: props.on_agent_config_change,
                            }
                        }

                        // Data Sources Tab
                        TabContent {
                            value: "DataSources",
                            DataSourcesSettings {}
                        }

                        // Advanced Tab
                        TabContent {
                            value: "Advanced",
                            AdvancedSettings {}
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
                        "Save Changes"
                    }
                }
            }
        }
    }
}

#[component]
fn GeneralSettings(
    agent_config: Option<AgentConfig>,
    on_agent_config_change: EventHandler<AgentConfig>,
) -> Element {
    let config = agent_config.unwrap_or_default();

    rsx! {
        div {
            class: "space-y-6",

            // Language
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
                        Select {
                            SelectTrigger {
                                SelectValue {
                                    placeholder: "Select language"
                                }
                            }
                            SelectContent {
                                SelectItem {
                                    value: "en",
                                    "English"
                                }
                                SelectItem {
                                    value: "zh",
                                    "中文"
                                }
                                SelectItem {
                                    value: "ja",
                                    "日本語"
                                }
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
                class: "space-y-4",
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
            class: "space-y-6",

            // Theme
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Theme"
            }
            div {
                class: "grid grid-cols-3 gap-4",
                Button {
                    onclick: move |_| on_theme_change.call(Theme::Light),
                    variant: if matches!(theme, Some(Theme::Light)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: "flex flex-col items-center gap-2 h-20",
                    div {
                        class: "text-2xl",
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
                    class: "flex flex-col items-center gap-2 h-20",
                    div {
                        class: "text-2xl",
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
                    class: "flex flex-col items-center gap-2 h-20",
                    div {
                        class: "text-2xl",
                        "🔄"
                    }
                    span {
                        class: "text-sm",
                        "Auto"
                    }
                }
            }
        }
    }
}

#[component]
fn ModelsSettings(
    models: Vec<Model>,
    selected_model: Option<String>,
    on_select_model: EventHandler<String>,
    providers: Vec<Provider>,
    on_add_provider: EventHandler<Provider>,
    on_remove_provider: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "space-y-6",

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
                                class: "grid grid-cols-2 gap-4 text-sm",
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
                                        "Provider: "
                                    }
                                    span {
                                        class: "text-gray-900 dark:text-gray-100",
                                        "{model.provider}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Separator {}

            // Providers
            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Providers"
            }
            div {
                class: "space-y-3",
                {providers.iter().map(|provider| {
                    rsx! {
                        ProviderItem {
                            provider: provider.clone(),
                            on_remove: props.on_remove_provider,
                        }
                    }
                })}
                Button {
                    onclick: move |_| {
                        // TODO: Show add provider dialog
                        on_add_provider.call(Provider {
                            id: "new".to_string(),
                            name: "New Provider".to_string(),
                            api_key: None,
                            base_url: None,
                            models: vec![],
                            active: true,
                        });
                    },
                    variant: ButtonVariant::Ghost,
                    class: "w-full justify-start gap-2",
                    "+ Add Provider"
                }
            }
        }
    }
}

#[component]
fn ProviderItem(
    provider: Provider,
    on_remove: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-900 rounded-lg",
            div {
                class: "flex items-center gap-3",
                div {
                    class: "w-8 h-8 bg-blue-500 rounded-lg flex items-center justify-center text-white text-sm font-bold",
                        "{provider.name.chars().next().unwrap_or('P')}"
                }
                div {
                    h4 {
                        class: "font-medium text-gray-900 dark:text-gray-100",
                        "{provider.name}"
                    }
                    p {
                        class: "text-sm text-gray-500 dark:text-gray-400",
                        "{provider.models.len()} models"
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
                    onclick: move |_| on_remove.call(provider.id.clone()),
                    variant: ButtonVariant::Ghost,
                    class: "w-6 h-6 p-0 text-red-500",
                    "×"
                }
            }
        }
    }
}

#[component]
fn AgentSettings(
    agent_config: Option<AgentConfig>,
    on_agent_config_change: EventHandler<AgentConfig>,
) -> Element {
    let config = agent_config.unwrap_or_default();
    let mut config_signal = use_signal(|| config);

    rsx! {
        div {
            class: "space-y-6",

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
                    Select {
                        value: format!("{:?}", config_signal.read().goose_mode),
                        on_value_change: move |value| {
                            let mut config = config_signal.read().clone();
                            config.goose_mode = match value.as_str() {
                                "Chat" => GooseMode::Chat,
                                "Agent" => GooseMode::Agent,
                                "Auto" => GooseMode::Auto,
                                _ => GooseMode::Agent,
                            };
                            config_signal.set(config.clone());
                            on_agent_config_change.call(config);
                        },
                        SelectTrigger {
                            SelectValue {}
                        }
                        SelectContent {
                            SelectItem {
                                value: "Chat",
                                "Chat - Simple conversation mode"
                            }
                            SelectItem {
                                value: "Agent",
                                "Agent - Full agent capabilities with tools"
                            }
                            SelectItem {
                                value: "Auto",
                                "Auto - Automatically choose best mode"
                            }
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
                }
            }
        }
    }
}

#[component]
fn DataSourcesSettings() -> Element {
    rsx! {
        div {
            class: "space-y-6",

            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Data Sources"
            }

            div {
                class: "text-center py-8",
                div {
                    class: "text-4xl mb-2",
                    "📊"
                }
                p {
                    class: "text-gray-500 dark:text-gray-400",
                    "Data source configuration coming soon"
                }
            }
        }
    }
}

#[component]
fn AdvancedSettings() -> Element {
    rsx! {
        div {
            class: "space-y-6",

            h3 {
                class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                "Advanced Settings"
            }

            div {
                class: "space-y-4",
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Log Level"
                    }
                    Select {
                        SelectTrigger {
                            SelectValue {
                                placeholder: "Select log level"
                            }
                        }
                        SelectContent {
                            SelectItem {
                                value: "error",
                                "Error"
                            }
                            SelectItem {
                                value: "warn",
                                "Warning"
                            }
                            SelectItem {
                                value: "info",
                                "Info"
                            }
                            SelectItem {
                                value: "debug",
                                "Debug"
                            }
                        }
                    }
                }

                div {
                    div {
                        class: "flex items-center justify-between",
                        label {
                            class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Enable debug mode"
                        }
                        Switch {
                            checked: false,
                            on_checked_change: move |_checked| {
                                // TODO: Implement debug mode
                            },
                        }
                    }
                }
            }
        }
    }
}