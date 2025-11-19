// Enhanced Streaming Chat Components
use dioxus::prelude::*;
use futures::StreamExt;
use api::{EnhancedStreamChunk, ChunkType, StreamMetadata, ChatMessage, Role};

#[derive(Debug, Clone, PartialEq)]
pub enum StreamingState {
    Idle,
    Connecting,
    Streaming,
    Thinking,
    ToolCall,
    ToolResult,
    Complete,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct StreamingMessage {
    pub content: String,
    pub chunk_type: ChunkType,
    pub metadata: Option<StreamMetadata>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_complete: bool,
}

#[component]
pub fn StreamingChatContainer(
    messages: Signal<Vec<StreamingMessage>>,
    on_send_message: EventHandler<String>,
    streaming_state: Signal<StreamingState>,
    current_input: Signal<String>,
    placeholder: Option<String>,
) -> Element {
    let scroll_area_ref = use_node_ref();

    // Auto-scroll when new messages arrive
    use_effect(move || {
        if let Some(element) = scroll_area_ref.get() {
            use web_sys::HtmlElement;
            if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
                html_element.scroll_into_view_with_scroll_into_view_options(
                    &web_sys::ScrollIntoViewOptions::new().block(web_sys::ScrollLogicalPosition::End)
                );
            }
        }
    });

    rsx! {
        div { class: "flex flex-col h-full bg-gray-50 dark:bg-gray-900",
            // Streaming Status Bar
            StreamingStatusBar {
                state: streaming_state.clone(),
            }

            // Messages Area
            div {
                ref: scroll_area_ref,
                class: "flex-1 overflow-y-auto p-4 space-y-4",
                if messages.read().is_empty() {
                    div { class: "text-center text-gray-500 dark:text-gray-400 mt-8",
                        h3 { class: "text-lg font-medium mb-2", "Start a conversation" }
                        p { "Type a message below to begin chatting with the AI assistant." }
                    }
                } else {
                    {messages.read().iter().map(|message| {
                        rsx! {
                            StreamingMessageBubble {
                                key: "{message.timestamp.timestamp_nanos()}",
                                message: message.clone(),
                            }
                        }
                    })}
                }
            }

            // Input Area
            StreamingChatInput {
                current_input: current_input.clone(),
                on_send_message,
                streaming_state: streaming_state.clone(),
                placeholder: placeholder.unwrap_or("Type your message...".to_string()),
            }
        }
    }
}

#[component]
fn StreamingStatusBar(state: Signal<StreamingState>) -> Element {
    rsx! {
        div { class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-4 py-2",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center space-x-2",
                    match *state.read() {
                        StreamingState::Idle => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-gray-400 rounded-full" }
                                span { class: "text-sm text-gray-600 dark:text-gray-400", "Ready" }
                            }
                        },
                        StreamingState::Connecting => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-yellow-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-yellow-600 dark:text-yellow-400", "Connecting..." }
                            }
                        },
                        StreamingState::Streaming => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-blue-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-blue-600 dark:text-blue-400", "Receiving response..." }
                            }
                        },
                        StreamingState::Thinking => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-purple-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-purple-600 dark:text-purple-400", "Thinking..." }
                            }
                        },
                        StreamingState::ToolCall => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-orange-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-orange-600 dark:text-orange-400", "Using tools..." }
                            }
                        },
                        StreamingState::ToolResult => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-green-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-green-600 dark:text-green-400", "Processing tool results..." }
                            }
                        },
                        StreamingState::Complete => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-green-500 rounded-full" }
                                span { class: "text-sm text-green-600 dark:text-green-400", "Complete" }
                            }
                        },
                        StreamingState::Error(ref error) => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-red-500 rounded-full" }
                                span { class: "text-sm text-red-600 dark:text-red-400", "Error: {error}" }
                            }
                        },
                    }
                }

                // Token count could be added here
                div { class: "text-xs text-gray-500 dark:text-gray-400",
                    "Enhanced streaming enabled"
                }
            }
        }
    }
}

