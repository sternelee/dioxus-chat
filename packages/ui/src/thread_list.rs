use crate::components::{
    avatar::{Avatar, AvatarFallback, AvatarImageSize},
    button::{Button, ButtonVariant},
    dropdown_menu::{
        DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator,
        DropdownMenuTrigger,
    },
    input::Input,
    separator::Separator,
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct ThreadListProps {
    pub threads: Vec<Thread>,
    pub current_thread: Option<String>,
    pub on_select_thread: EventHandler<String>,
    pub on_new_thread: EventHandler,
    pub on_delete_thread: EventHandler<String>,
    pub on_rename_thread: EventHandler<(String, String)>,
    pub on_toggle_favorite: EventHandler<String>,
    pub search_term: Option<String>,
    pub on_search_change: EventHandler<String>,
    pub collapsed: Option<bool>,
}

#[derive(Clone, PartialEq)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub last_message: Option<String>,
    pub timestamp: Option<String>,
    pub favorite: bool,
    pub model_name: Option<String>,
    pub provider_name: Option<String>,
    pub message_count: usize,
}

#[component]
pub fn ThreadList(props: ThreadListProps) -> Element {
    let collapsed = props.collapsed.unwrap_or(false);
    let threads = props.threads.clone();
    let current_thread = props.current_thread.clone();
    let search_term = use_signal(|| props.search_term.unwrap_or_default());

    // Filter threads based on search term
    let filtered_threads = use_memo(move || {
        let term = search_term.read().clone();
        if term.is_empty() {
            threads.clone()
        } else {
            threads
                .iter()
                .filter(|thread| {
                    thread.title.to_lowercase().contains(&term.to_lowercase())
                        || thread.last_message.as_ref().map_or(false, |msg| {
                            msg.to_lowercase().contains(&term.to_lowercase())
                        })
                })
                .cloned()
                .collect()
        }
    });

    // Separate favorite and regular threads
    let favorite_threads = use_memo(move || {
        filtered_threads
            .read()
            .iter()
            .filter(|thread| thread.favorite)
            .cloned()
            .collect::<Vec<_>>()
    });

    let regular_threads = use_memo(move || {
        filtered_threads
            .read()
            .iter()
            .filter(|thread| !thread.favorite)
            .cloned()
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: if collapsed {
                "w-16 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full"
            } else {
                "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full"
            },

            // Header with search
            div {
                class: "p-4 border-b border-gray-200 dark:border-gray-700 space-y-3",

                if !collapsed {
                    // New chat button
                    Button {
                        onclick: move |_| props.on_new_thread.call(()),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Primary,
                        "+ {if props.search_term.is_some() { " New Chat " } else { "New Chat " }}"
                    }

                    // Search input
                    div {
                        class: "relative",
                        Input {
                            placeholder: "Search conversations...",
                            value: "{search_term}",
                            oninput: move |evt| {
                                let value = evt.value();
                                search_term.set(value.clone());
                                props.on_search_change.call(value);
                            },
                            class: "w-full pl-9",
                        }
                        div {
                            class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400",
                            "🔍"
                        }
                    }
                } else {
                    // Collapsed new chat button
                    Button {
                        onclick: move |_| props.on_new_thread.call(()),
                        class: "w-full p-2 justify-center",
                        variant: ButtonVariant::Ghost,
                        "+"
                    }
                }
            }

            // Thread list
            div {
                class: "flex-1 overflow-y-auto",

                if !collapsed {
                    div {
                        class: "space-y-2 p-2",

                        // Favorite threads section
                        if !favorite_threads.read().is_empty() {
                            div {
                                class: "space-y-1",
                                div {
                                    class: "px-3 py-1 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "Starred"
                                }
                                Separator {}
                                {favorite_threads.read().iter().map(|thread| {
                                    rsx!(ThreadItem {
                                        thread: thread.clone(),
                                        current_thread: current_thread.clone(),
                                        on_select: props.on_select_thread,
                                        on_delete: props.on_delete_thread,
                                        on_rename: props.on_rename_thread,
                                        on_toggle_favorite: props.on_toggle_favorite,
                                    })
                                })}
                            }
                        }

                        // Regular threads section
                        if !regular_threads.read().is_empty() {
                            div {
                                class: "space-y-1",
                                if !favorite_threads.read().is_empty() {
                                    div {
                                        class: "px-3 py-1 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                        "Recent"
                                    }
                                    Separator {}
                                }
                                {regular_threads.read().iter().map(|thread| {
                                    rsx!(ThreadItem {
                                        thread: thread.clone(),
                                        current_thread: current_thread.clone(),
                                        on_select: props.on_select_thread,
                                        on_delete: props.on_delete_thread,
                                        on_rename: props.on_rename_thread,
                                        on_toggle_favorite: props.on_toggle_favorite,
                                    })
                                })}
                            }
                        }

                        // Empty state
                        if filtered_threads.read().is_empty() {
                            div {
                                class: "flex flex-col items-center justify-center py-8 text-gray-500 dark:text-gray-400",
                                div {
                                    class: "text-4xl mb-2",
                                    "💬"
                                }
                                p {
                                    class: "text-sm text-center",
                                    if search_term.read().is_empty() {
                                        "No conversations yet"
                                    } else {
                                        "No conversations found"
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Collapsed view
                    div {
                        class: "p-2 space-y-1",
                        {filtered_threads.read().iter().map(|thread| {
                            let thread_id = thread.id.clone();
                            let title_char = thread.title.chars().next().unwrap_or('C');
                            let is_current = Some(thread_id.clone()) == current_thread;

                            rsx! {
                                Button {
                                    key: "{thread_id}",
                                    onclick: move |_| props.on_select_thread.call(thread_id.clone()),
                                    class: if is_current {
                                        "w-full p-2 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800"
                                    } else {
                                        "w-full p-2 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                                    },
                                    variant: ButtonVariant::Ghost,
                                    title: "{thread.title}",
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

#[component]
fn ThreadItem(
    thread: Thread,
    current_thread: Option<String>,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_rename: EventHandler<(String, String)>,
    on_toggle_favorite: EventHandler<String>,
) -> Element {
    let is_current = Some(thread.id.clone()) == current_thread;
    let show_dropdown = use_signal(|| false);

    rsx! {
        div {
            class: if is_current {
                "p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 cursor-pointer group"
            } else {
                "p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer transition-colors group"
            },
            onclick: move |_| on_select.call(thread.id.clone()),

            div {
                class: "flex items-start justify-between",

                // Thread content
                div {
                    class: "flex-1 min-w-0",

                    // Title with favorite indicator
                    div {
                        class: "flex items-center gap-2",
                        h4 {
                            class: "font-medium text-gray-900 dark:text-gray-100 truncate",
                            "{thread.title}"
                        }
                        if thread.favorite {
                            span {
                                class: "text-yellow-500 text-sm",
                                title: "Starred",
                                "⭐"
                            }
                        }
                    }

                    // Model and provider info
                    if let Some(model_name) = &thread.model_name {
                        div {
                            class: "flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400 mt-1",
                            if let Some(provider) = &thread.provider_name {
                                span {
                                    class: "bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded",
                                    "{provider}"
                                }
                            }
                            span {
                                class: "bg-blue-100 dark:bg-blue-900 px-1.5 py-0.5 rounded text-blue-700 dark:text-blue-300",
                                "{model_name}"
                            }
                        }
                    }

                    // Last message
                    if let Some(last_msg) = &thread.last_message {
                        p {
                            class: "text-sm text-gray-500 dark:text-gray-400 truncate mt-1",
                            "{last_msg}"
                        }
                    }

                    // Timestamp and message count
                    div {
                        class: "flex items-center justify-between text-xs text-gray-400 dark:text-gray-500 mt-2",
                        if let Some(ts) = &thread.timestamp {
                            "{ts}"
                        }
                        if thread.message_count > 0 {
                            span {
                                class: "bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded",
                                "{thread.message_count}"
                            }
                        }
                    }
                }

                // Action buttons
                div {
                    class: "flex items-center gap-1 ml-2 opacity-0 group-hover:opacity-100 transition-opacity",

                    // Favorite toggle
                    Button {
                        onclick: move |event| {
                            event.stop_propagation();
                            on_toggle_favorite.call(thread.id.clone());
                        },
                        class: "w-6 h-6 p-0",
                        variant: ButtonVariant::Ghost,
                        size: "sm",
                        title: if thread.favorite { "Remove from favorites" } else { "Add to favorites" },
                        "{if thread.favorite { '⭐' } else { '☆' }}"
                    }

                    // More options dropdown
                    DropdownMenu {
                        DropdownMenuTrigger {
                            Button {
                                onclick: move |event| {
                                    event.stop_propagation();
                                    show_dropdown.set(true);
                                },
                                class: "w-6 h-6 p-0",
                                variant: ButtonVariant::Ghost,
                                size: "sm",
                                "⋯"
                            }
                        }
                        DropdownMenuContent {
                            DropdownMenuItem::<String> {
                                value: "favorite".to_string(),
                                index: 0usize,
                                on_select: move |_: String| {
                                    on_toggle_favorite.call(thread.id.clone());
                                    show_dropdown.set(false);
                                },
                                "{if thread.favorite { 'Remove from favorites' } else { 'Add to favorites' }}"
                            }
                            DropdownMenuItem::<String> {
                                value: "rename".to_string(),
                                index: 1usize,
                                on_select: move |_: String| {
                                    // TODO: Implement rename functionality
                                    show_dropdown.set(false);
                                },
                                "Rename"
                            }
                            DropdownMenuSeparator {}
                            DropdownMenuItem::<String> {
                                value: "delete".to_string(),
                                index: 2usize,
                                on_select: move |_: String| {
                                    on_delete.call(thread.id.clone());
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
}

