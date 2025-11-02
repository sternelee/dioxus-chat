use crate::components::{
    avatar::{Avatar, AvatarFallback, AvatarImageSize},
    button::{Button, ButtonVariant},
    dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger},
    separator::Separator,
    toast::Toast,
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct ThreadContentProps {
    pub messages: Vec<ChatMessage>,
    pub streaming_content: Option<String>,
    pub on_copy_message: EventHandler<String>,
    pub on_regenerate_response: EventHandler<String>,
    pub on_edit_message: EventHandler<(String, String)>,
    pub on_delete_message: EventHandler<String>,
    pub is_last_message_streaming: Option<bool>,
}

#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Option<String>,
    pub avatar_url: Option<String>,
    pub model_name: Option<String>,
    pub is_streaming: Option<bool>,
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub metadata: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
    pub status: ToolCallStatus,
}

#[derive(Clone, PartialEq)]
pub enum ToolCallStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[component]
pub fn ThreadContent(props: ThreadContentProps) -> Element {
    let messages = props.messages.clone();
    let streaming_content = props.streaming_content.clone();

    rsx! {
        div {
            class: "flex-1 overflow-y-auto bg-white dark:bg-gray-900",

            div {
                class: "max-w-4xl mx-auto px-4 py-6 space-y-6",

                // Render all messages
                {messages.iter().enumerate().map(|(index, message)| {
                    let is_last = index == messages.len() - 1;
                    let is_streaming = props.is_last_message_streaming.unwrap_or(false) && is_last;

                    rsx! {
                        MessageItem {
                            message: message.clone(),
                            is_last: is_last,
                            is_streaming: is_streaming,
                            on_copy: props.on_copy_message,
                            on_regenerate: props.on_regenerate_response,
                            on_edit: props.on_edit_message,
                            on_delete: props.on_delete_message,
                        }
                    }
                })}

                // Streaming content display
                if let Some(streaming) = streaming_content {
                    MessageItem {
                        message: ChatMessage {
                            id: "streaming".to_string(),
                            role: MessageRole::Assistant,
                            content: streaming,
                            timestamp: None,
                            avatar_url: None,
                            model_name: None,
                            is_streaming: Some(true),
                            reasoning_content: None,
                            tool_calls: None,
                            metadata: None,
                        },
                        is_last: true,
                        is_streaming: true,
                        on_copy: props.on_copy_message,
                        on_regenerate: props.on_regenerate_response,
                        on_edit: props.on_edit_message,
                        on_delete: props.on_delete_message,
                    }
                }
            }
        }
    }
}

#[component]
fn MessageItem(
    message: ChatMessage,
    is_last: bool,
    is_streaming: bool,
    on_copy: EventHandler<String>,
    on_regenerate: EventHandler<String>,
    on_edit: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
) -> Element {
    let mut show_dropdown = use_signal(|| false);
    let copied = use_signal(|| false);

    let handle_copy = move |text: String| {
        // TODO: Implement actual clipboard API
        copied.set(true);
        on_copy.call(text);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            copied.set(false);
        });
    };

    let message_role_class = match message.role {
        MessageRole::User => "justify-end",
        MessageRole::Assistant => "justify-start",
        MessageRole::System => "justify-center",
        MessageRole::Tool => "justify-start",
    };

    rsx! {
        div {
            class: "flex {message_role_class} group",

            match message.role {
                MessageRole::User => rsx! {
                    UserMessage {
                        message: message.clone(),
                        on_edit: on_edit,
                        on_delete: on_delete,
                    }
                },
                MessageRole::Assistant => rsx! {
                    AssistantMessage {
                        message: message.clone(),
                        is_last: is_last,
                        is_streaming: is_streaming,
                        on_copy: on_copy,
                        on_regenerate: on_regenerate,
                        on_edit: on_edit,
                        on_delete: on_delete,
                        copied: *copied.read(),
                    }
                },
                MessageRole::System => rsx! {
                    SystemMessage {
                        message: message.clone(),
                    }
                },
                MessageRole::Tool => rsx! {
                    ToolMessage {
                        message: message.clone(),
                    }
                },
            }
        }
    }
}

