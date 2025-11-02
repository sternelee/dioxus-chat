use crate::components::{
    button::{Button, ButtonVariant},
    input::Input,
};
use crate::{
    left_panel::{LeftPanel, LeftPanelView},
    thread_content::{ThreadContent, ThreadChatMessage, MessageRole, ToolCall, ToolCallStatus},
    thread_list::Thread,
    settings_panel::{SettingsPanel, SettingsTab, SettingsModel},
    chat_input::ChatInput,
    settings_menu::{SettingsMenu, Theme},
};
use dioxus::prelude::*;
use api::{AgentConfig, GooseMode};

#[derive(Clone, PartialEq, Props)]
pub struct ChatAppProps {
    pub threads: Vec<Thread>,
    pub current_thread: Option<String>,
    pub messages: Vec<ThreadChatMessage>,
    pub streaming_content: Option<String>,
    pub settings_open: bool,
    pub settings_tab: SettingsTab,
    pub theme: Option<Theme>,
    pub models: Vec<SettingsModel>,
    pub selected_model: Option<String>,
    pub agent_config: Option<AgentConfig>,
}

#[component]
pub fn ChatApp(props: ChatAppProps) -> Element {
    let mut left_panel_collapsed = use_signal(|| false);
    let mut left_panel_view = use_signal(|| LeftPanelView::Threads);
    let mut user_input = use_signal(|| String::new());

    rsx! {
        div {
            class: "flex h-screen bg-gray-50 dark:bg-gray-900",

            // Left Panel
            LeftPanel {
                threads: props.threads.clone(),
                current_thread: props.current_thread.clone(),
                current_view: left_panel_view.read().clone(),
                collapsed: *left_panel_collapsed.read(),
                on_select_thread: move |thread_id| {
                    // TODO: Handle thread selection
                },
                on_new_thread: move |_| {
                    // TODO: Handle new thread creation
                },
                on_delete_thread: move |thread_id| {
                    // TODO: Handle thread deletion
                },
                on_rename_thread: move |(thread_id, new_name)| {
                    // TODO: Handle thread renaming
                },
                on_toggle_favorite: move |thread_id| {
                    // TODO: Handle favorite toggle
                },
                on_change_view: move |view| {
                    left_panel_view.set(view);
                },
                on_search_change: move |search_term| {
                    // TODO: Handle search
                },
                on_clear_all_threads: move |_| {
                    // TODO: Handle clear all threads
                },
                on_import_threads: move |_| {
                    // TODO: Handle import threads
                },
                on_export_threads: move |_| {
                    // TODO: Handle export threads
                },
            }

            // Main Chat Area
            div {
                class: "flex-1 flex flex-col",

                // Header
                div {
                    class: "bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            class: "flex items-center gap-4",
                            Button {
                                onclick: move |_| left_panel_collapsed.set(!*left_panel_collapsed.read()),
                                variant: ButtonVariant::Ghost,
                                class: "w-8 h-8 p-0",
                                {if *left_panel_collapsed.read() { "☰" } else { "✕" }}
                            }
                            h1 {
                                class: "text-xl font-semibold text-gray-900 dark:text-gray-100",
                                "Dioxus Chat"
                            }
                        }
                        div {
                            class: "flex items-center gap-2",
                            Button {
                                onclick: move |_| {
                                    // TODO: Open settings
                                },
                                variant: ButtonVariant::Ghost,
                                "⚙️"
                            }
                        }
                    }
                }

                // Messages Area
                div {
                    class: "flex-1 overflow-y-auto",
                    ThreadContent {
                        messages: props.messages.clone(),
                        streaming_content: props.streaming_content.clone(),
                        on_copy_message: move |text| {
                            // TODO: Handle copy message
                        },
                        on_regenerate_response: move |message_id| {
                            // TODO: Handle regenerate response
                        },
                        on_edit_message: move |(message_id, new_content)| {
                            // TODO: Handle edit message
                        },
                        on_delete_message: move |message_id| {
                            // TODO: Handle delete message
                        },
                        is_last_message_streaming: props.streaming_content.is_some(),
                    }
                }

                // Chat Input
                div {
                    class: "bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-6 py-4",
                    ChatInput {
                        value: user_input.read().clone(),
                        oninput: move |evt| user_input.set(evt.value()),
                        on_submit: move |text| {
                            // TODO: Handle message submission
                            user_input.set(String::new());
                        },
                        placeholder: "Type your message...",
                        disabled: false,
                        streaming: props.streaming_content.is_some(),
                    }
                }
            }
        }

        // Settings Modal
        SettingsPanel {
            open: props.settings_open,
            on_open_change: move |open| {
                // TODO: Handle settings open/close
            },
            active_tab: props.settings_tab.clone(),
            on_tab_change: move |tab| {
                // TODO: Handle settings tab change
            },
            models: props.models.clone(),
            selected_model: props.selected_model.clone(),
            on_select_model: move |model_id| {
                // TODO: Handle model selection
            },
            theme: props.theme.clone(),
            on_theme_change: move |theme| {
                // TODO: Handle theme change
            },
            agent_config: props.agent_config.clone(),
            on_agent_config_change: move |config| {
                // TODO: Handle agent config change
            },
        }
    }
}

