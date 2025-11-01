use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::switch::Switch;
use api::{AgentConfig, GooseMode};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct SettingsMenuProps {
    pub open: bool,
    pub on_open_change: EventHandler<bool>,
    pub models: Vec<Model>,
    pub selected_model: Option<String>,
    pub on_select_model: EventHandler<String>,
    pub theme: Option<Theme>,
    pub on_theme_change: EventHandler<Theme>,
    pub spell_check: Option<bool>,
    pub on_spell_check_change: EventHandler<bool>,
    pub agent_config: Option<AgentConfig>,
    pub on_agent_config_change: EventHandler<AgentConfig>,
}

#[derive(Clone, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[component]
pub fn SettingsMenu(props: SettingsMenuProps) -> Element {
    // Clone the agent config to avoid borrow checker issues
    let initial_agent_config = props.agent_config.clone().unwrap_or_default();
    let mut agent_config_signal = use_signal(|| initial_agent_config);

    let handle_select_model = move |model_id: String| {
        props.on_select_model.call(model_id);
    };

    let handle_theme_change = move |theme: Theme| {
        props.on_theme_change.call(theme);
    };

    let handle_spell_check_change = move |checked: bool| {
        props.on_spell_check_change.call(checked);
    };

    rsx! {
        if props.open {
            div {
                class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                onclick: move |_| props.on_open_change.call(false),
                div {
                    class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 max-h-[80vh] overflow-y-auto",
                    onclick: move |e: dioxus::prelude::Event<MouseData>| e.stop_propagation(),

                    // Header
                    div {
                        class: "px-6 py-4 border-b border-gray-200 dark:border-gray-700",
                        h2 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                            "Settings"
                        }
                    }

                    // Settings content
                    div {
                        class: "p-6 space-y-6",

                        // Model selection
                        div {
                            h3 {
                                class: "text-sm font-medium text-gray-900 dark:text-gray-100 mb-2",
                                "AI Model"
                            }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500",
                                onchange: move |event| {
                                    handle_select_model(event.value());
                                },

                                option {
                                    value: "",
                                    disabled: true,
                                    selected: props.selected_model.is_none(),
                                    "Select a model"
                                }

                                for model in &props.models {
                                    option {
                                        value: "{model.id}",
                                        selected: props.selected_model.as_ref() == Some(&model.id),
                                        "{model.name} - {model.provider}"
                                    }
                                }
                            }

                            if let Some(description) = props.models.iter()
                                .find(|m| Some(&m.id) == props.selected_model.as_ref())
                                .and_then(|m| m.description.as_ref()) {
                                p {
                                    class: "mt-2 text-sm text-gray-500 dark:text-gray-400",
                                    "{description}"
                                }
                            }
                        }

                        div {
                            class: "border-t border-gray-200 dark:border-gray-700 my-4"
                        }

                        // Theme selection
                        div {
                            h3 {
                                class: "text-sm font-medium text-gray-900 dark:text-gray-100 mb-2",
                                "Theme"
                            }
                            div {
                                class: "space-y-2",
                                Button {
                                    onclick: move |_| handle_theme_change(Theme::Light),
                                    variant: if matches!(props.theme, Some(Theme::Light)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                                    class: "w-full justify-start",
                                    "Light"
                                }
                                Button {
                                    onclick: move |_| handle_theme_change(Theme::Dark),
                                    variant: if matches!(props.theme, Some(Theme::Dark)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                                    class: "w-full justify-start",
                                    "Dark"
                                }
                                Button {
                                    onclick: move |_| handle_theme_change(Theme::Auto),
                                    variant: if matches!(props.theme, Some(Theme::Auto)) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                                    class: "w-full justify-start",
                                    "Auto"
                                }
                            }
                        }

                        div {
                            class: "border-t border-gray-200 dark:border-gray-700 my-4"
                        }

                        // Agent Configuration
                        div {
                            h3 {
                                class: "text-sm font-medium text-gray-900 dark:text-gray-100 mb-2",
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
                                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500",
                                        onchange: move |event: Event<FormData>| {
                                            let mut config = agent_config_signal.read().clone();
                                                config.goose_mode = match event.value().as_str() {
                                                    "Chat" => GooseMode::Chat,
                                                    "Agent" => GooseMode::Agent,
                                                    "Auto" => GooseMode::Auto,
                                                    _ => GooseMode::Agent,
                                                };
                                                agent_config_signal.set(config.clone());
                                                props.on_agent_config_change.call(config);
                                        },

                                        option {
                                            value: "Chat",
                                            selected: matches!(agent_config_signal.read().goose_mode, GooseMode::Chat),
                                            "Chat - Simple conversation mode"
                                        }
                                        option {
                                            value: "Agent",
                                            selected: matches!(agent_config_signal.read().goose_mode, GooseMode::Agent),
                                            "Agent - Full agent capabilities with tools"
                                        }
                                        option {
                                            value: "Auto",
                                            selected: matches!(agent_config_signal.read().goose_mode, GooseMode::Auto),
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
                                        value: agent_config_signal.read().max_iterations.to_string(),
                                        placeholder: "10",
                                        onchange: move |event: Event<FormData>| {
                                            if let Ok(iterations) = event.value().parse::<usize>() {
                                                let mut config = agent_config_signal.read().clone();
                                                config.max_iterations = iterations;
                                                agent_config_signal.set(config.clone());
                                                props.on_agent_config_change.call(config);
                                            }
                                        },
                                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                                            checked: agent_config_signal.read().enable_tool_inspection,
                                            on_checked_change: move |checked| {
                                                let mut config = agent_config_signal.read().clone();
                                                config.enable_tool_inspection = checked;
                                                agent_config_signal.set(config.clone());
                                                props.on_agent_config_change.call(config);
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
                                            checked: agent_config_signal.read().enable_auto_compact,
                                            on_checked_change: move |checked| {
                                                let mut config = agent_config_signal.read().clone();
                                                config.enable_auto_compact = checked;
                                                agent_config_signal.set(config.clone());
                                                props.on_agent_config_change.call(config);
                                            },
                                        }
                                    }
                                    p {
                                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                                        "Automatically compact conversation history when context limit is approached"
                                    }
                                }

                                // Compact Threshold
                                if agent_config_signal.read().enable_auto_compact {
                                    div {
                                        label {
                                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                                            "Compact Threshold (%)"
                                        }
                                        Input {
                                            value: (agent_config_signal.read().compact_threshold * 100.0).to_string(),
                                            placeholder: "80",
                                            onchange: move |event: Event<FormData>| {
                                                if let Ok(threshold) = event.value().parse::<f32>() {
                                                    let mut config = agent_config_signal.read().clone();
                                                    config.compact_threshold = threshold / 100.0;
                                                    agent_config_signal.set(config.clone());
                                                    props.on_agent_config_change.call(config);
                                                }
                                            },
                                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                                        value: agent_config_signal.read().max_turns_without_tools.to_string(),
                                        placeholder: "3",
                                        onchange: move |event: Event<FormData>| {
                                            if let Ok(turns) = event.value().parse::<usize>() {
                                                if let Ok(turns) = event.value().parse::<usize>() {
                                                let mut config = agent_config_signal.read().clone();
                                                    config.max_turns_without_tools = turns;
                                                    props.on_agent_config_change.call(config);
                                                }
                                            }
                                        },
                                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                                            checked: agent_config_signal.read().require_confirmation,
                                            on_checked_change: move |checked| {
                                                let mut config = agent_config_signal.read().clone();
                                                config.require_confirmation = checked;
                                                agent_config_signal.set(config.clone());
                                                props.on_agent_config_change.call(config);
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
                                            checked: agent_config_signal.read().enable_extensions,
                                            on_checked_change: move |checked| {
                                                let mut config = agent_config_signal.read().clone();
                                                config.enable_extensions = checked;
                                                agent_config_signal.set(config.clone());
                                                props.on_agent_config_change.call(config);
                                            },
                                        }
                                    }
                                    p {
                                        class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                                        "Enable agent extensions and plugins"
                                    }
                                }

                                // Extension Timeout
                                if agent_config_signal.read().enable_extensions {
                                    div {
                                        label {
                                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                                            "Extension Timeout (seconds)"
                                        }
                                        Input {
                                            value: agent_config_signal.read().extension_timeout.to_string(),
                                            placeholder: "30",
                                            onchange: move |event: Event<FormData>| {
                                                if let Ok(timeout) = event.value().parse::<u64>() {
                                                    if let Ok(timeout) = event.value().parse::<u64>() {
                                                    let mut config = agent_config_signal.read().clone();
                                                        config.extension_timeout = timeout;
                                                        props.on_agent_config_change.call(config);
                                                    }
                                                }
                                            },
                                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        }
                                        p {
                                            class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                                            "Maximum time to wait for extension responses"
                                        }
                                    }
                                }
                            }
                        }

                        div {
                            class: "border-t border-gray-200 dark:border-gray-700 my-4"
                        }

                        // Advanced settings
                        div {
                            h3 {
                                class: "text-sm font-medium text-gray-900 dark:text-gray-100 mb-2",
                                "Advanced"
                            }
                            div {
                                class: "space-y-2 text-sm text-gray-600 dark:text-gray-400",
                                p { "Version: 1.0.0" }
                                p { "Data directory: ~/.dioxus-chat" }
                                p { "Cache size: 124 MB" }
                            }
                        }
                    }

                    // Footer
                    div {
                        class: "px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-2",
                        Button {
                            onclick: move |_| props.on_open_change.call(false),
                            variant: ButtonVariant::Ghost,
                            "Close"
                        }
                    }
                }
            }
        }
    }
}