#[component]
fn StreamingMessageBubble(message: StreamingMessage) -> Element {
    let is_user = matches!(message.chunk_type, ChunkType::Content) && message.content.contains("User:");

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

                // Message content based on chunk type
                match message.chunk_type {
                    ChunkType::Content => {
                        rsx! {
                            div { class: "text-sm",
                                if message.content.contains('\n') {
                                    pre { class: "whitespace-pre-wrap font-sans", "{message.content}" }
                                } else {
                                    p { "{message.content}" }
                                }
                            }
                        }
                    },
                    ChunkType::Thinking => {
                        rsx! {
                            div { class: "mb-2 p-2 bg-purple-50 border border-purple-200 rounded-md",
                                div { class: "flex items-center mb-1",
                                    span { class: "text-purple-600 mr-1", "🧠" }
                                    span { class: "font-semibold text-sm text-purple-700", "Thinking:" }
                                }
                                pre { class: "text-xs text-purple-600 whitespace-pre-wrap font-mono", "{message.content}" }
                            }
                        }
                    },
                    ChunkType::ToolCall => {
                        rsx! {
                            div { class: "mb-2 p-2 bg-orange-50 border border-orange-200 rounded-md",
                                div { class: "flex items-center mb-2",
                                    span { class: "text-orange-600 mr-1", "🔧" }
                                    span { class: "font-semibold text-sm text-orange-700", "Tool Call:" }
                                }
                                div { class: "text-sm font-mono text-orange-800",
                                    "{message.content}"
                                }
                                if let Some(ref metadata) = message.metadata {
                                    div { class: "text-xs text-gray-500 mt-1",
                                        "Agent: {metadata.agent_name} | Iteration: {metadata.iteration}"
                                    }
                                }
                            }
                        }
                    },
                    ChunkType::ToolResult => {
                        rsx! {
                            div { class: "mb-2 p-2 bg-green-50 border border-green-200 rounded-md",
                                div { class: "flex items-center mb-2",
                                    span { class: "text-green-600 mr-1", "✅" }
                                    span { class: "font-semibold text-sm text-green-700", "Tool Result:" }
                                }
                                pre { class: "text-xs text-gray-600 whitespace-pre-wrap font-mono bg-gray-50 p-1 rounded",
                                    "{message.content}"
                                }
                            }
                        }
                    },
                    ChunkType::Metadata => {
                        rsx! {
                            div { class: "text-xs text-gray-500 italic",
                                "{message.content}"
                            }
                        }
                    },
                    ChunkType::Error => {
                        rsx! {
                            div { class: "mb-2 p-2 bg-red-50 border border-red-200 rounded-md",
                                div { class: "flex items-center mb-1",
                                    span { class: "text-red-600 mr-1", "❌" }
                                    span { class: "font-semibold text-sm text-red-700", "Error:" }
                                }
                                div { class: "text-sm text-red-800",
                                    "{message.content}"
                                }
                            }
                        }
                    },
                }

                // Timestamp and metadata
                div { class: "text-xs text-gray-500 dark:text-gray-400 mt-2 flex items-center justify-between",
                    span { "{message.timestamp.format(\"%H:%M:%S\")}" }
                    if !message.is_complete {
                        span { class: "animate-pulse", "⚡" }
                    }
                }
            }
        }
    }
}

