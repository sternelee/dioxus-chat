use dioxus::prelude::*;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agent::{
    Agent, AgentEvent, Conversation, AgentFactory, UiChatMessage, MessageContent,
    EnhancedStreamChunk, ChunkType,
};

/// State for the Goose Chat component
#[derive(Clone)]
pub struct GooseChatState {
    pub agent: Arc<RwLock<Box<dyn Agent>>>,
    pub conversation: Option<Conversation>,
    pub messages: Vec<UiChatMessage>,
    pub is_loading: bool,
    pub current_input: String,
    pub error: Option<String>,
    pub selected_model: String,
    pub agent_mode: String,
}

impl Default for GooseChatState {
    fn default() -> Self {
        // This will be properly initialized in the component
        Self {
            agent: Arc::new(RwLock::new(
                async_std::task::block_on(async {
                    AgentFactory::create_default_agent()
                        .await
                        .unwrap_or_else(|_| {
                            panic!("Failed to create default agent")
                        })
                })
            )),
            conversation: None,
            messages: Vec::new(),
            is_loading: false,
            current_input: String::new(),
            error: None,
            selected_model: "mock-local".to_string(),
            agent_mode: "chat".to_string(),
        }
    }
}

/// Main Goose Chat component
#[component]
pub fn GooseChat() -> Element {
    let mut state = use_signal(GooseChatState::default);
    let messages_ref = use_signal(Vec::<UiChatMessage>::new);

    // Initialize agent on mount
    use_coroutine(|_| {
        let state = state.clone();
        async move {
            if let Err(e) = initialize_agent(&mut state.write()).await {
                state.write().error = Some(format!("Failed to initialize agent: {}", e));
            }
        }
    });

    rsx! {
        div { class: "flex h-screen bg-gray-50 dark:bg-gray-900",
            // Sidebar for conversation management
            div { class: "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full",
                ConversationSidebar {
                    state: state.clone(),
                    messages: messages_ref.clone(),
                }
            }

            // Main chat area
            div { class: "flex-1 flex flex-col h-full",
                ChatHeader {
                    state: state.clone(),
                }

                ChatMessages {
                    messages: messages_ref.clone(),
                }

                ChatInput {
                    state: state.clone(),
                    messages: messages_ref.clone(),
                }
            }
        }
    }
}

/// Initialize the agent with rig integration
async fn initialize_agent(state: &mut GooseChatState) -> anyhow::Result<()> {
    let agent = AgentFactory::create_default_agent().await?;
    state.agent = Arc::new(RwLock::new(agent));
    Ok(())
}

