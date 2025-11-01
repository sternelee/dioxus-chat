use api::{
    get_available_models, send_message, ChatMessage as ApiMessage, ChatRequest, ModelConfig,
};
use dioxus::prelude::*;
use ui::{ChatContainer, ChatMessage, ConversationItem, Model, ModelSelector, Sidebar};

#[component]
pub fn Chat() -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut conversations = use_signal(Vec::<ConversationItem>::new);
    let mut current_conversation = use_signal(|| Option::<String>::None);
    let mut available_models = use_signal(Vec::<Model>::new);
    let mut selected_model = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let mut show_sidebar = use_signal(|| false);

    // Load available models on mount
    use_effect(move || {
        spawn(async move {
            match get_available_models().await {
                Ok(models) => {
                    let ui_models: Vec<Model> = models
                        .into_iter()
                        .map(|m| Model {
                            id: m.id.clone(),
                            name: m.name.clone(),
                            provider: m.provider.clone(),
                            description: m.description.clone(),
                            capabilities: m.capabilities.clone(),
                        })
                        .collect();
                    available_models.set(ui_models);
                    if let Some(first_model) = available_models().first() {
                        selected_model.set(Some(first_model.id.clone()));
                    }
                }
                Err(e) => {
                    println!("Failed to load models: {:?}", e);
                }
            }
        });
    });

    let handle_send_message = move |content: String| {
        let user_message = ChatMessage {
            id: format!("msg_{}", chrono::Utc::now().timestamp_nanos()),
            content: content.clone(),
            is_user: true,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            avatar: None,
        };

        messages.with_mut(|msgs| msgs.push(user_message));
        loading.set(true);

        spawn(async move {
            let api_request = ChatRequest {
                messages: vec![ApiMessage {
                    role: "user".to_string(),
                    content,
                    timestamp: Some(chrono::Utc::now().to_rfc3339()),
                }],
                model: selected_model().unwrap_or_else(|| "llama-2-7b-chat".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(1000),
                stream: Some(false),
            };

            match send_message(api_request).await {
                Ok(response) => {
                    let ai_message = ChatMessage {
                        id: format!("msg_{}", chrono::Utc::now().timestamp_nanos()),
                        content: response.content,
                        is_user: false,
                        timestamp: Some(chrono::Utc::now().to_rfc3339()),
                        avatar: None,
                    };
                    messages.with_mut(|msgs| msgs.push(ai_message));
                }
                Err(e) => {
                    println!("Failed to send message: {:?}", e);
                    let error_message = ChatMessage {
                        id: format!("msg_{}", chrono::Utc::now().timestamp_nanos()),
                        content: format!("Error: Failed to get response from AI. Please try again.\n\nDetails: {}", e),
                        is_user: false,
                        timestamp: Some(chrono::Utc::now().to_rfc3339()),
                        avatar: None,
                    };
                    messages.with_mut(|msgs| msgs.push(error_message));
                }
            }
            loading.set(false);
        });
    };

    let handle_new_conversation = move |_| {
        let new_conversation_id = format!("conv_{}", chrono::Utc::now().timestamp_nanos());
        let new_conversation = ConversationItem {
            id: new_conversation_id.clone(),
            title: "New Chat".to_string(),
            last_message: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            pinned: Some(false),
        };

        conversations.with_mut(|convs| {
            convs.insert(0, new_conversation);
        });

        current_conversation.set(Some(new_conversation_id));
        messages.set(Vec::new());
        show_sidebar.set(false);
    };

    let handle_select_conversation = move |conversation_id: String| {
        current_conversation.set(Some(conversation_id));
        // In a real app, you would load the conversation history here
        messages.set(Vec::new());
        show_sidebar.set(false);
    };

    let handle_delete_conversation = move |conversation_id: String| {
        conversations.with_mut(|convs| {
            convs.retain(|conv| conv.id != conversation_id);
        });

        if current_conversation() == Some(conversation_id) {
            current_conversation.set(None);
            messages.set(Vec::new());
        }
    };

    let handle_select_model = move |model_id: String| {
        selected_model.set(Some(model_id));
    };

    rsx! {
        div {
            class: "flex h-screen bg-gray-100 dark:bg-gray-900 relative",

            // Mobile sidebar overlay
            if show_sidebar() {
                div {
                    class: "fixed inset-0 bg-black bg-opacity-50 z-40",
                    onclick: move |_| show_sidebar.set(false),
                }
            }

            // Sidebar (slide-in on mobile)
            div {
                class: if show_sidebar() {
                    "fixed left-0 top-0 h-full bg-white dark:bg-gray-900 z-50 transition-transform duration-300 ease-in-out"
                } else {
                    "fixed left-0 top-0 h-full bg-white dark:bg-gray-900 z-50 transition-transform duration-300 ease-in-out transform -translate-x-full"
                },
                style: "width: 280px;",

                Sidebar {
                    conversations: conversations(),
                    current_conversation: current_conversation(),
                    on_select_conversation: handle_select_conversation,
                    on_new_conversation: handle_new_conversation,
                    on_delete_conversation: handle_delete_conversation,
                    collapsed: false,
                }
            }

            // Main chat area
            div {
                class: "flex-1 flex flex-col min-w-0",

                // Mobile header with menu button
                div {
                    class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-4 flex items-center gap-3",
                    button {
                        class: "p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                        onclick: move |_| show_sidebar.set(!show_sidebar()),
                        div {
                            class: "w-6 h-6 text-gray-600 dark:text-gray-400",
                            dangerous_inner_html: "☰"
                        }
                    }
                    h1 {
                        class: "text-lg font-semibold text-gray-900 dark:text-gray-100 flex-1",
                        "Mobile AI Chat"
                    }
                    ModelSelector {
                        models: available_models(),
                        selected_model: selected_model(),
                        on_select_model: handle_select_model,
                        loading: false,
                    }
                }

                // Chat container
                ChatContainer {
                    messages: messages(),
                    on_send_message: handle_send_message,
                    loading: loading(),
                    placeholder: Some("Type your message here...".to_string()),
                }
            }
        }
    }
}

