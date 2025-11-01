use crate::components::{
    avatar::{Avatar, AvatarFallback, AvatarImage, AvatarImageSize},
    button::{Button, ButtonVariant},
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct MessageProps {
    pub content: String,
    pub is_user: bool,
    pub timestamp: Option<String>,
    pub avatar: Option<String>,
    pub on_edit: Option<EventHandler<String>>,
    pub on_delete: Option<EventHandler>,
    pub is_last_message: Option<bool>,
}

#[component]
pub fn Message(props: MessageProps) -> Element {
    let mut show_menu = use_signal(|| false);
    let mut is_editing = use_signal(|| false);
    let mut edited_content = use_signal(|| props.content.clone());

    let is_last_message = props.is_last_message.unwrap_or(false);

    let handle_edit = move |_| {
        if let Some(on_edit) = props.on_edit {
            on_edit.call(edited_content());
            is_editing.set(false);
        }
    };

    let handle_delete = move |_| {
        if let Some(on_delete) = props.on_delete {
            on_delete.call(());
        }
    };

    let render_content = || {
        if is_editing() {
            rsx! {
                div {
                    class: "relative",
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white text-sm",
                        rows: 3,
                        value: "{edited_content}",
                        oninput: move |event| {
                            edited_content.set(event.value());
                        }
                    }
                    div {
                        class: "flex gap-2 mt-2 justify-end",
                        Button {
                            onclick: move |_| is_editing.set(false),
                            variant: ButtonVariant::Ghost,
                            "Cancel"
                        }
                        Button {
                            onclick: handle_edit,
                            variant: ButtonVariant::Primary,
                            "Save"
                        }
                    }
                }
            }
        } else {
            // Basic markdown rendering
            let processed_content = props
                .content
                .replace("**", "<strong>")
                .replace("*", "</strong>")
                .replace("```", "<code>")
                .replace("`", "</code>")
                .replace("\n", "<br>");

            rsx! {
                div {
                    class: "prose prose-sm dark:prose-invert max-w-none",
                    dangerous_inner_html: "{processed_content}"
                }
            }
        }
    };

    rsx! {
        div {
            class: "group relative flex gap-3 p-4 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors",

            // Avatar
            Avatar {
                size: AvatarImageSize::Small,
                if let Some(src) = props.avatar.clone() {
                    AvatarImage {
                        src: src.clone(),
                        alt: if props.is_user { "User" } else { "AI" },
                    }
                } else {
                    AvatarFallback {
                        class: if props.is_user { "bg-blue-500" } else { "bg-green-500" },
                        if props.is_user { "U" } else { "AI" }
                    }
                }
            }

            // Message content
            div {
                class: "flex-1 min-w-0",

                // Header with author and timestamp
                div {
                    class: "flex items-center justify-between mb-1",
                    div {
                        class: "flex items-baseline gap-2",
                        span {
                            class: "text-sm font-medium text-gray-900 dark:text-gray-100",
                            if props.is_user { "You" } else { "Assistant" }
                        }
                        if let Some(timestamp) = props.timestamp {
                            span {
                                class: "text-xs text-gray-500 dark:text-gray-400",
                                "{timestamp}"
                            }
                        }
                    }

                    // Action menu (only show for user messages and last message)
                    if props.is_user && is_last_message {
                        div {
                            class: "opacity-0 group-hover:opacity-100 transition-opacity",
                            button {
                                class: "p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded",
                                onclick: move |_| show_menu.set(!show_menu()),
                                "⋮"
                            }

                            if show_menu() {
                                div {
                                    class: "absolute right-0 top-8 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-10 py-1",
                                    if props.on_edit.is_some() {
                                        button {
                                            class: "w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-sm",
                                            onclick: move |_| {
                                                is_editing.set(true);
                                                show_menu.set(false);
                                            },
                                            "Edit"
                                        }
                                    }
                                    if props.on_delete.is_some() {
                                        button {
                                            class: "w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-sm text-red-600 dark:text-red-400",
                                            onclick: handle_delete,
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Message content or edit form
                {render_content()}

                // Message metadata for AI messages
                if !props.is_user {
                    div {
                        class: "mt-2 text-xs text-gray-500 dark:text-gray-400",
                        "AI response generated"
                    }
                }
            }
        }
    }
}