/// Conversation sidebar component
#[component]
fn ConversationSidebar(
    state: Signal<GooseChatState>,
    messages: Signal<Vec<UiChatMessage>>,
) -> Element {
    let conversations = use_signal(Vec::<Conversation>::new);

    rsx! {
        div { class: "p-4 border-b border-gray-200 dark:border-gray-700",
            h2 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100", "Conversations" }
            button {
                class: "mt-2 w-full px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors",
                onclick: move |_| {
                    spawn(async move {
                        if let Err(e) = create_new_conversation(state, messages).await {
                            state.write().error = Some(format!("Failed to create conversation: {}", e));
                        }
                    });
                },
                "+ New Chat"
            }
        }

        div { class: "flex-1 overflow-y-auto p-2",
            if conversations.read().is_empty() {
                div { class: "text-center text-gray-500 dark:text-gray-400 py-8",
                    p { "No conversations yet. Start a new chat!" }
                }
            } else {
                {conversations.read().iter().map(|conv| {
                    rsx! {
                        div {
                            key: "{conv.id}",
                            class: "p-3 mb-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer transition-colors",
                            onclick: move |_| {
                                spawn(async move {
                                    if let Err(e) = load_conversation(state, messages, conv.id.clone()).await {
                                        state.write().error = Some(format!("Failed to load conversation: {}", e));
                                    }
                                });
                            },
                            div { class: "font-medium text-gray-900 dark:text-gray-100",
                                "{conv.get_title()}"
                            }
                            div { class: "text-sm text-gray-500 dark:text-gray-400 mt-1",
                                "{conv.messages.len()} messages"
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Chat header with model selection
#[component]
fn ChatHeader(state: Signal<GooseChatState>) -> Element {
    rsx! {
        div { class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 p-4",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center space-x-4",
                    h1 { class: "text-xl font-semibold text-gray-900 dark:text-gray-100", "Goose Chat" }

                    // Agent mode selector
                    select {
                        class: "px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100",
                        value: "{state.read().agent_mode}",
                        onchange: move |evt| {
                            state.write().agent_mode = evt.value.clone();
                            spawn(async move {
                                if let Err(e) = switch_agent_mode(state, evt.value).await {
                                    state.write().error = Some(format!("Failed to switch agent mode: {}", e));
                                }
                            });
                        },
                        option { value: "chat", "Chat Mode" }
                        option { value: "reasoning", "Reasoning Mode" }
                        option { value: "tool", "Tool Mode" }
                    }

                    // Model selector
                    select {
                        class: "px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100",
                        value: "{state.read().selected_model}",
                        onchange: move |evt| {
                            state.write().selected_model = evt.value.clone();
                        },
                        option { value: "mock-local", "Mock Local" }
                        option { value: "deepseek-chat", "DeepSeek Chat" }
                        option { value: "deepseek-r1-distill-llama-70b", "DeepSeek R1" }
                        option { value: "openai/gpt-4o", "GPT-4o" }
                        option { value: "anthropic/claude-3.5-sonnet", "Claude 3.5 Sonnet" }
                        option { value: "google/gemini-1.5-pro", "Gemini 1.5 Pro" }
                    }
                }

                if let Some(ref error) = state.read().error {
                    div { class: "text-red-500 text-sm",
                        "{error}"
                    }
                }
            }
        }
    }
}

/// Message display area
#[component]
fn ChatMessages(messages: Signal<Vec<UiChatMessage>>) -> Element {
    let scroll_area_ref = use_node_ref();

    // Auto-scroll when new messages arrive
    use_effect(move || {
        if let Some(element) = scroll_area_ref.get() {
            use web_sys::HtmlElement;
            if let Some(html_element) element.dyn_ref::<HtmlElement>() {
                html_element.scroll_into_view_with_scroll_into_view_options(
                    &web_sys::ScrollIntoViewOptions::new().block(web_sys::ScrollLogicalPosition::End)
                );
            }
        }
    });

    rsx! {
        div {
            ref: scroll_area_ref,
            class: "flex-1 overflow-y-auto p-4 space-y-4",
            if messages.read().is_empty() {
                div { class: "text-center text-gray-500 dark:text-gray-400 mt-8",
                    h3 { class: "text-lg font-medium mb-2", "Welcome to Goose Chat!" }
                    p { "Start a conversation by typing a message below." }
                }
            } else {
                {messages.read().iter().map(|message| {
                    rsx! {
                        MessageBubble {
                            message: message.clone(),
                        }
                    }
                })}
            }
        }
    }
}

/// Individual message bubble
#[component]
fn MessageBubble(message: UiChatMessage) -> Element {
    let is_user = matches!(message.role, Role::User);

    rsx! {
        div {
            class: if is_user {
                "flex justify-end mb-4"
            } else {
                "flex justify-start mb-4"
            },
            div {
                class: if is_user {
                    "max-w-xs lg:max-w-md bg-blue-500 text-white rounded-lg p-3"
                } else {
                    "max-w-xs lg:max-w-md bg-gray-100 text-gray-800 rounded-lg p-3"
                },

                // Message content with support for different types
                match &message.content {
                    MessageContent::Text(text) => {
                        rsx! {
                            div { class: if is_user { "text-sm" } else { "text-sm" },
                                if text.contains('\n') {
                                    pre { class: "whitespace-pre-wrap font-sans", "{text}" }
                                } else {
                                    p { "{text}" }
                                }
                            }
                        }
                    }
                    MessageContent::Thinking(content) => {
                        rsx! {
                            div { class: "mb-3 p-2 bg-purple-50 border border-purple-200 rounded-md",
                                div { class: "flex items-center mb-1",
                                    span { class: "text-purple-600 mr-1", "🧠" }
                                    span { class: "font-semibold text-sm text-purple-700", "Thinking:" }
                                }
                                pre { class: "text-xs text-purple-600 whitespace-pre-wrap font-mono", "{content}" }
                            }
                        }
                    }
                    MessageContent::ToolCall(tool_call) => {
                        rsx! {
                            div { class: "mb-3 p-2 bg-orange-50 border border-orange-200 rounded-md",
                                div { class: "flex items-center mb-2",
                                    span { class: "text-orange-600 mr-1", "🔧" }
                                    span { class: "font-semibold text-sm text-orange-700", "Tool Call:" }
                                }
                                div { class: "text-sm font-mono text-orange-800",
                                    "{tool_call.name}" }
                                pre { class: "text-xs text-gray-600 mt-1 whitespace-pre-wrap font-mono bg-gray-50 p-1 rounded",
                                    "{tool_call.arguments}" }
                            }
                        }
                    }
                    MessageContent::ToolResult(tool_result) => {
                        rsx! {
                            div { class: "mb-3 p-2 bg-green-50 border border-green-200 rounded-md",
                                div { class: "flex items-center mb-2",
                                    span { class: "text-green-600 mr-1", "✅" }
                                    span { class: "font-semibold text-sm text-green-700", "Tool Result:" }
                                }
                                if let Some(ref error) = tool_result.error {
                                    div { class: "text-xs text-red-600", "❌ {error}" }
                                } else {
                                    pre { class: "text-xs text-gray-600 whitespace-pre-wrap font-mono bg-gray-50 p-1 rounded",
                                        "{tool_result.result}" }
                                }
                            }
                        }
                    }
                }

                // Timestamp
                if let Some(timestamp) = &message.timestamp {
                    div { class: "text-xs text-gray-500 dark:text-gray-400 mt-2",
                        "{timestamp.format(\"%H:%M\")}"
                    }
                }
            }
        }
    }
}

/// Chat input component
#[component]
fn ChatInput(
    state: Signal<GooseChatState>,
    messages: Signal<Vec<UiChatMessage>>,
) -> Element {
    let input_ref = use_node_ref();

    rsx! {
        div { class: "border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 p-4",
            div { class: "flex space-x-2",
                input {
                    ref: input_ref,
                    r#type: "text",
                    class: "flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                    placeholder: "Type your message...",
                    value: "{state.read().current_input}",
                    oninput: move |evt| {
                        state.write().current_input = evt.value.clone();
                    },
                    onkeydown: move |evt| {
                        if evt.key == "Enter" && !evt.modifiers().contains(dioxus::html::input_data::KeyboardModifiers::SHIFT) {
                            evt.prevent_default();
                            spawn(async move {
                                if let Err(e) = send_message(state, messages).await {
                                    state.write().error = Some(format!("Failed to send message: {}", e));
                                }
                            });
                        }
                    },
                    disabled: state.read().is_loading
                }

                button {
                    class: "px-4 py-2 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white rounded-lg transition-colors",
                    onclick: move |_| {
                        spawn(async move {
                            if let Err(e) = send_message(state, messages).await {
                                state.write().error = Some(format!("Failed to send message: {}", e));
                            }
                        });
                    },
                    disabled: state.read().is_loading || state.read().current_input.trim().is_empty(),
                    if state.read().is_loading {
                        "..."
                    } else {
                        "Send"
                    }
                }
            }
        }
    }
}

/// Create a new conversation
async fn create_new_conversation(
    state: Signal<GooseChatState>,
    messages: Signal<Vec<UiChatMessage>>,
) -> anyhow::Result<()> {
    let conversation = Conversation::new(Vec::new())?;
    messages.set(Vec::new());
    state.write().conversation = Some(conversation.clone());
    state.write().error = None;
    Ok(())
}

/// Load an existing conversation
async fn load_conversation(
    state: Signal<GooseChatState>,
    messages: Signal<Vec<UiChatMessage>>,
    conversation_id: String,
) -> anyhow::Result<()> {
    // In a real implementation, this would load from storage
    // For now, we'll create a simple placeholder
    let conversation = Conversation::new(Vec::new())?;
    messages.set(conversation.messages.clone());
    state.write().conversation = Some(conversation);
    state.write().error = None;
    Ok(())
}

/// Switch agent mode
async fn switch_agent_mode(
    state: Signal<GooseChatState>,
    mode: String,
) -> anyhow::Result<()> {
    let agent = match mode.as_str() {
        "reasoning" => AgentFactory::create_reasoning_agent().await?,
        "tool" => AgentFactory::create_tool_agent().await?,
        _ => AgentFactory::create_default_agent().await?,
    };

    state.write().agent = Arc::new(RwLock::new(agent));
    state.write().error = None;
    Ok(())
}

/// Send a message using the agent
async fn send_message(
    state: Signal<GooseChatState>,
    messages: Signal<Vec<UiChatMessage>>,
) -> anyhow::Result<()> {
    let input = state.read().current_input.clone();
    if input.trim().is_empty() {
        return Ok(());
    }

    // Create user message
    let user_message = UiChatMessage {
        role: Role::User,
        content: MessageContent::Text(input.clone()),
        timestamp: Some(chrono::Utc::now()),
        tool_calls: None,
        tool_results: None,
        metadata: None,
    };

    // Get or create conversation
    let conversation = if let Some(ref conv) = state.read().conversation {
        conv.clone()
    } else {
        let new_conv = Conversation::new(Vec::new())?;
        state.write().conversation = Some(new_conv.clone());
        new_conv
    };

    // Add user message to conversation
    let mut updated_conversation = conversation.clone();
    updated_conversation.add_message(user_message.clone());

    // Update UI
    let mut current_messages = messages.read().clone();
    current_messages.push(user_message);
    messages.set(current_messages);

    // Set loading state
    state.write().is_loading = true;
    state.write().current_input.clear();
    state.write().error = None;

    // Get agent and send message
    let agent = state.read().agent.clone();
    let mut agent_guard = agent.write().await;

    let mut event_stream = agent.reply(
        updated_conversation.clone(),
        Some("You are a helpful assistant. Provide clear and helpful responses."),
        None,
    ).await?;

    let mut response_message = None;
    let mut current_messages = messages.read().clone();

    // Process events
    while let Some(event_result) = event_stream.next().await {
        match event_result? {
            AgentEvent::Message(msg) => {
                if !matches!(msg.role, Role::User) {
                    current_messages.push(msg.clone());
                    messages.set(current_messages.clone());
                    response_message = Some(msg);
                }
            }
            AgentEvent::ToolCall(tool_call) => {
                // Handle tool call - for now just display it
                let tool_msg = UiChatMessage {
                    role: Role::Assistant,
                    content: MessageContent::ToolCall(tool_call),
                    timestamp: Some(chrono::Utc::now()),
                    tool_calls: None,
                    tool_results: None,
                    metadata: None,
                };
                current_messages.push(tool_msg);
                messages.set(current_messages.clone());
            }
            AgentEvent::ToolResult(tool_result) => {
                // Handle tool result - for now just display it
                let result_msg = UiChatMessage {
                    role: Role::Assistant,
                    content: MessageContent::ToolResult(tool_result),
                    timestamp: Some(chrono::Utc::now()),
                    tool_calls: None,
                    tool_results: None,
                    metadata: None,
                };
                current_messages.push(result_msg);
                messages.set(current_messages.clone());
            }
            AgentEvent::Token(_usage) => {
                // Update token usage display (could be added to UI)
            }
            AgentEvent::Error(error) => {
                state.write().error = Some(error);
                break;
            }
            AgentEvent::Done => {
                break;
            }
        }
    }

    // Update conversation with response
    if let Some(response) = response_message {
        updated_conversation.add_message(response);
        if let Some(ref mut conv) = state.write().conversation {
            *conv = updated_conversation;
        }
    }

    state.write().is_loading = false;
    Ok(())
}