#[component]
fn StreamingChatInput(
    current_input: Signal<String>,
    on_send_message: EventHandler<String>,
    streaming_state: Signal<StreamingState>,
    placeholder: String,
) -> Element {
    let input_ref = use_node_ref();
    let is_streaming = matches!(*streaming_state.read(),
        StreamingState::Connecting | StreamingState::Streaming | StreamingState::Thinking |
        StreamingState::ToolCall | StreamingState::ToolResult
    );

    rsx! {
        div { class: "border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 p-4",
            div { class: "flex space-x-2",
                input {
                    ref: input_ref,
                    r#type: "text",
                    class: "flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                    placeholder: "{placeholder}",
                    value: "{current_input.read()}",
                    oninput: move |evt| {
                        current_input.set(evt.value.clone());
                    },
                    onkeydown: move |evt| {
                        if evt.key == "Enter" && !evt.modifiers().contains(dioxus::html::input_data::KeyboardModifiers::SHIFT) {
                            evt.prevent_default();
                            let message = current_input.read().clone();
                            if !message.trim().is_empty() && !is_streaming {
                                current_input.set(String::new());
                                on_send_message.call(message);
                            }
                        }
                    },
                    disabled: is_streaming
                }

                button {
                    class: if is_streaming {
                        "px-4 py-2 bg-gray-400 text-white rounded-lg cursor-not-allowed"
                    } else if current_input.read().trim().is_empty() {
                        "px-4 py-2 bg-gray-400 text-white rounded-lg cursor-not-allowed"
                    } else {
                        "px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
                    },
                    onclick: move |_| {
                        let message = current_input.read().clone();
                        if !message.trim().is_empty() && !is_streaming {
                            current_input.set(String::new());
                            on_send_message.call(message);
                        }
                    },
                    disabled: is_streaming || current_input.read().trim().is_empty(),
                    if is_streaming {
                        div { class: "flex items-center",
                            span { class: "animate-spin h-4 w-4 mr-1", "⟳" }
                            "Sending..."
                        }
                    } else {
                        "Send"
                    }
                }

                // Enhanced Features Button (could expand to show tool status, etc.)
                button {
                    class: "px-3 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors",
                    title: "Enhanced streaming features enabled",
                    disabled: true,
                    "⚡"
                }
            }

            // Input hints
            if is_streaming {
                div { class: "mt-2 text-xs text-gray-500 dark:text-gray-400 flex items-center",
                    span { class: "animate-pulse mr-1", "●" }
                    span { "AI is responding... Enhanced streaming with tool support enabled." }
                }
            }
        }
    }
}

#[component]
pub fn StreamingControls(
    on_toggle_streaming: EventHandler<bool>,
    on_clear_history: EventHandler<()>,
    is_streaming_enabled: bool,
) -> Element {
    rsx! {
        div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4",
            h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                "Streaming Controls"
            }

            div { class: "space-y-4",
                // Streaming Toggle
                div { class: "flex items-center justify-between",
                    label { class: "flex items-center cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "mr-3 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: is_streaming_enabled,
                            onchange: move |evt| {
                                on_toggle_streaming.call(evt.checked);
                            }
                        }
                        div {
                            span { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                "Enhanced Streaming"
                            }
                            p { class: "text-xs text-gray-500 dark:text-gray-400",
                                "Enable real-time response with thinking process and tool visualization"
                            }
                        }
                    }
                    div {
                        class: if is_streaming_enabled {
                            "w-8 h-4 bg-blue-600 rounded-full relative"
                        } else {
                            "w-8 h-4 bg-gray-300 rounded-full relative"
                        },
                        div {
                            class: if is_streaming_enabled {
                                "absolute right-0 top-0 w-4 h-4 bg-white rounded-full transition-transform"
                            } else {
                                "absolute left-0 top-0 w-4 h-4 bg-white rounded-full transition-transform"
                            }
                        }
                    }
                }

                // Action Buttons
                div { class="flex space-x-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors",
                        onclick: move |_| {
                            on_clear_history.call(());
                        },
                        "Clear History"
                    }
                }

                // Streaming Features Info
                if is_streaming_enabled {
                    div { class="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg",
                        h4 { class="text-sm font-medium text-blue-900 dark:text-blue-100 mb-2",
                            "Enhanced Features Active:"
                        }
                        ul { class="text-xs text-blue-800 dark:text-blue-200 space-y-1",
                            li { "● Real-time streaming responses" }
                            li { "● Thinking process visualization" }
                            li { "● Tool call and result display" }
                            li { "● Metadata and agent information" }
                            li { "● Enhanced error handling" }
                        }
                    }
                }
            }
        }
    }
}