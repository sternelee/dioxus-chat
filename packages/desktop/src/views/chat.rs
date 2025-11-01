use api::{ChatMessage as ApiMessage, ChatRequest, ChatService};
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

    // Initialize local chat service and models
    use_effect(move || {
        let service = ChatService::new();
        let models = service
            .get_available_models()
            .into_iter()
            .map(|m| Model {
                id: m.id.clone(),
                name: m.name.clone(),
                provider: m.provider.clone(),
                description: m.description.clone(),
                capabilities: m.capabilities.clone(),
            })
            .collect();

        available_models.set(models);
        if let Some(first_model) = available_models().first() {
            selected_model.set(Some(first_model.id.clone()));
        }
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

            let service = ChatService::new();
            match service.send_message(api_request).await {
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
                        content: format!("Error: Failed to get response from local AI model.\n\nDetails: {}\n\nPlease ensure you have the required model files in the models/ directory.", e),
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
    };

    let handle_select_conversation = move |conversation_id: String| {
        current_conversation.set(Some(conversation_id));
        // In a real app, you would load the conversation history from local storage
        messages.set(Vec::new());
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
            class: "flex h-screen bg-gray-100 dark:bg-gray-900",

            // Sidebar
            Sidebar {
                conversations: conversations(),
                current_conversation: current_conversation(),
                on_select_conversation: handle_select_conversation,
                on_new_conversation: handle_new_conversation,
                on_delete_conversation: handle_delete_conversation,
                collapsed: false,
            }

            // Main chat area
            div {
                class: "flex-1 flex flex-col",

                // Header with model selector
                div {
                    class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-4",
                    div {
                        class: "max-w-4xl mx-auto flex items-center justify-between",
                        h1 {
                            class: "text-xl font-semibold text-gray-900 dark:text-gray-100",
                            "Desktop AI Chat (Local Models)"
                        }
                        ModelSelector {
                            models: available_models(),
                            selected_model: selected_model(),
                            on_select_model: handle_select_model,
                            loading: false,
                        }
                    }
                }

                // Chat container
                ChatContainer {
                    messages: messages(),
                    on_send_message: handle_send_message,
                    loading: loading(),
                    placeholder: Some("Type your message here... (Using local AI models)".to_string()),
                }
            }
        }
    }
}