#[component]
fn UserMessage(
    message: ChatMessage,
    on_edit: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
) -> Element {
    let mut show_dropdown = use_signal(|| false);
    let edit_message_id = message.id.clone();
    let delete_message_id = message.id.clone();
    let edit_message_content = message.content.clone();

    rsx! {
        div {
            class: "max-w-3xl",

            div {
                class: "flex items-end gap-3",

                // Avatar
                Avatar {
                    size: AvatarImageSize::Medium,
                    AvatarFallback {
                        class: "bg-blue-500 text-white",
                        "U"
                    }
                }

                // Message content
                div {
                    class: "flex-1",

                    div {
                        class: "bg-blue-500 text-white rounded-2xl rounded-br-sm px-4 py-3 inline-block",

                        // Message text
                        div {
                            class: "text-sm whitespace-pre-wrap",
                            "{message.content}"
                        }

                        // Timestamp
                        if let Some(ts) = &message.timestamp {
                            div {
                                class: "text-xs text-blue-100 mt-1 opacity-75",
                                "{ts}"
                            }
                        }
                    }
                }

                // Action buttons
                DropdownMenu {
                    DropdownMenuTrigger {
                        Button {
                            onclick: move |_| show_dropdown.set(true),
                            class: "opacity-0 group-hover:opacity-100 transition-opacity w-8 h-8 p-0",
                            variant: ButtonVariant::Ghost,
                            "⋯"
                        }
                    }
                    DropdownMenuContent {
                        DropdownMenuItem::<String> {
                            value: "edit".to_string(),
                            index: 0usize,
                            on_select: move |_: String| {
                                on_edit.call((edit_message_id.clone(), edit_message_content.clone()));
                                show_dropdown.set(false);
                            },
                            "Edit"
                        }
                        DropdownMenuItem::<String> {
                            value: "delete".to_string(),
                            index: 1usize,
                            on_select: move |_: String| {
                                on_delete.call(delete_message_id.clone());
                                show_dropdown.set(false);
                            },
                            class: "text-red-600 dark:text-red-400",
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AssistantMessage(
    message: ChatMessage,
    is_last: bool,
    is_streaming: bool,
    on_copy: EventHandler<String>,
    on_regenerate: EventHandler<String>,
    on_edit: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
    copied: bool,
) -> Element {
    let mut show_dropdown = use_signal(|| false);
    let regenerate_message_id = message.id.clone();
    let copy_message_content = message.content.clone();
    let edit_message_id = message.id.clone();
    let delete_message_id = message.id.clone();
    let edit_message_content = message.content.clone();

    rsx! {
        div {
            class: "max-w-4xl",

            div {
                class: "flex items-start gap-3",

                // Avatar
                Avatar {
                    size: AvatarImageSize::Medium,
                    AvatarFallback {
                        class: "bg-green-500 text-white",
                        "AI"
                    }
                }

                // Message content
                div {
                    class: "flex-1 space-y-3",

                    // Model info
                    if let Some(model) = &message.model_name {
                        div {
                            class: "flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400",
                            span {
                                class: "bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded",
                                "{model}"
                            }
                        }
                    }

                    // Reasoning content (thinking)
                    if let Some(reasoning) = &message.reasoning_content {
                        div {
                            class: "bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3",
                            div {
                                class: "flex items-center gap-2 text-sm text-yellow-800 dark:text-yellow-200 mb-2",
                                span {
                                    class: "animate-pulse",
                                    "🤔"
                                }
                                "Thinking..."
                            }
                            div {
                                class: "text-sm font-mono text-yellow-700 dark:text-yellow-300",
                                "{reasoning}"
                            }
                        }
                    }

                    // Main message content
                    div {
                        class: "bg-gray-100 dark:bg-gray-800 rounded-2xl rounded-bl-sm px-4 py-3",

                        // Message text
                        div {
                            class: "text-sm text-gray-900 dark:text-gray-100 whitespace-pre-wrap",
                            if is_streaming {
                                "{message.content}█"
                            } else {
                                "{message.content}"
                            }
                        }

                        // Tool calls
                        if let Some(tool_calls) = &message.tool_calls {
                            div {
                                class: "mt-3 space-y-2",
                                {tool_calls.iter().map(|tool_call| {
                                    rsx! {
                                        ToolCallDisplay {
                                            tool_call: tool_call.clone(),
                                        }
                                    }
                                })}
                            }
                        }
                    }

                    // Action buttons
                    div {
                        class: "flex items-center gap-2 mt-2 opacity-0 group-hover:opacity-100 transition-opacity",

                        if is_last && !is_streaming {
                            Button {
                                onclick: move |_| on_regenerate.call(regenerate_message_id.clone()),
                                class: "text-xs",
                                variant: ButtonVariant::Ghost,
                                size: "sm",
                                "🔄 Regenerate"
                            }
                        }

                        Button {
                            onclick: move |_| handle_copy(copy_message_content.clone()),
                            class: "text-xs",
                            variant: ButtonVariant::Ghost,
                            size: "sm",
                            {if copied { "✓ Copied" } else { "📋 Copy" }}
                        }

                        DropdownMenu {
                            DropdownMenuTrigger {
                                Button {
                                    onclick: move |_| show_dropdown.set(true),
                                    class: "text-xs w-6 h-6 p-0",
                                    variant: ButtonVariant::Ghost,
                                    size: "sm",
                                    "⋯"
                                }
                            }
                            DropdownMenuContent {
                                DropdownMenuItem::<String> {
                                    value: "copy".to_string(),
                                    index: 0usize,
                                    on_select: move |_: String| {
                                        handle_copy(copy_message_content.clone());
                                        show_dropdown.set(false);
                                    },
                                    "Copy"
                                }
                                DropdownMenuItem::<String> {
                                    value: "edit".to_string(),
                                    index: 1usize,
                                    on_select: move |_: String| {
                                        on_edit.call((edit_message_id.clone(), edit_message_content.clone()));
                                        show_dropdown.set(false);
                                    },
                                    "Edit"
                                }
                                DropdownMenuItem::<String> {
                                    value: "delete".to_string(),
                                    index: 2usize,
                                    on_select: move |_: String| {
                                        on_delete.call(delete_message_id.clone());
                                        show_dropdown.set(false);
                                    },
                                    class: "text-red-600 dark:text-red-400",
                                    "Delete"
                                }
                            }
                        }

                        // Timestamp
                        if let Some(ts) = &message.timestamp {
                            span {
                                class: "text-xs text-gray-500 dark:text-gray-400 ml-auto",
                                "{ts}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SystemMessage(message: ChatMessage) -> Element {
    rsx! {
        div {
            class: "max-w-2xl mx-auto",
            div {
                class: "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg px-4 py-2 text-sm text-center",
                "{message.content}"
            }
        }
    }
}

#[component]
fn ToolMessage(message: ChatMessage) -> Element {
    rsx! {
        div {
            class: "max-w-3xl",
            div {
                class: "bg-purple-50 dark:bg-purple-900/20 border border-purple-200 dark:border-purple-800 rounded-lg p-3",
                div {
                    class: "flex items-center gap-2 text-sm text-purple-800 dark:text-purple-200 mb-2",
                    span {
                        "🔧"
                    }
                    "Tool Call"
                }
                div {
                    class: "text-sm font-mono text-purple-700 dark:text-purple-300",
                    "{message.content}"
                }
            }
        }
    }
}

#[component]
fn ToolCallDisplay(tool_call: ToolCall) -> Element {
    let status_icon = match tool_call.status {
        ToolCallStatus::Pending => "⏳",
        ToolCallStatus::Running => "🔄",
        ToolCallStatus::Completed => "✅",
        ToolCallStatus::Failed => "❌",
    };

    rsx! {
        div {
            class: "bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg p-3",
            div {
                class: "flex items-center justify-between mb-2",
                div {
                    class: "flex items-center gap-2 text-sm",
                    span {
                        "{status_icon}"
                    }
                    span {
                        class: "font-medium text-gray-900 dark:text-gray-100",
                        "{tool_call.name}"
                    }
                }
                span {
                    class: "text-xs text-gray-500 dark:text-gray-400",
                    "ID: {tool_call.id}"
                }
            }

            if !tool_call.arguments.is_empty() {
                div {
                    class: "text-xs font-mono text-gray-600 dark:text-gray-400 mb-2",
                    "Arguments: {tool_call.arguments}"
                }
            }

            if let Some(result) = &tool_call.result {
                div {
                    class: "text-xs bg-gray-50 dark:bg-gray-800 rounded p-2 font-mono",
                    "{result}"
                }
            }
        }
    }
}

