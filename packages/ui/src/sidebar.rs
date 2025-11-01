use crate::components::{
    avatar::{Avatar, AvatarFallback, AvatarImageSize},
    button::{Button, ButtonVariant},
    separator::Separator,
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct SidebarProps {
    pub conversations: Vec<ConversationItem>,
    pub current_conversation: Option<String>,
    pub on_select_conversation: EventHandler<String>,
    pub on_new_conversation: EventHandler,
    pub on_delete_conversation: EventHandler<String>,
    pub collapsed: Option<bool>,
}

#[derive(Clone, PartialEq)]
pub struct ConversationItem {
    pub id: String,
    pub title: String,
    pub last_message: Option<String>,
    pub timestamp: Option<String>,
    pub pinned: Option<bool>,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let collapsed = props.collapsed.unwrap_or(false);
    let conversations = props.conversations.clone();
    let current_conversation = props.current_conversation.clone();
    let on_select_conversation = props.on_select_conversation;
    let on_new_conversation = props.on_new_conversation;
    let on_delete_conversation = props.on_delete_conversation;

    rsx! {
        div {
            class: if collapsed {
                "w-16 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col"
            } else {
                "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col"
            },

            // Header
            div {
                class: "p-4 border-b border-gray-200 dark:border-gray-700",
                if !collapsed {
                    Button {
                        onclick: move |_| on_new_conversation.call(()),
                        class: "w-full",
                        "+ New Chat"
                    }
                } else {
                    Button {
                        onclick: move |_| on_new_conversation.call(()),
                        class: "w-full p-2",
                        variant: ButtonVariant::Ghost,
                        "+"
                    }
                }
            }

            Separator {},

            // Conversations list
            div {
                class: "flex-1 overflow-y-auto",
                if !collapsed {
                    div {
                        class: "p-2",
                        {conversations.iter().map(|conversation| {
                            let conversation_id = conversation.id.clone();
                            let title = conversation.title.clone();
                            let last_message = conversation.last_message.clone();
                            let timestamp = conversation.timestamp.clone();
                            let pinned = conversation.pinned.unwrap_or(false);

                            let select_id = conversation_id.clone();
                            let delete_id = conversation_id.clone();
                            let class_id = conversation_id.clone();

                            rsx! {
                                div {
                                    key: "{conversation_id}",
                                    class: if Some(class_id) == current_conversation {
                                        "p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 cursor-pointer"
                                    } else {
                                        "p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer transition-colors"
                                    },
                                    onclick: move |_| on_select_conversation.call(select_id.clone()),
                                    div {
                                        class: "flex items-start justify-between",
                                        div {
                                            class: "flex-1 min-w-0",
                                            h4 {
                                                class: "font-medium text-gray-900 dark:text-gray-100 truncate",
                                                "{title}"
                                            }
                                            if let Some(last_msg) = last_message {
                                                p {
                                                    class: "text-sm text-gray-500 dark:text-gray-400 truncate mt-1",
                                                    "{last_msg}"
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex items-center gap-1 ml-2",
                                            if pinned {
                                                span {
                                                    class: "text-yellow-500 text-sm",
                                                    "📌"
                                                }
                                            }
                                            Button {
                                                onclick: move |event: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                                                    event.stop_propagation();
                                                    on_delete_conversation.call(delete_id.clone());
                                                },
                                                class: "w-6 h-6 opacity-0 hover:opacity-100 transition-opacity",
                                                variant: ButtonVariant::Ghost,
                                                size: "sm",
                                                "×"
                                            }
                                        }
                                    }
                                    if let Some(ts) = timestamp {
                                        div {
                                            class: "text-xs text-gray-400 dark:text-gray-500 mt-2",
                                            "{ts}"
                                        }
                                    }
                                }
                            }
                        })}
                    }
                } else {
                    div {
                        class: "p-2",
                        {conversations.iter().map(|conversation| {
                            let conversation_id = conversation.id.clone();
                            let title_char = conversation.title.chars().next().unwrap_or('C');

                            let select_id = conversation_id.clone();
                            let class_id = conversation_id.clone();

                            rsx! {
                                Button {
                                    key: "{conversation_id}",
                                    onclick: move |_| on_select_conversation.call(select_id.clone()),
                                    class: if Some(class_id) == current_conversation {
                                        "w-full p-2 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800"
                                    } else {
                                        "w-full p-2 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                                    },
                                    variant: ButtonVariant::Ghost,
                                    Avatar {
                                        size: AvatarImageSize::Small,
                                        AvatarFallback {
                                            class: "bg-gray-300 dark:bg-gray-600",
                                            "{title_char}"
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
