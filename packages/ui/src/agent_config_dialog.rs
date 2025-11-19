// Enhanced Agent Configuration Dialog with emoji picker and parameter management
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use api::{AgentConfig, GooseMode};
use crate::ui_components::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentParameter {
    pub key: String,
    pub value: serde_json::Value,
    pub param_type: ParameterType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Json,
}

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AgentData {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub instructions: String,
    pub config: AgentConfig,
    pub parameters: Vec<AgentParameter>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentConfigDialogState {
    pub agent_data: AgentData,
    pub show_emoji_picker: bool,
    pub name_error: Option<String>,
    pub selected_emoji: String,
}

impl Default for AgentConfigDialogState {
    fn default() -> Self {
        Self {
            agent_data: AgentData {
                id: uuid::Uuid::new_v4().to_string(),
                name: String::new(),
                avatar: None,
                description: None,
                instructions: String::new(),
                config: AgentConfig {
                    goose_mode: GooseMode::Chat,
                    max_iterations: 10,
                    require_confirmation: false,
                    readonly_tools: vec![],
                    enable_tool_inspection: true,
                    enable_auto_compact: true,
                    compact_threshold: 0.8,
                    max_turns_without_tools: 3,
                    enable_autopilot: false,
                    enable_extensions: true,
                    extension_timeout: 30,
                },
                parameters: vec![],
            },
            show_emoji_picker: false,
            name_error: None,
            selected_emoji: "🤖".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AgentConfigDialogProps {
    pub open: bool,
    pub on_open_change: EventHandler<bool>,
    pub on_save: EventHandler<AgentData>,
    pub editing_agent: Option<AgentData>,
}

#[component]
pub fn AgentConfigDialog(props: AgentConfigDialogProps) -> Element {
    let mut state = use_signal(AgentConfigDialogState::default);

    // Reset form when dialog opens or editing agent changes
    use_effect(move || {
        if props.open {
            if let Some(ref agent) = props.editing_agent {
                state.set(AgentConfigDialogState {
                    agent_data: agent.clone(),
                    show_emoji_picker: false,
                    name_error: None,
                    selected_emoji: agent.avatar.clone().unwrap_or("🤖".to_string()),
                });
            } else {
                state.set(AgentConfigDialogState::default());
            }
        }
    });

    let handle_save = move |_| {
        let current_state = state.read();
        if current_state.agent_data.name.trim().is_empty() {
            state.write().name_error = Some("Agent name is required".to_string());
            return;
        }

        let mut agent_data = current_state.agent_data.clone();
        agent_data.avatar = Some(current_state.selected_emoji.clone());

        props.on_save.call(agent_data);
        props.on_open_change.call(false);
    };

    let handle_name_change = move |name: String| {
        state.write().agent_data.name = name;
        state.write().name_error = None;
    };

    let handle_emoji_select = move |emoji: String| {
        state.write().selected_emoji = emoji;
        state.write().show_emoji_picker = false;
    };

    let handle_mode_change = move |mode_str: String| {
        let mode = match mode_str.as_str() {
            "Agent" => GooseMode::Agent,
            "Auto" => GooseMode::Auto,
            _ => GooseMode::Chat,
        };
        state.write().agent_data.config.goose_mode = mode;
    };

    rsx! {
        Dialog {
            open: props.open,
            on_open_change: props.on_open_change,
            max_width: Some("max-w-2xl".to_string()),
            DialogHeader {
                DialogTitle {
                    {
                        if props.editing_agent.is_some() {
                            "Edit Agent"
                        } else {
                            "Create New Agent"
                        }
                    }
                }
            }

            DialogContent {
                // Avatar and Name Section
                div { class: "space-y-4",
                    div { class: "flex items-start gap-4",
                        // Avatar/Emoji Selection
                        div { class: "flex flex-col items-center",
                            label { class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                                "Avatar"
                            }
                            div {
                                class: "relative",
                                button {
                                    class: "w-16 h-16 rounded-full border-2 border-dashed border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500 transition-colors flex items-center justify-center",
                                    onclick: move |_| state.write().show_emoji_picker = !state.read().show_emoji_picker,
                                }
                                Avatar {
                                    src: None,
                                    fallback: state.read().selected_emoji.clone(),
                                    size: AvatarSize::Lg,
                                }
                            }

                            // Emoji Picker (simplified version)
                            if state.read().show_emoji_picker {
                                div {
                                    class: "absolute top-full mt-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-3 z-10",
                                    div { class: "grid grid-cols-6 gap-2",
                                        for emoji in ["🤖", "👤", "🧠", "🛠️", "💬", "🎯", "🔮", "⚡", "🌟", "🎨", "📚"] {
                                            button {
                                                class: "text-2xl hover:bg-gray-100 dark:hover:bg-gray-700 p-1 rounded transition-colors",
                                                onclick: move |_| handle_emoji_select(emoji.to_string()),
                                                "{emoji}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Name and Description
                        div { class: "flex-1 space-y-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                                    "Name"
                                }
                                Input {
                                    value: state.read().agent_data.name.clone(),
                                    oninput: handle_name_change,
                                    placeholder: "Enter agent name...",
                                    class: if state.read().name_error.is_some() {
                                        "border-red-500"
                                    } else {
                                        ""
                                    },
                                }
                                if let Some(ref error) = state.read().name_error {
                                    p { class: "text-xs text-red-500 mt-1", "{error}" }
                                }
                            }

                            div {
                                label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                                    "Description"
                                }
                                Textarea {
                                    value: state.read().agent_data.description.clone().unwrap_or_default(),
                                    oninput: move |desc| state.write().agent_data.description = Some(desc),
                                    placeholder: "Enter agent description...",
                                    rows: 3,
                                }
                            }
                        }
                    }
                }

                // Instructions
                div {
                    label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                        "Instructions"
                    }
                    Textarea {
                        value: state.read().agent_data.instructions.clone(),
                        oninput: move |instructions| state.write().agent_data.instructions = instructions,
                        placeholder: "Enter agent instructions...",
                        rows: 4,
                    }
                    p { class: "text-xs text-gray-500 dark:text-gray-400 mt-1",
                        "Instructions guide how the agent should behave and respond"
                    }
                }

                // Agent Configuration
                div {
                    h3 { class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-4",
                        "Configuration"
                    }
                    div { class: "space-y-4",
                        // Agent Mode
                        div {
                            label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                                "Agent Mode"
                            }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{state.read().agent_data.config.goose_mode:?}",
                                onchange: move |evt| handle_mode_change(evt.value()),
                                option { value: "Chat", "💬 Chat Mode - Natural conversation" }
                                option { value: "Agent", "🔧 Agent Mode - Tool using assistant" }
                                option { value: "Auto", "🤖 Auto Mode - Autonomous agent" }
                            }
                            p { class: "text-xs text-gray-500 dark:text-gray-400 mt-1",
                                match state.read().agent_data.config.goose_mode {
                                    GooseMode::Chat => "Optimized for natural conversation",
                                    GooseMode::Agent => "Can use tools and perform complex tasks",
                                    GooseMode::Auto => "Autonomous agent that takes initiative",
                                }
                            }
                        }

                        // Max Iterations
                        div {
                            label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1",
                                "Max Iterations"
                            }
                            Input {
                                value: state.read().agent_data.config.max_iterations.to_string(),
                                r#type: "number".to_string(),
                                oninput: move |value| {
                                    if let Ok(num) = value.parse() {
                                        state.write().agent_data.config.max_iterations = num;
                                    }
                                },
                                class: "w-24",
                            }
                        }

                        // Checkboxes for boolean settings
                        div { class: "space-y-3",
                            div { class: "flex items-center justify-between",
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "Enable Tool Inspection"
                                }
                                Switch {
                                    checked: state.read().agent_data.config.enable_tool_inspection,
                                    on_checked_change: move |checked| {
                                        state.write().agent_data.config.enable_tool_inspection = checked;
                                    },
                                }
                            }

                            div { class: "flex items-center justify-between",
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "Enable Auto Compact"
                                }
                                Switch {
                                    checked: state.read().agent_data.config.enable_auto_compact,
                                    on_checked_change: move |checked| {
                                        state.write().agent_data.config.enable_auto_compact = checked;
                                    },
                                }
                            }

                            div { class: "flex items-center justify-between",
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "Enable Extensions"
                                }
                                Switch {
                                    checked: state.read().agent_data.config.enable_extensions,
                                    on_checked_change: move |checked| {
                                        state.write().agent_data.config.enable_extensions = checked;
                                    },
                                }
                            }

                            div { class: "flex items-center justify-between",
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "Require Confirmation"
                                }
                                Switch {
                                    checked: state.read().agent_data.config.require_confirmation,
                                    on_checked_change: move |checked| {
                                        state.write().agent_data.config.require_confirmation = checked;
                                    },
                                }
                            }
                        }
                    }
                }
            }

            DialogFooter {
                Button {
                    onclick: move |_| props.on_open_change.call(false),
                    variant: ButtonVariant::Ghost,
                    "Cancel"
                }
                Button {
                    onclick: handle_save,
                    variant: ButtonVariant::Primary,
                    {
                        if props.editing_agent.is_some() {
                            "Save Changes"
                        } else {
                            "Create Agent"
                        }
                    }
                }
            }
        }
    }
}