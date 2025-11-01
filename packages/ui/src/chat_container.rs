use crate::chat_input::ChatInput;
use crate::message::Message;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Clone, PartialEq, Props)]
pub struct ChatContainerProps {
    pub messages: Vec<ChatMessage>,
    pub on_send_message: EventHandler<String>,
    pub loading: Option<bool>,
    pub streaming: Option<bool>,
    pub on_stop_streaming: Option<EventHandler>,
    pub placeholder: Option<String>,
}

#[component]
pub fn ChatContainer(props: ChatContainerProps) -> Element {
    let loading = props.loading.unwrap_or(false);
    let streaming = props.streaming.unwrap_or(false);
    let _messages_end = use_signal(|| 0);

    // Auto scroll to bottom when new messages arrive
    use_effect(move || {
        // This will trigger the effect when messages change
        // The actual scrolling will be handled in the div below
    });

    rsx! {
        div {
            class: "flex flex-col h-full bg-white dark:bg-gray-900",
            // Messages area
            div {
                class: "flex-1 overflow-y-auto",
                div {
                    class: "max-w-4xl mx-auto",
                    if props.messages.is_empty() {
                        div {
                            class: "flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400 py-12",
                            div {
                                class: "text-6xl mb-4",
                                "💬"
                            }
                            h3 {
                                class: "text-xl font-semibold mb-2",
                                "Start a conversation"
                            }
                            p {
                                class: "text-center",
                                "Type your message below to begin chatting with the AI assistant."
                            }
                        }
                    } else {
                        for message in &props.messages {
                            Message {
                                key: "{message.id}",
                                content: message.content.clone(),
                                is_user: message.is_user,
                                timestamp: message.timestamp.clone(),
                                avatar: message.avatar.clone(),
                            }
                        }
                        if loading || streaming {
                            div {
                                class: "flex gap-3 p-4",
                                div {
                                    class: "w-8 h-8 bg-green-500 rounded-full flex items-center justify-center text-white text-sm font-medium",
                                    "AI"
                                }
                                div {
                                    class: "flex gap-1 items-center",
                                    div { class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce" }
                                    div { class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce" }
                                    div { class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce" }
                                }
                                span {
                                    class: "ml-2 text-sm text-gray-500 dark:text-gray-400",
                                    if streaming { "AI is thinking..." } else { "Loading..." }
                                }
                            }
                        }
                    }
                    div {
                        style: "height: 1px;"
                    }
                }
            }
            // Input area
            ChatInput {
                on_send: props.on_send_message,
                disabled: loading,
                streaming: streaming,
                on_stop_streaming: props.on_stop_streaming,
                placeholder: props.placeholder,
            }
        }
    }
}

