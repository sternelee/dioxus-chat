use dioxus::prelude::*;
use api::{ChatRequest, ChatMessage as ApiMessage, Role, ChatResponse};
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Clone, PartialEq)]
pub struct SimpleChatMessage {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: Option<String>,
    pub thinking_content: Option<String>,
    pub tool_calls: Option<Vec<SimpleToolCall>>,
    pub tool_results: Option<Vec<SimpleToolResult>>,
}

#[derive(Clone, PartialEq)]
pub struct SimpleToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Clone, PartialEq)]
pub struct SimpleToolResult {
    pub tool_call_id: String,
    pub result: String,
    pub error: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct SimpleConversationItem {
    pub id: String,
    pub title: String,
    pub last_message: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub supports_streaming: bool,
}

#[derive(Clone, PartialEq)]
pub struct ConversationState {
    pub id: String,
    pub title: String,
    pub messages: Vec<SimpleChatMessage>,
    pub last_updated: String,
    pub model: Option<String>,
    pub token_usage: TokenUsage,
}

impl ConversationState {
    pub fn new(id: String, title: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            id,
            title,
            messages: Vec::new(),
            last_updated: format!("{}", now),
            model: None,
            token_usage: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        }
    }

    pub fn add_message(&mut self, message: SimpleChatMessage) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_updated = format!("{}", now);
        self.messages.push(message);

        // Update title based on first user message if it's still "New Chat"
        if self.title == "New Chat" && self.messages.len() == 1 {
            if let Some(first_msg) = self.messages.first() {
                if first_msg.is_user {
                    self.title = first_msg.content.chars().take(50).collect::<String>();
                    if first_msg.content.len() > 50 {
                        self.title.push_str("...");
                    }
                }
            }
        }
    }

    pub fn get_last_message(&self) -> Option<String> {
        self.messages.last().map(|msg| msg.content.chars().take(100).collect::<String>())
    }

    pub fn as_conversation_item(&self) -> SimpleConversationItem {
        SimpleConversationItem {
            id: self.id.clone(),
            title: self.title.clone(),
            last_message: self.get_last_message(),
            timestamp: Some(self.last_updated.clone()),
        }
    }
}

