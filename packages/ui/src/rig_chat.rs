// Rig-Integrated Chat Components
use dioxus::prelude::*;
use api::{AgentConfig, GooseMode, ChatRequest, ChatMessage, Role};

#[derive(Clone, PartialEq, Props)]
pub struct SimpleChatMessage {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: Option<String>,
}

#[derive(Clone, PartialEq, Props)]
pub struct SimpleConversationItem {
    pub id: String,
    pub title: String,
    pub last_message: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigChatMessage {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: Option<String>,
    pub agent_mode: Option<GooseMode>,
    pub is_thinking: bool,
}

impl From<RigChatMessage> for SimpleChatMessage {
    fn from(msg: RigChatMessage) -> Self {
        Self {
            id: msg.id,
            content: if msg.is_thinking {
                format!("🧠 Thinking: {}", msg.content)
            } else {
                msg.content
            },
            is_user: msg.is_user,
            timestamp: msg.timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigChatState {
    pub messages: Vec<RigChatMessage>,
    pub agent_config: AgentConfig,
    pub is_streaming: bool,
    pub current_model: String,
}

impl Default for RigChatState {
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
        }
    }
}

#[component]
pub fn RigChatContainer(
    state: Signal<RigChatState>,
    on_send_message: EventHandler<String>,
    on_config_change: Option<EventHandler<AgentConfig>>,
) -> Element {
    let mut message_input = use_signal(String::new);
    let messages = state.read().messages.clone();

    rsx! {
        div { class: "flex h-full bg-gray-50 dark:bg-gray-900",

            // Main Chat Area
            div { class: "flex-1 flex flex-col",
                // Messages area
                div { class: "flex-1 overflow-y-auto p-4 space-y-4",
                    if messages.is_empty() {
                        div { class: "flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400",
                            div { class: "text-6xl mb-4", "💬" }
                            p { class: "text-lg", "Start a conversation!" }
                            p { class: "text-sm mt-2",
                                {
                                    format!("Chat with {} ({})",
                                        state.read().current_model,
                                        format!("{:?}", state.read().agent_config.goose_mode).to_lowercase()
                                    )
                                }
                            }
                        }
                    } else {
                        for message in messages {
                            div {
                                class: if message.is_user {
                                    "flex justify-end"
                                } else {
                                    "flex justify-start"
                                },

                                div {
                                    class: if message.is_user {
                                        "max-w-xs lg:max-w-2xl bg-blue-500 text-white rounded-lg p-3"
                                    } else {
                                        "max-w-xs lg:max-w-2xl bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-lg p-3"
                                    },

                                    p { class: "text-sm", "{message.content}" }

                                    if let Some(timestamp) = message.timestamp {
                                        p { class: "text-xs mt-1 opacity-70", "{timestamp}" }
                                    }
                                }
                            }
                        }

                        if state.read().is_streaming {
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

                // Input area
                div { class: "border-t border-gray-200 dark:border-gray-700 p-4",
                    div { class: "flex gap-2",
                        input {
                            r#type: "text",
                            class: "flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "Type your message here...",
                            value: "{message_input}",
                            oninput: move |evt| message_input.set(evt.value()),
                            disabled: state.read().is_streaming
                        }

                        button {
                            class: if state.read().is_streaming || message_input.read().trim().is_empty() {
                                "px-4 py-2 bg-gray-400 text-white rounded-lg cursor-not-allowed"
                            } else {
                                "px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
                            },
                            onclick: move |_| {
                                let content = message_input.read().clone();
                                if !content.trim().is_empty() && !state.read().is_streaming {
                                    message_input.set(String::new());
                                    on_send_message.call(content);
                                }
                            },
                            disabled: state.read().is_streaming || message_input.read().trim().is_empty(),
                            {
                                if state.read().is_streaming {
                                    "⏳"
                                } else {
                                    "Send"
                                }
                            }
                        }
                    }
                }
            }

            // Agent Configuration Panel (Collapsible)
            div { class: "w-80 border-l border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-4",
                h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                    "Agent Settings"
                }

                // Agent Mode Selector
                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                        "Agent Mode"
                    }
                    select {
                        class: "w-full p-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100",
                        value: "{state.read().agent_config.goose_mode:?}",
                        onchange: move |evt| {
                            let mode = match evt.value().as_str() {
                                "Agent" => GooseMode::Agent,
                                "Auto" => GooseMode::Auto,
                                _ => GooseMode::Chat,
                            };
                            state.write().agent_config.goose_mode = mode;
                            if let Some(ref handler) = on_config_change {
                                handler.call(state.read().agent_config.clone());
                            }
                        },
                        option { value: "Chat", "💬 Chat Mode" }
                        option { value: "Agent", "🔧 Agent Mode" }
                        option { value: "Auto", "🤖 Auto Mode" }
                    }
                    p { class: "text-xs text-gray-500 dark:text-gray-400 mt-1",
                        match state.read().agent_config.goose_mode {
                            GooseMode::Chat => "Natural conversation mode",
                            GooseMode::Agent => "Can use tools and perform complex tasks",
                            GooseMode::Auto => "Autonomous agent that takes initiative",
                        }
                    }
                }

                // Model Selection
                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                        "Model"
                    }
                    select {
                        class: "w-full p-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100",
                        value: "{state.read().current_model}",
                        onchange: move |evt| {
                            state.write().current_model = evt.value();
                        },
                        option { value: "gpt-3.5-turbo", "GPT-3.5 Turbo" }
                        option { value: "gpt-4", "GPT-4" }
                        option { value: "gpt-4-turbo", "GPT-4 Turbo" }
                        option { value: "claude-3", "Claude 3" }
                        option { value: "claude-3.5", "Claude 3.5" }
                        option { value: "mock-local", "Mock Local" }
                    }
                }

                // Agent Statistics
                div { class: "mb-6",
                    h4 { class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-3",
                        "Configuration"
                    }
                    div { class: "space-y-2 text-sm",
                        div { class: "flex justify-between",
                            span { class: "text-gray-600 dark:text-gray-400", "Max Iterations:" }
                            span { class: "font-medium", "{state.read().agent_config.max_iterations}" }
                        }
                        div { class: "flex justify-between",
                            span { class: "text-gray-600 dark:text-gray-400", "Tool Inspection:" }
                            span { class: "font-medium",
                                if state.read().agent_config.enable_tool_inspection { "Enabled" } else { "Disabled" }
                            }
                        }
                        div { class: "flex justify-between",
                            span { class: "text-gray-600 dark:text-gray-400", "Auto Compact:" }
                            span { class: "font-medium",
                                if state.read().agent_config.enable_auto_compact { "Enabled" } else { "Disabled" }
                            }
                        }
                        div { class: "flex justify-between",
                            span { class: "text-gray-600 dark:text-gray-400", "Messages:" }
                            span { class: "font-medium", "{state.read().messages.len()}" }
                        }
                    }
                }

                // Status Indicators
                div { class: "mt-auto",
                    div { class: "flex items-center space-x-2 mb-3",
                        div {
                            class: if state.read().is_streaming {
                                "w-2 h-2 bg-green-500 rounded-full animate-pulse"
                            } else {
                                "w-2 h-2 bg-gray-400 rounded-full"
                            }
                        }
                        span { class: "text-sm text-gray-600 dark:text-gray-400",
                            if state.read().is_streaming { "Processing..." } else { "Ready" }
                        }
                    }

                    if state.read().agent_config.goose_mode != GooseMode::Chat {
                        div { class: "p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg",
                            div { class: "flex items-center mb-1",
                                span { class: "text-blue-600 mr-1", "ℹ️" }
                                span { class: "text-sm font-medium text-blue-700 dark:text-blue-300",
                                    "Agent Mode Active"
                                }
                            }
                            p { class: "text-xs text-blue-600 dark:text-blue-400",
                                "The agent can use tools and perform autonomous actions."
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn RigChatSidebar(
    conversations: Vec<SimpleConversationItem>,
    current_conversation: Option<String>,
    on_select_conversation: EventHandler<String>,
    on_new_conversation: EventHandler,
    on_delete_conversation: EventHandler<String>,
    agent_count: Option<usize>,
) -> Element {
    rsx! {
        div { class: "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full",

            // Header with Agent Count
            div { class: "p-4 border-b border-gray-200 dark:border-gray-700",
                button {
                    onclick: move |_| on_new_conversation.call(()),
                    class: "w-full bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg mb-3",
                    "+ New Conversation"
                }

                if let Some(count) = agent_count {
                    div { class: "text-xs text-gray-500 dark:text-gray-400 text-center",
                        "Available Agents: {count}"
                    }
                }
            }

            // Conversation List
            div { class: "flex-1 overflow-y-auto p-2",
                if conversations.is_empty() {
                    div { class: "text-center text-gray-500 dark:text-gray-400 py-8",
                        div { class: "text-4xl mb-2", "💭" }
                        p { "No conversations yet" }
                        p { class: "text-sm mt-1", "Start a new conversation to see it here" }
                    }
                } else {
                    for conversation in conversations {
                        let conv_id = conversation.id.clone();
                        let current_id = current_conversation.clone();
                        let is_current = current_id.as_ref()
                            .map(|id| id == &conversation.id)
                            .unwrap_or(false);

                        div {
                            class: if is_current {
                                "p-3 mb-2 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 cursor-pointer"
                            } else {
                                "p-3 mb-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer transition-colors"
                            },
                            onclick: move |_| on_select_conversation.call(conv_id.clone()),

                            div { class: "flex justify-between items-start",
                                div { class: "flex-1 min-w-0",
                                    h4 { class: "font-medium text-gray-900 dark:text-gray-100 truncate",
                                        "{conversation.title}"
                                    }
                                    if let Some(last_msg) = conversation.last_message {
                                        p { class: "text-sm text-gray-500 dark:text-gray-400 truncate mt-1",
                                            "{last_msg}"
                                        }
                                    }
                                    if let Some(timestamp) = conversation.timestamp {
                                        p { class: "text-xs text-gray-400 dark:text-gray-500 mt-1",
                                            "{timestamp}"
                                        }
                                    }
                                }
                                button {
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        on_delete_conversation.call(conv_id.clone());
                                    },
                                    class: "w-6 h-6 p-0 opacity-0 hover:opacity-100 transition-opacity text-red-500 hover:text-red-700",
                                    "🗑️"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Utility functions for working with the rig API
pub fn create_chat_request(content: String, config: &AgentConfig, model: String, conversation_history: Vec<RigChatMessage>) -> ChatRequest {
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
        content,
        timestamp: Some(chrono::Utc::now()),
        tool_calls: None,
        tool_results: None,
    });

    ChatRequest {
        messages,
        model,
        system_prompt: None,
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