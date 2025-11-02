use dioxus::prelude::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;

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

#[component]
pub fn SimpleChatContainer(
    messages: Vec<SimpleChatMessage>,
    on_send_message: EventHandler<String>,
    loading: bool,
    placeholder: Option<String>,
) -> Element {
    let mut message_input = use_signal(String::new);
    let placeholder_text = placeholder.unwrap_or_else(|| "Type your message here...".to_string());

    rsx! {
        div {
            class: "flex flex-col h-full bg-white dark:bg-gray-900",

            // Messages area
            div {
                class: "flex-1 overflow-y-auto p-4 space-y-4",

                if messages.is_empty() {
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
                            {placeholder_text}
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

                                p {
                                    class: "text-sm",
                                    "{message.content}"
                                }

                                if let Some(timestamp) = message.timestamp {
                                    p {
                                        class: "text-xs mt-1 opacity-70",
                                        "{timestamp}"
                                    }
                                }
                            }
                        }
                    }

                    if loading {
                        div {
                            class: "flex justify-start",
                            div {
                                class: "bg-gray-200 dark:bg-gray-700 rounded-lg p-3",
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

            // Input area
            div {
                class: "border-t border-gray-200 dark:border-gray-700 p-4",
                div {
                    class: "flex gap-2",
                    Input {
                        placeholder: "{placeholder_text}",
                        value: "{message_input}",
                        oninput: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| message_input.set(evt.value()),
                        onkeydown: move |evt: KeyboardEvent| {
                            if evt.key() == Key::Enter && !evt.modifiers().contains(KeyModifiers::SHIFT) {
                                evt.prevent_default();
                                let content = message_input.read().clone();
                                if !content.trim().is_empty() {
                                    message_input.set(String::new());
                                    on_send_message.call(content);
                                }
                            }
                        },
                        class: "flex-1",
                        disabled: loading
                    }
                    Button {
                        onclick: move |_| {
                            let content = message_input.read().clone();
                            if !content.trim().is_empty() {
                                message_input.set(String::new());
                                on_send_message.call(content);
                            }
                        },
                        variant: ButtonVariant::Primary,
                        disabled: loading,
                        {
                            if loading {
                                "⏳"
                            } else {
                                "Send"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SimpleSidebar(
    conversations: Vec<SimpleConversationItem>,
    current_conversation: Option<String>,
    on_select_conversation: EventHandler<String>,
    on_new_conversation: EventHandler,
    on_delete_conversation: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full",

            // Header
            div {
                class: "p-4 border-b border-gray-200 dark:border-gray-700",
                Button {
                    onclick: move |_| on_new_conversation.call(()),
                    class: "w-full justify-start gap-2",
                    variant: ButtonVariant::Primary,
                    "+ New Chat"
                }
            }

            // Conversation list
            div {
                class: "flex-1 overflow-y-auto p-2",

                if conversations.is_empty() {
                    div {
                        class: "text-center text-gray-500 dark:text-gray-400 py-8",
                        p {
                            "No conversations yet"
                        }
                    }
                } else {
                    for conversation in conversations {
                        let is_current = if let Some(current_id) = current_conversation.as_ref() {
                            current_id == &conversation.id
                        } else {
                            false
                        };
                        let conv_id = conversation.id.clone();

                        div {
                            class: if is_current {
                                "p-3 mb-2 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 cursor-pointer"
                            } else {
                                "p-3 mb-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer transition-colors"
                            },
                            onclick: move |_| on_select_conversation.call(conv_id.clone()),

                            div {
                                class: "flex justify-between items-start",
                                div {
                                    class: "flex-1 min-w-0",
                                    h4 {
                                        class: "font-medium text-gray-900 dark:text-gray-100 truncate",
                                        "{conversation.title}"
                                    }
                                    if let Some(last_msg) = conversation.last_message {
                                        p {
                                            class: "text-sm text-gray-500 dark:text-gray-400 truncate mt-1",
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
                                Button {
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        on_delete_conversation.call(conv_id.clone());
                                    },
                                    class: "w-6 h-6 p-0 opacity-0 hover:opacity-100 transition-opacity",
                                    variant: ButtonVariant::Ghost,
                                    size: "sm",
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

#[component]
pub fn SimpleModelSelector(
    models: Vec<crate::model_selector::Model>,
    selected_model: Option<String>,
    on_select_model: EventHandler<String>,
    loading: bool,
) -> Element {
    rsx! {
        div {
            class: "relative",
            select {
                class: "w-full p-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                value: selected_model.unwrap_or_default(),
                onchange: move |evt| on_select_model.call(evt.value()),
                disabled: loading,

                for model in models {
                    option {
                        value: "{model.id}",
                        "{model.name} ({model.provider})"
                    }
                }
            }
            if loading {
                div {
                    class: "absolute right-2 top-1/2 -translate-y-1/2",
                    "⏳"
                }
            }
        }
    }
}