#[component]
pub fn SimpleChat() -> Element {
    let mut conversations = use_signal(HashMap::<String, ConversationState>::new);
    let mut current_conversation_id = use_signal(|| Option::<String>::None);
    let mut available_models = use_signal(Vec::<Model>::new);
    let mut selected_model = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let mut streaming = use_signal(|| false);
    let mut message_input = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut models_loaded = use_signal(|| false);

    // Load models from the real API
    use_effect(move || {
        spawn(async move {
            match api::get_available_models().await {
                Ok(models) => {
                    let ui_models = models
                        .into_iter()
                        .map(|m| Model {
                            id: m.id.clone(),
                            name: m.name.clone(),
                            provider: m.provider.clone(),
                            description: m.description.clone(),
                            supports_streaming: m.supports_streaming,
                        })
                        .collect();

                    available_models.set(ui_models);

                    // Set default model if none selected
                    if selected_model().is_none() {
                        if let Some(first_model) = available_models().first() {
                            selected_model.set(Some(first_model.id.clone()));
                        }
                    }

                    models_loaded.set(true);
                }
                Err(e) => {
                    eprintln!("Failed to load models: {}", e);
                    // Fallback to mock models if API fails
                    let fallback_models = vec![
                        Model {
                            id: "mock-local".to_string(),
                            name: "Local Model (Fallback)".to_string(),
                            provider: "Local".to_string(),
                            description: Some("Fallback model for testing".to_string()),
                            supports_streaming: false,
                        },
                    ];

                    available_models.set(fallback_models);
                    models_loaded.set(true);
                }
            }
        });
    });

    // Get current conversation messages
    let current_messages = current_conversation_id().and_then(|id| {
        conversations().get(&id).map(|conv| conv.messages.clone())
    }).unwrap_or_default();

    // Get conversation items for sidebar
    let conversation_items: Vec<SimpleConversationItem> = conversations()
        .values()
        .map(|conv| conv.as_conversation_item())
        .collect();

    let mut handle_send_message = move |content: String| {
        // Clear any previous errors
        error.set(None);

        // Get or create current conversation
        let conv_id = if let Some(id) = current_conversation_id() {
            id.clone()
        } else {
            // Create new conversation if none exists
            let new_id = format!("conv_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
            let new_conv = ConversationState::new(new_id.clone(), "New Chat".to_string());
            conversations.with_mut(|convs| { convs.insert(new_id.clone(), new_conv); });
            current_conversation_id.set(Some(new_id.clone()));
            new_id
        };

        // Create user message
        let user_message = SimpleChatMessage {
            id: format!("msg_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
            content: content.clone(),
            is_user: true,
            timestamp: Some(format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())),
            thinking_content: None,
            tool_calls: None,
            tool_results: None,
        };

        // Add user message to conversation
        conversations.with_mut(|convs| {
            if let Some(conv) = convs.get_mut(&conv_id) {
                conv.add_message(user_message);
                conv.model = selected_model();
            }
        });

        // Get conversation history for API request
        let conversation_history = conversations()
            .get(&conv_id)
            .map(|conv| {
                conv.messages.iter().map(|msg| ApiMessage {
                    role: if msg.is_user { Role::User } else { Role::Assistant },
                    content: msg.content.clone(),
                    timestamp: None, // Simplified for now
                    tool_calls: None,
                    tool_results: None,
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let model_to_use = selected_model().unwrap_or_else(|| "mock-local".to_string());
        loading.set(true);
        streaming.set(true);

        // Create an initial AI message for streaming updates
        let ai_message_id = format!("msg_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

        // Add initial empty AI message
        let initial_ai_message = SimpleChatMessage {
            id: ai_message_id.clone(),
            content: String::new(),
            is_user: false,
            timestamp: Some(format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())),
            thinking_content: None,
            tool_calls: None,
            tool_results: None,
        };

        conversations.with_mut(|convs| {
            if let Some(conv) = convs.get_mut(&conv_id) {
                conv.add_message(initial_ai_message);
            }
        });

        spawn(async move {
            // Prepare API request
            let api_request = ChatRequest {
                messages: conversation_history,
                model: model_to_use.clone(),
                system_prompt: None,
                temperature: Some(0.7),
                max_tokens: Some(2000),
                top_p: None,
                frequency_penalty: None,
                presence_penalty: None,
                stream: true, // Enable streaming
                agent_config: None,
                tools: None,
            };

            // Call the real streaming API
            match api::send_message_stream(api_request).await {
                Ok(response_json) => {
                    // Parse the response
                    match serde_json::from_str::<ChatResponse>(&response_json) {
                        Ok(response) => {
                            if let Some(msg) = response.message {
                                // Convert tool calls if present
                                let tool_calls = msg.tool_calls.map(|calls| {
                                    calls.into_iter().map(|call| SimpleToolCall {
                                        id: call.id,
                                        name: call.name,
                                        arguments: call.arguments.to_string(),
                                    }).collect()
                                });

                                // Convert tool results if present
                                let tool_results = msg.tool_results.map(|results| {
                                    results.into_iter().map(|result| SimpleToolResult {
                                        tool_call_id: result.tool_call_id,
                                        result: result.result.to_string(),
                                        error: result.error,
                                    }).collect()
                                });

                                // Update the final AI message with all content
                                conversations.with_mut(|convs| {
                                    if let Some(conv) = convs.get_mut(&conv_id) {
                                        if let Some(last_msg) = conv.messages.last_mut() {
                                            if !last_msg.is_user {
                                                last_msg.content = msg.content.clone();
                                                last_msg.thinking_content = response.thinking_content.clone();
                                                last_msg.tool_calls = tool_calls;
                                                last_msg.tool_results = tool_results;
                                            }
                                        }
                                    }
                                });

                                // Simulate streaming for the main content only
                                let words: Vec<String> = msg.content.split_whitespace().map(|s| s.to_string()).collect();
                                let words_len = words.len();
                                let mut accumulated_content = String::new();

                                for (i, word) in words.into_iter().enumerate() {
                                    // Add word with space
                                    accumulated_content.push_str(&word);
                                    if i < words_len - 1 {
                                        accumulated_content.push(' ');
                                    }

                                    // Update UI with partial content (thinking content stays complete)
                                    conversations.with_mut(|convs| {
                                        if let Some(conv) = convs.get_mut(&conv_id) {
                                            if let Some(last_msg) = conv.messages.last_mut() {
                                                if !last_msg.is_user {
                                                    last_msg.content = accumulated_content.clone();
                                                }
                                            }
                                        }
                                    });

                                    // Small delay to simulate streaming effect
                                    tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
                                }

                                // Final update with token usage if provided
                                if let Some(usage) = response.token_usage {
                                    conversations.with_mut(|convs| {
                                        if let Some(conv) = convs.get_mut(&conv_id) {
                                            conv.token_usage.prompt_tokens = usage.prompt_tokens;
                                            conv.token_usage.completion_tokens = usage.completion_tokens;
                                            conv.token_usage.total_tokens = usage.total_tokens;
                                        }
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to parse response: {}", e)));
                            // Remove the empty AI message on error
                            conversations.with_mut(|convs| {
                                if let Some(conv) = convs.get_mut(&conv_id) {
                                    conv.messages.pop();
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    error.set(Some(format!("Failed to send message: {}", e)));
                    // Remove the empty AI message on error
                    conversations.with_mut(|convs| {
                        if let Some(conv) = convs.get_mut(&conv_id) {
                            conv.messages.pop();
                        }
                    });
                }
            }

            loading.set(false);
            streaming.set(false);
        });
    };

    rsx! {
        div {
            class: "flex h-screen bg-gray-100 dark:bg-gray-900",
            style: "font-family: system-ui, -apple-system, sans-serif;",

            // Sidebar
            div {
                class: "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full",

                // Header
                div {
                    class: "p-4 border-b border-gray-200 dark:border-gray-700",
                    button {
                        onclick: move |_| {
                            let new_conversation_id = format!("conv_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                            let new_conversation = ConversationState::new(new_conversation_id.clone(), "New Chat".to_string());

                            conversations.with_mut(|convs| {
                                convs.insert(new_conversation_id.clone(), new_conversation);
                            });

                            current_conversation_id.set(Some(new_conversation_id));
                            error.set(None);
                        },
                        class: "w-full justify-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors font-medium",
                        "+ New Chat"
                    }
                }

                // Conversation list
                div {
                    class: "flex-1 overflow-y-auto p-2",

                    if conversation_items.is_empty() {
                        div {
                            class: "text-center text-gray-500 dark:text-gray-400 py-8",
                            p {
                                "No conversations yet"
                            }
                        }
                    } else {
                        {conversation_items.into_iter().map(|conversation| {
                            let conv_id_select = conversation.id.clone();
                            let conv_id_delete = conversation.id.clone();
                            rsx! {
                                div {
                                    key: "{conversation.id}",
                                    class: "p-3 mb-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer transition-colors",
                                    onclick: move |_| {
                                        current_conversation_id.set(Some(conv_id_select.clone()));
                                        error.set(None);
                                    },

                                    div {
                                        class: "flex justify-between items-start",
                                        div {
                                            class: "flex-1 min-w-0",
                                            h4 {
                                                class: "font-medium text-gray-900 dark:text-gray-100 truncate text-sm",
                                                "{conversation.title}"
                                            }
                                            if let Some(last_msg) = conversation.last_message {
                                                p {
                                                    class: "text-xs text-gray-500 dark:text-gray-400 truncate mt-1",
                                                    "{last_msg}"
                                                }
                                            }
                                            if let Some(timestamp) = conversation.timestamp {
                                                p {
                                                    class: "text-xs text-gray-400 dark:text-gray-500 mt-1",
                                                    "{timestamp}"
                                                }
                                            }
                                        }
                                        button {
                                            onclick: move |event| {
                                                event.stop_propagation();
                                                conversations.with_mut(|convs| {
                                                    convs.remove(&conv_id_delete);
                                                });

                                                if current_conversation_id() == Some(conv_id_delete.clone()) {
                                                    current_conversation_id.set(None);
                                                }
                                                error.set(None);
                                            },
                                            class: "w-6 h-6 p-0 opacity-0 hover:opacity-100 transition-opacity text-red-500 hover:text-red-700 text-sm",
                                            "🗑️"
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            // Main chat area
            div {
                class: "flex-1 flex flex-col",

                // Header
                div {
                    class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-4",
                    div {
                        class: "max-w-4xl mx-auto flex items-center justify-between",
                        div {
                            class: "flex items-center gap-4",
                            h1 {
                                class: "text-xl font-semibold text-gray-900 dark:text-gray-100",
                                "AI Chat"
                            }
                            if let Some(conv_id) = current_conversation_id() {
                                if let Some(conv) = conversations().get(&conv_id) {
                                    div {
                                        class: "text-sm text-gray-500 dark:text-gray-400",
                                        "Tokens: {conv.token_usage.total_tokens}"
                                    }
                                }
                            }
                        }
                        select {
                            class: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors text-sm",
                            value: selected_model().unwrap_or_default(),
                            onchange: move |evt| {
                                selected_model.set(Some(evt.value()));
                                // Update current conversation's model
                                if let Some(conv_id) = current_conversation_id() {
                                    conversations.with_mut(|convs| {
                                        if let Some(conv) = convs.get_mut(&conv_id) {
                                            conv.model = Some(evt.value());
                                        }
                                    });
                                }
                            },
                            disabled: available_models().is_empty(),

                            for model in available_models() {
                                option {
                                    value: "{model.id}",
                                    "{model.name} ({model.provider})"
                                }
                            }
                        }
                    }
                }

                // Error display
                if let Some(err) = error() {
                    div {
                        class: "mx-4 mt-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg",
                        div {
                            class: "flex items-start gap-3",
                            span {
                                class: "text-red-500 text-lg",
                                "⚠️"
                            }
                            div {
                                class: "flex-1",
                                h3 {
                                    class: "font-medium text-red-800 dark:text-red-200 text-sm",
                                    "Error"
                                }
                                p {
                                    class: "text-red-700 dark:text-red-300 text-sm mt-1",
                                    "{err}"
                                }
                            }
                            button {
                                class: "text-red-500 hover:text-red-700 text-xl ml-2",
                                onclick: move |_| error.set(None),
                                "×"
                            }
                        }
                    }
                }

                // Chat messages
                div {
                    class: "flex-1 overflow-y-auto p-4",

                    if current_messages.is_empty() {
                        div {
                            class: "flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400",
                            div {
                                class: "text-6xl mb-4",
                                "💬"
                            }
                            p {
                                class: "text-lg",
                                "Start a conversation!"
                            }
                            p {
                                class: "text-sm mt-2",
                                if current_conversation_id().is_some() {
                                    "Type your message here..."
                                } else {
                                    "Start a new conversation or select an existing one..."
                                }
                            }
                        }
                    } else {
                        div {
                            class: "space-y-4",
                            for message in current_messages.clone() {
                                div {
                                    key: "{message.id}",
                                    class: if message.is_user {
                                        "flex justify-end mb-4"
                                    } else {
                                        "flex justify-start mb-4"
                                    },

                                    div {
                                        class: if message.is_user {
                                            "max-w-xs lg:max-w-md bg-blue-500 text-white rounded-lg p-3"
                                        } else {
                                            "max-w-xs lg:max-w-md bg-gray-100 text-gray-800 rounded-lg p-3"
                                        },

                                        if message.is_user {
                                            // User message rendering
                                            div {
                                                div { class: "font-semibold text-sm mb-1", "You" }
                                                p { class: "text-sm", "{message.content}" }
                                                if let Some(ts) = &message.timestamp {
                                                    div { class: "text-xs opacity-75 mt-1", "{ts}" }
                                                }
                                            }
                                        } else {
                                            // Assistant message with complex content
                                            div {
                                                div { class: "font-semibold text-sm mb-1", "Assistant" }

                                                // Thinking content (if present)
                                                if let Some(thinking) = &message.thinking_content {
                                                    if !thinking.is_empty() {
                                                        div { class: "mb-3 p-2 bg-purple-50 border border-purple-200 rounded-md",
                                                            div { class: "flex items-center mb-1",
                                                                span { class: "text-purple-600 mr-1", "🧠" }
                                                                span { class: "font-semibold text-sm text-purple-700", "Thinking:" }
                                                            }
                                                            pre { class: "text-xs text-purple-600 whitespace-pre-wrap font-mono", "{thinking}" }
                                                        }
                                                    }
                                                }

                                                // Tool calls (if present)
                                                if let Some(tool_calls) = &message.tool_calls {
                                                    if !tool_calls.is_empty() {
                                                        div { class: "mb-3 p-2 bg-orange-50 border border-orange-200 rounded-md",
                                                            div { class: "flex items-center mb-2",
                                                                span { class: "text-orange-600 mr-1", "🔧" }
                                                                span { class: "font-semibold text-sm text-orange-700", "Tool Calls:" }
                                                            }
                                                            for tool_call in tool_calls {
                                                                div { class: "mb-2 p-2 bg-white rounded border border-orange-100",
                                                                    div { class: "font-mono text-sm font-semibold text-orange-800",
                                                                        "📞 {tool_call.name}" }
                                                                    pre { class: "text-xs text-gray-600 mt-1 whitespace-pre-wrap font-mono bg-gray-50 p-1 rounded",
                                                                        "{tool_call.arguments}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Tool results (if present)
                                                if let Some(tool_results) = &message.tool_results {
                                                    if !tool_results.is_empty() {
                                                        div { class: "mb-3 p-2 bg-green-50 border border-green-200 rounded-md",
                                                            div { class: "flex items-center mb-2",
                                                                span { class: "text-green-600 mr-1", "✅" }
                                                                span { class: "font-semibold text-sm text-green-700", "Tool Results:" }
                                                            }
                                                            for result in tool_results {
                                                                div { class: "mb-2 p-2 bg-white rounded border border-green-100",
                                                                    div { class: "font-mono text-sm font-semibold text-green-800",
                                                                        "📋 {result.tool_call_id}" }
                                                                    if let Some(error) = &result.error {
                                                                        div { class: "text-xs text-red-600 mt-1", "❌ {error}" }
                                                                    } else {
                                                                        pre { class: "text-xs text-gray-600 mt-1 whitespace-pre-wrap font-mono bg-gray-50 p-1 rounded",
                                                                            "{result.result}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Main content
                                                if !message.content.is_empty() {
                                                    div { class: "text-sm",
                                                        if message.content.contains('\n') {
                                                            pre { class: "whitespace-pre-wrap font-sans", "{message.content}" }
                                                        } else {
                                                            p { "{message.content}" }
                                                        }
                                                    }
                                                }

                                                if let Some(ts) = &message.timestamp {
                                                    div { class: "text-xs text-gray-500 mt-2", "{ts}" }
                                                }

                                                // Show streaming indicator
                                                if streaming() && message.content.is_empty() {
                                                    span {
                                                        class: "inline-block w-2 h-4 bg-blue-500 animate-pulse ml-1",
                                                        "▊"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if loading() {
                                div {
                                    class: "flex justify-start",
                                    div {
                                        class: "bg-gray-200 dark:bg-gray-700 rounded-lg px-4 py-2",
                                        div {
                                            class: "flex space-x-1",
                                            div {
                                                class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                                            }
                                            div {
                                                class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce",
                                                style: "animation-delay: 0.1s"
                                            }
                                            div {
                                                class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce",
                                                style: "animation-delay: 0.2s"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Input area
                div {
                    class: "border-t border-gray-200 dark:border-gray-700 p-4",
                    div {
                        class: "max-w-4xl mx-auto",
                        div {
                            class: "flex gap-2",
                            textarea {
                                class: "flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none",
                                placeholder: if current_conversation_id().is_some() {
                                    "Type your message here..."
                                } else {
                                    "Start a new conversation or select an existing one..."
                                },
                                value: "{message_input}",
                                disabled: loading(),
                                rows: 2,
                                oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                                    message_input.set(evt.value());
                                },
                                onkeydown: move |evt: dioxus::prelude::Event<KeyboardData>| {
                                    if evt.key() == Key::Enter && !evt.modifiers().contains(Modifiers::SHIFT) {
                                        evt.prevent_default();
                                        let content = message_input.read().clone();
                                        if !content.trim().is_empty() {
                                            message_input.set(String::new());
                                            handle_send_message(content);
                                        }
                                    }
                                }
                            }
                            button {
                                class: "px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors disabled:bg-gray-400 flex items-center gap-2",
                                onclick: move |_| {
                                    let content = message_input.read().clone();
                                    if !content.trim().is_empty() {
                                        message_input.set(String::new());
                                        handle_send_message(content);
                                    }
                                },
                                disabled: loading(),
                                if loading() {
                                    div {
                                        class: "w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
                                    }
                                }
                                span {
                                    "Send"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}