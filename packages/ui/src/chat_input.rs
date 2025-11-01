use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct ChatInputProps {
    pub on_send: EventHandler<String>,
    pub disabled: Option<bool>,
    pub placeholder: Option<String>,
    pub streaming: Option<bool>,
    pub on_stop_streaming: Option<EventHandler>,
}

#[derive(Clone, PartialEq)]
pub struct UploadedFile {
    pub name: String,
    pub data_url: String,
    pub size: u64,
    pub file_type: String,
}

#[component]
pub fn ChatInput(props: ChatInputProps) -> Element {
    let mut input = use_signal(|| String::new());
    let mut uploaded_files = use_signal(Vec::<UploadedFile>::new);
    let mut is_drag_over = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    let disabled = props.disabled.unwrap_or(false);
    let streaming = props.streaming.unwrap_or(false);
    let placeholder = props
        .placeholder
        .unwrap_or_else(|| "Type your message...".to_string());

    let handle_send = move |_event: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
        let message = input();
        if (!message.trim().is_empty() || !uploaded_files().is_empty()) && !disabled && !streaming {
            let files = uploaded_files();
            let content = if files.is_empty() {
                message
            } else {
                format!("{}\n\n[Attachments: {}]", message, files.len())
            };

            props.on_send.call(content.clone());
            input.set(String::new());
            uploaded_files.set(Vec::new());
            error_message.set(None);
        }
    };

    let handle_stop_streaming = move |_| {
        if let Some(on_stop) = props.on_stop_streaming {
            on_stop.call(());
        }
    };

    let handle_file_upload = move |_files: Vec<String>| {
        // Placeholder for file upload functionality
        // In a real implementation, this would handle file reading
        error_message.set(Some("File upload functionality coming soon!".to_string()));
    };

    let mut remove_file = move |index: usize| {
        uploaded_files.with_mut(|files| {
            if index < files.len() {
                files.remove(index);
            }
        });
    };

    rsx! {
        div {
            class: "border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900",

            // Error message
            if let Some(error) = error_message() {
                div {
                    class: "px-4 py-2 bg-red-50 dark:bg-red-900/20 border-b border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 text-sm",
                    "{error}"
                }
            }

            // File attachments
            if !uploaded_files().is_empty() {
                div {
                    class: "px-4 py-2 border-b border-gray-200 dark:border-gray-700",
                    div {
                        class: "flex flex-wrap gap-2",
                        for (index, file) in uploaded_files().iter().enumerate() {
                            div {
                                class: "relative inline-flex items-center gap-2 px-3 py-1 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg text-sm",
                                span {
                                    class: "truncate max-w-xs",
                                    "{file.name}"
                                }
                                button {
                                    class: "text-red-500 hover:text-red-700 ml-1",
                                    onclick: move |_| remove_file(index),
                                    "×"
                                }
                            }
                        }
                    }
                }
            }

            // Main input area
            div {
                class: "p-4",
                div {
                    class: if is_drag_over() {
                        "flex gap-3 items-end border-2 border-dashed border-blue-400 bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4"
                    } else {
                        "flex gap-3 items-end"
                    },

                    // Text area
                    div {
                        class: "flex-1 relative",
                        textarea {
                            class: "w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 min-h-12 max-h-48",
                            placeholder: "{placeholder}",
                            value: "{input}",
                            disabled: disabled,
                            rows: 1,
                            oninput: move |event| {
                                input.set(event.value());
                            },
                            onkeydown: move |event| {
                                match event.code() {
                                    Code::Enter if !event.modifiers().shift() => {
                                        event.prevent_default();
                                        let message = input();
                                        if (!message.trim().is_empty() || !uploaded_files().is_empty()) && !disabled && !streaming {
                                            let files = uploaded_files();
                                            let content = if files.is_empty() {
                                                message
                                            } else {
                                                format!("{}\n\n[Attachments: {}]", message, files.len())
                                            };

                                            props.on_send.call(content.clone());
                                            input.set(String::new());
                                            uploaded_files.set(Vec::new());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    // Action buttons
                    div {
                        class: "flex gap-2",

                        if streaming {
                            Button {
                                onclick: handle_stop_streaming,
                                variant: ButtonVariant::Destructive,
                                "Stop"
                            }
                        } else {
                            // File upload button
                            Button {
                                variant: ButtonVariant::Ghost,
                                class: "px-3",
                                "📎"
                            }

                            // Send button
                            Button {
                                onclick: handle_send,
                                disabled: disabled || (input().trim().is_empty() && uploaded_files().is_empty()),
                                variant: ButtonVariant::Primary,
                                class: "px-6",
                                "Send"
                            }
                        }
                    }
                }
            }
        }
    }
}

