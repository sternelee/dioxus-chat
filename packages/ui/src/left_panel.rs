use crate::components::{
    button::{Button, ButtonVariant},
    dropdown_menu::{
        DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator,
        DropdownMenuTrigger,
    },
    input::Input,
    separator::Separator,
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};
use crate::thread_list::{Thread, ThreadList};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct LeftPanelProps {
    pub threads: Vec<Thread>,
    pub current_thread: Option<String>,
    pub current_view: LeftPanelView,
    pub collapsed: Option<bool>,
    pub on_select_thread: EventHandler<String>,
    pub on_new_thread: EventHandler,
    pub on_delete_thread: EventHandler<String>,
    pub on_rename_thread: EventHandler<(String, String)>,
    pub on_toggle_favorite: EventHandler<String>,
    pub on_change_view: EventHandler<LeftPanelView>,
    pub on_search_change: EventHandler<String>,
    pub on_clear_all_threads: EventHandler,
    pub on_import_threads: EventHandler,
    pub on_export_threads: EventHandler,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LeftPanelView {
    Threads,
    Assistants,
    Settings,
}

#[component]
pub fn LeftPanel(props: LeftPanelProps) -> Element {
    let collapsed = props.collapsed.unwrap_or(false);
    let current_view = props.current_view.clone();
    let search_term = use_signal(String::new);

    rsx! {
        div {
            class: if collapsed {
                "w-16 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full"
            } else {
                "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full"
            },

            // Header with main menu
            div {
                class: "p-4 border-b border-gray-200 dark:border-gray-700",
                if !collapsed {
                    div {
                        class: "space-y-3",

                        // Logo and title
                        div {
                            class: "flex items-center gap-3",
                            div {
                                class: "w-8 h-8 bg-blue-500 rounded-lg flex items-center justify-center text-white font-bold",
                                "D"
                            }
                            div {
                                class: "flex-1",
                                h1 {
                                    class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                                    "Dioxus Chat"
                                }
                                p {
                                    class: "text-xs text-gray-500 dark:text-gray-400",
                                    "AI Assistant"
                                }
                            }
                        }

                        // Search
                        div {
                            class: "relative",
                            Input {
                                placeholder: match current_view {
                                    LeftPanelView::Threads => "Search conversations...",
                                    LeftPanelView::Assistants => "Search assistants...",
                                    LeftPanelView::Settings => "Search settings...",
                                },
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
                    }
                } else {
                    // Collapsed logo
                    div {
                        class: "flex justify-center",
                        div {
                            class: "w-8 h-8 bg-blue-500 rounded-lg flex items-center justify-center text-white font-bold",
                            "D"
                        }
                    }
                }
            }

            // Navigation tabs
            if !collapsed {
                div {
                    class: "px-4 py-2 border-b border-gray-200 dark:border-gray-700",
                    div {
                        class: "flex space-x-1",
                        Button {
                            onclick: move |_| props.on_change_view.call(LeftPanelView::Threads),
                            class: "flex-1 justify-start gap-2",
                            variant: if matches!(current_view, LeftPanelView::Threads) {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Ghost
                            },
                            "💬 Threads"
                        }
                        Button {
                            onclick: move |_| props.on_change_view.call(LeftPanelView::Assistants),
                            class: "flex-1 justify-start gap-2",
                            variant: if matches!(current_view, LeftPanelView::Assistants) {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Ghost
                            },
                            "🤖 Assistants"
                        }
                        Button {
                            onclick: move |_| props.on_change_view.call(LeftPanelView::Settings),
                            class: "flex-1 justify-start gap-2",
                            variant: if matches!(current_view, LeftPanelView::Settings) {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Ghost
                            },
                            "⚙️ Settings"
                        }
                    }
                }
            }

            // Content area
            div {
                class: "flex-1 overflow-hidden",

                match current_view {
                    LeftPanelView::Threads => rsx! {
                        ThreadsView {
                            threads: props.threads.clone(),
                            current_thread: props.current_thread.clone(),
                            collapsed,
                            on_select_thread: props.on_select_thread,
                            on_new_thread: props.on_new_thread,
                            on_delete_thread: props.on_delete_thread,
                            on_rename_thread: props.on_rename_thread,
                            on_toggle_favorite: props.on_toggle_favorite,
                            on_clear_all_threads: props.on_clear_all_threads,
                            on_import_threads: props.on_import_threads,
                            on_export_threads: props.on_export_threads,
                        }
                    },
                    LeftPanelView::Assistants => rsx! {
                        AssistantsView {
                            collapsed,
                        }
                    },
                    LeftPanelView::Settings => rsx! {
                        SettingsView {
                            collapsed,
                            on_change_view: props.on_change_view,
                        }
                    },
                }
            }

            // Footer with actions
            if !collapsed {
                div {
                    class: "p-4 border-t border-gray-200 dark:border-gray-700",
                    div {
                        class: "space-y-2",
                        Button {
                            onclick: move |_| props.on_new_thread.call(()),
                            class: "w-full justify-start gap-2",
                            variant: ButtonVariant::Primary,
                            "➕ New Thread"
                        }

                        DropdownMenu {
                            DropdownMenuTrigger {
                                Button {
                                    class: "w-full justify-start gap-2",
                                    variant: ButtonVariant::Ghost,
                                    "⋯ More"
                                }
                            }
                            DropdownMenuContent {
                                DropdownMenuItem::<String> {
                                    value: "import".to_string(),
                                    index: 0usize,
                                    on_select: move |_: String| props.on_import_threads.call(()),
                                    "📥 Import Threads"
                                }
                                DropdownMenuItem::<String> {
                                    value: "export".to_string(),
                                    index: 1usize,
                                    on_select: move |_: String| props.on_export_threads.call(()),
                                    "📤 Export Threads"
                                }
                                DropdownMenuSeparator {}
                                DropdownMenuItem::<String> {
                                    value: "clear".to_string(),
                                    index: 2usize,
                                    on_select: move |_: String| props.on_clear_all_threads.call(()),
                                    class: "text-red-600 dark:text-red-400",
                                    "🗑️ Clear All Threads"
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
fn ThreadsView(
    threads: Vec<Thread>,
    current_thread: Option<String>,
    collapsed: bool,
    on_select_thread: EventHandler<String>,
    on_new_thread: EventHandler,
    on_delete_thread: EventHandler<String>,
    on_rename_thread: EventHandler<(String, String)>,
    on_toggle_favorite: EventHandler<String>,
    on_clear_all_threads: EventHandler,
    on_import_threads: EventHandler,
    on_export_threads: EventHandler,
) -> Element {
    rsx! {
        ThreadList {
            threads,
            current_thread,
            on_select_thread,
            on_new_thread,
            on_delete_thread,
            on_rename_thread,
            on_toggle_favorite,
            collapsed,
        }
    }
}

#[component]
fn AssistantsView(collapsed: bool) -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-y-auto p-4",
            if !collapsed {
                div {
                    class: "space-y-4",
                    div {
                        class: "text-center py-8",
                        div {
                            class: "text-4xl mb-2",
                            "🤖"
                        }
                        h3 {
                            class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-2",
                            "Assistants"
                        }
                        p {
                            class: "text-sm text-gray-500 dark:text-gray-400",
                            "Create and manage AI assistants for specific tasks"
                        }
                        Button {
                            class: "mt-4",
                            "+ Create Assistant"
                        }
                    }
                }
            } else {
                div {
                    class: "flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400",
                    div {
                        class: "text-2xl",
                        "🤖"
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsView(collapsed: bool, on_change_view: EventHandler<LeftPanelView>) -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-y-auto",
            if !collapsed {
                div {
                    class: "p-4 space-y-2",
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        "🎨 Appearance"
                    }
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        "🔗 Data Sources"
                    }
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        "🧠 Models"
                    }
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        "⚡ Performance"
                    }
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        "🔒 Privacy"
                    }
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        "📊 Analytics"
                    }
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-full justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        "❓ About"
                    }
                }
            } else {
                div {
                    class: "flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400 space-y-3",
                    div {
                        class: "text-2xl",
                        "⚙️"
                    }
                    Button {
                        onclick: move |_| on_change_view.call(LeftPanelView::Settings),
                        class: "w-10 h-10 p-0",
                        variant: ButtonVariant::Ghost,
                        title: "Settings",
                        "⚙️"
                    }
                }
            }
        }
    }
}