// Example usage function
pub fn example_chat_app() -> Element {
    let sample_threads = vec![
        Thread {
            id: "1".to_string(),
            title: "Getting Started with Rust".to_string(),
            last_message: Some("How do I get started with Rust programming?".to_string()),
            timestamp: Some("2 hours ago".to_string()),
            favorite: true,
            model_name: Some("gpt-4".to_string()),
            provider_name: Some("OpenAI".to_string()),
            message_count: 5,
        },
        Thread {
            id: "2".to_string(),
            title: "Dioxus Framework Discussion".to_string(),
            last_message: Some("What are the benefits of using Dioxus?".to_string()),
            timestamp: Some("1 day ago".to_string()),
            favorite: false,
            model_name: Some("claude-3".to_string()),
            provider_name: Some("Anthropic".to_string()),
            message_count: 12,
        },
    ];

    let sample_messages = vec![
        ThreadChatMessage {
            id: "msg1".to_string(),
            role: MessageRole::User,
            content: "Hello! Can you help me understand how to build a chat application with Dioxus?".to_string(),
            timestamp: Some("10:30 AM".to_string()),
            avatar_url: None,
            model_name: None,
            is_streaming: Some(false),
            reasoning_content: None,
            tool_calls: None,
            metadata: None,
        },
        ThreadChatMessage {
            id: "msg2".to_string(),
            role: MessageRole::Assistant,
            content: "I'd be happy to help you build a chat application with Dioxus! Dioxus is a modern Rust framework for building user interfaces that compiles to multiple platforms. Here's what you need to know:

## Getting Started

1. **Install Dioxus CLI**:
```bash
cargo install dioxus-cli
```

2. **Create a new project**:
```bash
dx create my-chat-app
cd my-chat-app
```

3. **Key Components for a Chat App**:
- **ThreadList**: Display conversation list
- **ThreadContent**: Show messages in a thread
- **ChatInput**: Handle user input
- **LeftPanel**: Navigation and settings

4. **Essential Features**:
- Real-time message streaming
- Multiple model support
- Theme switching
- Settings management

Would you like me to dive deeper into any specific aspect of building the chat application?".to_string(),
            timestamp: Some("10:31 AM".to_string()),
            avatar_url: None,
            model_name: Some("gpt-4".to_string()),
            is_streaming: Some(false),
            reasoning_content: None,
            tool_calls: None,
            metadata: None,
        },
    ];

    let sample_models = vec![
        SettingsModel {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
            description: Some("Most capable GPT-4 model".to_string()),
            capabilities: vec!["chat".to_string(), "tools".to_string(), "vision".to_string()],
            context_limit: Some(128000),
            supports_tools: true,
            supports_streaming: true,
        },
        SettingsModel {
            id: "claude-3".to_string(),
            name: "Claude 3".to_string(),
            provider: "Anthropic".to_string(),
            description: Some("Advanced Claude model".to_string()),
            capabilities: vec!["chat".to_string(), "tools".to_string(), "analysis".to_string()],
            context_limit: Some(200000),
            supports_tools: true,
            supports_streaming: true,
        },
    ];

    let sample_agent_config = AgentConfig {
        max_iterations: 10,
        require_confirmation: false,
        readonly_tools: vec![],
        enable_tool_inspection: true,
        enable_auto_compact: true,
        compact_threshold: 0.8,
        max_turns_without_tools: 3,
        enable_autopilot: false,
        enable_extensions: true,
        extension_timeout: 30,
        goose_mode: GooseMode::Agent,
    };

    rsx! {
        ChatApp {
            threads: sample_threads,
            current_thread: Some("1".to_string()),
            messages: sample_messages,
            streaming_content: None,
            settings_open: false,
            settings_tab: SettingsTab::General,
            theme: Some(Theme::Auto),
            models: sample_models,
            selected_model: Some("gpt-4".to_string()),
            agent_config: Some(sample_agent_config),
        }
    }
}