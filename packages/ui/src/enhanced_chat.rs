// Enhanced Chat Interface with agent configuration and improved UI
use dioxus::prelude::*;
use api::{AgentConfig, GooseMode, ChatRequest, ChatMessage, Role};
use crate::ui_components::*;
use crate::agent_config_dialog::{AgentConfigDialog, AgentData};
use crate::parameter_manager::ParameterManager;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct EnhancedChatMessage {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: Option<String>,
    pub agent_name: Option<String>,
    pub agent_mode: Option<GooseMode>,
    pub is_thinking: bool,
    pub token_usage: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnhancedChatState {
    pub messages: Vec<EnhancedChatMessage>,
    pub agent_config: AgentConfig,
    pub is_streaming: bool,
    pub current_model: String,
    pub agent_name: String,
    pub show_config_dialog: bool,
    pub editing_agent: Option<AgentData>,
}

impl Default for EnhancedChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            agent_config: AgentConfig {
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
            is_streaming: false,
            current_model: "gpt-3.5-turbo".to_string(),
            agent_name: "Assistant".to_string(),
            show_config_dialog: false,
            editing_agent: None,
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct EnhancedChatContainerProps {
    pub state: Signal<EnhancedChatState>,
    pub on_send_message: EventHandler<String>,
    pub on_agent_config_change: Option<EventHandler<AgentConfig>>,
    pub available_models: Vec<String>,
}

#[component]
pub fn EnhancedChatContainer(props: EnhancedChatContainerProps) -> Element {
    let mut message_input = use_signal(String::new);
    let messages = props.state.read().messages.clone();

    rsx! {
        div { class: "flex h-full bg-gray-50 dark:bg-gray-900",

            // Main Chat Area
            div { class: "flex-1 flex flex-col",
                // Chat Header
                div { class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-4",
                    div { class: "flex items-center justify-between",
                        div { class: "flex items-center gap-3",
                            // Agent Avatar
                            Avatar {
                                src: None,
                                fallback: props.state.read().agent_name.chars().next().unwrap_or('A').to_string(),
                                size: AvatarSize::Md,
                            }

                            // Agent Info
                            div {
                                h3 { class: "font-medium text-gray-900 dark:text-gray-100",
                                    "{props.state.read().agent_name}"
                                }
                                div { class: "flex items-center gap-2 mt-1",
                                    Badge {
                                        variant: BadgeVariant::Secondary,
                                        "{props.state.read().current_model}"
                                    }
                                    Badge {
                                        variant: BadgeVariant::Outline,
                                        {
                                            format!("{:?}", props.state.read().agent_config.goose_mode)
                                        }
                                    }
                                    if props.state.read().is_streaming {
                                        Badge {
                                            variant: BadgeVariant::Default,
                                            "Streaming..."
                                        }
                                    }
                                }
                            }
                        }

                        // Configuration Button
                        Button {
                            onclick: move |_| {
                                props.state.write().show_config_dialog = true;
                            },
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            class: "ml-auto",
                            "⚙️"
                        }
                    }
                }

                // Messages Area
                div { class: "flex-1 overflow-y-auto p-4",
                    if messages.is_empty() {
                        div { class: "flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400",
                            div { class: "text-6xl mb-4", "💬" }
                            h3 { class: "text-lg font-medium mb-2", "Start a conversation!" }
                            p { class: "text-sm",
                                {
                                    format!("Chat with {} in {} mode",
                                        props.state.read().current_model,
                                        format!("{:?}", props.state.read().agent_config.goose_mode).to_lowercase()
                                    )
                                }
                            }
                        }
                    } else {
                        for (index, message) in messages.iter().enumerate() {
                            EnhancedMessageBubble {
                                key: "{message.id}-{index}",
                                message: message.clone(),
                            }
                        }

                        if props.state.read().is_streaming {
                            div { class: "flex justify-start",
                                div { class: "bg-gray-200 dark:bg-gray-700 rounded-lg p-3",
                                    div { class: "flex space-x-1",
                                        div { class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce" }
                                        div { class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce", style: "animation-delay: 0.1s" }
                                        div { class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce", style: "animation-delay: 0.2s" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Input Area with Enhanced Features
                div { class: "border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-4",
                    div { class: "space-y-3",
                        // Quick Actions Bar
                        div { class: "flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400",
                            span { class: "font-medium", "Quick Actions:" }
                            button {
                                class: "px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors",
                                onclick: move |_| {
                                    let content = "Help me understand the current conversation context";
                                    props.on_send_message.call(content.to_string());
                                },
                                "📝 Summarize"
                            }
                            button {
                                class: "px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors",
                                onclick: move |_| {
                                    let content = "What tools can you use in your current mode?";
                                    props.on_send_message.call(content.to_string());
                                },
                                "🛠️ Show Tools"
                            }
                            button {
                                class: "px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors",
                                onclick: move |_| {
                                    let content = "Explain your current agent mode and capabilities";
                                    props.on_send_message.call(content.to_string());
                                },
                                "🔍 Explain Mode"
                            }
                        }

                        // Message Input Area
                        div { class: "flex gap-2",
                            // Model Selector
                            select {
                                class: "px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{props.state.read().current_model}",
                                onchange: move |evt| props.state.write().current_model = evt.value(),
                                for model in props.available_models.iter() {
                                    option { value: "{model}", "{model}" }
                                }
                            }

                            // Message Input
                            div { class: "flex-1",
                                textarea {
                                    r#type: "text",
                                    class: "w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none",
                                    placeholder: "Type your message here...",
                                    value: "{message_input}",
                                    oninput: move |evt| message_input.set(evt.value()),
                                    disabled: props.state.read().is_streaming,
                                    rows: 1,
                                    style: "min-height: 48px; max-height: 120px;",
                                }
                            }

                            // Send Button
                            button {
                                class: if props.state.read().is_streaming || message_input.read().trim().is_empty() {
                                    "px-6 py-3 bg-gray-400 text-white rounded-lg cursor-not-allowed"
                                } else {
                                    "px-6 py-3 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
                                },
                                onclick: move |_| {
                                    let content = message_input.read().clone();
                                    if !content.trim().is_empty() && !props.state.read().is_streaming {
                                        message_input.set(String::new());
                                        props.on_send_message.call(content);
                                    }
                                },
                                disabled: props.state.read().is_streaming || message_input.read().trim().is_empty(),
                                {
                                    if props.state.read().is_streaming {
                                        "⏳ Processing..."
                                    } else {
                                        "Send Message"
                                    }
                                }
                            }
                        }

                        // Status Bar
                        div { class: "flex items-center justify-between text-xs text-gray-500 dark:text-gray-400",
                            div { class: "flex items-center gap-4",
                                span {
                                    {
                                        format!("Agent: {} ({})",
                                            props.state.read().agent_name,
                                            format!("{:?}", props.state.read().agent_config.goose_mode).to_lowercase()
                                        )
                                    }
                                }
                                span {
                                    {
                                        format!("Iterations: {}", props.state.read().agent_config.max_iterations)
                                    }
                                }
                            }
                            div { class: "flex items-center gap-2",
                                if let Some(token_count) = messages.iter().flat_map(|m| m.token_usage).reduce(|a, b| a + b) {
                                    span {
                                        {
                                            format!("Tokens used: {}", token_count)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Agent Configuration Dialog
        AgentConfigDialog {
            open: props.state.read().show_config_dialog,
            on_open_change: move |open| {
                props.state.write().show_config_dialog = open;
                if !open {
                    props.state.write().editing_agent = None;
                }
            },
            on_save: move |agent_data| {
                // Update agent configuration from saved data
                props.state.write().agent_name = agent_data.name;
                props.state.write().agent_config = agent_data.config;
                if let Some(ref handler) = props.on_agent_config_change {
                    handler.call(agent_data.config);
                }
            },
            editing_agent: props.state.read().editing_agent.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct EnhancedMessageBubbleProps {
    pub message: EnhancedChatMessage,
}

#[component]
pub fn EnhancedMessageBubble(props: EnhancedMessageBubbleProps) -> Element {
    rsx! {
        div {
            class: if props.message.is_user {
                "flex justify-end mb-4"
            } else {
                "flex justify-start mb-4"
            },

            div {
                class: if props.message.is_user {
                    "max-w-xs lg:max-w-2xl bg-blue-500 text-white rounded-lg p-3 shadow-md"
                } else {
                    "max-w-xs lg:max-w-2xl bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 rounded-lg p-3 shadow-md border border-gray-200 dark:border-gray-700"
                },

                // Message Header with Agent Info
                if !props.message.is_user {
                    div { class: "flex items-center justify-between mb-2",
                        div { class: "flex items-center gap-2",
                            if let Some(ref agent_name) = props.message.agent_name {
                                Avatar {
                                    src: None,
                                    fallback: agent_name.chars().next().unwrap_or('A').to_string(),
                                    size: AvatarSize::Sm,
                                }
                                span { class: "text-xs font-medium", "{agent_name}" }
                            }
                            if let Some(ref mode) = props.message.agent_mode {
                                Badge {
                                    variant: BadgeVariant::Outline,
                                    class: "text-xs",
                                    "{mode:?}"
                                }
                            }
                        }
                        if let Some(timestamp) = props.message.timestamp {
                            span { class: "text-xs opacity-70", "{timestamp}" }
                        }
                    }
                }

                // Message Content
                div {
                    class: "text-sm leading-relaxed",
                    if props.message.is_thinking {
                        span { class: "italic opacity-75", "🧠 Thinking: " }
                    }
                    "{props.message.content}"
                }

                // Message Footer with Token Usage
                if let Some(token_usage) = props.message.token_usage {
                    div { class: "mt-2 text-xs opacity-60 flex justify-between",
                        span {
                            {
                                format!("Tokens: {}", token_usage)
                            }
                        }
                        if props.message.is_user {
                            span { class: "text-right", "You" }
                        } else {
                            span { class: "text-right", "Assistant" }
                        }
                    }
                }
            }
        }
    }
}

// Utility function to create a chat request
pub fn create_enhanced_chat_request(
    content: String,
    config: &AgentConfig,
    model: String,
    conversation_history: Vec<EnhancedChatMessage>,
    agent_name: &str,
) -> ChatRequest {
    let mut messages: Vec<ChatMessage> = conversation_history
        .into_iter()
        .map(|msg| ChatMessage {
            role: if msg.is_user { Role::User } else { Role::Assistant },
            content: msg.content,
            timestamp: msg.timestamp.and_then(|ts| chrono::DateTime::parse_from_rfc3339(&ts).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            tool_calls: None,
            tool_results: None,
        })
        .collect();

    // Add the new user message
    messages.push(ChatMessage {
        role: Role::User,
        content: format!("[Agent: {}] {}", agent_name, content),
        timestamp: Some(chrono::Utc::now()),
        tool_calls: None,
        tool_results: None,
    });

    ChatRequest {
        messages,
        model,
        system_prompt: Some(format!("You are {}. Act in {:?} mode.", agent_name, config.goose_mode)),
        temperature: Some(0.7),
        max_tokens: None,
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stream: true,
        agent_config: Some(config.clone()),
        tools: None,
    }
}