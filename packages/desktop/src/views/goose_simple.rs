use dioxus::prelude::*;
use api::Role;

/// Simple Goose Chat component demonstrating the UI concept
#[component]
pub fn GooseSimpleChat() -> Element {
    let mut messages = use_signal(Vec::<SimpleMessage>::new);
    let mut input = use_signal(String::new);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "flex h-screen bg-gray-50 dark:bg-gray-900",
            // Main chat area
            div { class: "flex-1 flex flex-col h-full",
                // Header
                div { class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 p-4",
                    h1 { class: "text-xl font-semibold text-gray-900 dark:text-gray-100", "Goose Simple Chat" }
                    p { class: "text-sm text-gray-600 dark:text-gray-400 mt-1", "A simplified chat interface demonstrating Goose Agent concepts" }
                }

                // Messages area
                div { class: "flex-1 overflow-y-auto p-4",
                    if messages.read().is_empty() {
                        div { class: "text-center text-gray-500 dark:text-gray-400 mt-8",
                            h3 { class: "text-lg font-medium mb-2", "Welcome to Goose Chat!" }
                            p { "This demonstrates the key concepts from Goose Agent:" }
                            ul { class: "text-left mt-4 space-y-2 max-w-md mx-auto",
                                li { "🧠 **Thinking content** - Shows reasoning process" }
                                li { "🔧 **Tool calls** - Displays function execution" }
                                li { "✅ **Tool results** - Shows execution outcomes" }
                                li { "🤖 **Multiple providers** - DeepSeek, OpenRouter, etc." }
                            }
                            p { class: "mt-4", "Try typing 'think about something' or 'hello' to see different responses!" }
                        }
                    } else {
                        {messages.read().iter().map(|message| {
                            rsx! {
                                SimpleMessageBubble {
                                    message: message.clone(),
                                }
                            }
                        })}
                    }

                    if *loading.read() {
                        div { class: "flex justify-start mb-4",
                            div { class: "bg-gray-200 dark:bg-gray-700 rounded-lg px-4 py-2",
                                div { class: "flex space-x-1",
                                    div { class: "w-2 h-2 bg-gray-400 rounded-full animate-bounce" }
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

                // Input area
                div { class: "border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 p-4",
                    div { class: "flex space-x-2",
                        input {
                            r#type: "text",
                            class: "flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "Type your message...",
                            value: "{input.read()}",
                            oninput: move |evt| {
                                input.set(evt.value().clone());
                            },
                            disabled: *loading.read()
                        }

                        button {
                            class: "px-4 py-2 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white rounded-lg transition-colors",
                            onclick: move |_| {
                                if input.read().trim().is_empty() || *loading.read() {
                                    return;
                                }

                                let current_input = input.read().clone();
                                let user_msg = SimpleMessage {
                                    role: Role::User,
                                    content: current_input.clone(),
                                    timestamp: chrono::Utc::now().format("%H:%M").to_string(),
                                    message_type: MessageType::Text,
                                };

                                messages.write().push(user_msg);

                                input.set(String::new());
                                loading.set(true);

                                let mut messages = messages.clone();
                                let mut loading = loading.clone();
                                spawn(async move {
                                    // Simulate thinking delay
                                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

                                    let response = generate_response(&current_input);
                                    messages.write().push(response);
                                    loading.set(false);
                                });
                            },
                            disabled: *loading.read() || input.read().trim().is_empty(),
                            if *loading.read() {
                                "..."
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

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMessage {
    pub role: Role,
    pub content: String,
    pub timestamp: String,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Text,
    Thinking,
    ToolCall,
    ToolResult,
}

/// Generate response based on input (simulating different agent behaviors)
fn generate_response(input: &str) -> SimpleMessage {
    let lower_input = input.to_lowercase();

    if lower_input.contains("think") || lower_input.contains("reason") {
        // Simulate thinking response
        let thinking_content = format!(
            "Let me think through this step by step:\n\n1. **Understanding**: You're asking about \"{}\"\n2. **Analysis**: This requires careful consideration of multiple aspects\n3. **Approach**: I'll break this down into manageable components\n4. **Solution**: Based on my analysis, here's my thoughtful response\n\nThis demonstrates how reasoning models like DeepSeek R1 process information systematically before providing answers.",
            input
        );

        SimpleMessage {
            role: Role::Assistant,
            content: thinking_content,
            timestamp: chrono::Utc::now().format("%H:%M").to_string(),
            message_type: MessageType::Thinking,
        }
    } else if lower_input.contains("tool") || lower_input.contains("execute") {
        // Simulate tool call response
        let tool_response = "🔧 **Tool Call**: `search_information`\n\n**Arguments**:\n```json\n{{\n  \"query\": \"{}\",\n  \"sources\": [\"web\", \"documents\"]\n}}\n```\n\n✅ **Tool Result**: Found 5 relevant documents\n- Document 1: Overview of the topic\n- Document 2: Detailed analysis\n- Document 3: Implementation guide\n- Document 4: Best practices\n- Document 5: Related resources\n\nThis demonstrates how agents can use tools to gather and process information.".to_string();

        SimpleMessage {
            role: Role::Assistant,
            content: tool_response,
            timestamp: chrono::Utc::now().format("%H:%M").to_string(),
            message_type: MessageType::ToolCall,
        }
    } else if lower_input.contains("hello") || lower_input.contains("hi") {
        SimpleMessage {
            role: Role::Assistant,
            content: "Hello! I'm a Goose Agent-inspired chat interface. 🦢\n\nI demonstrate key concepts from the Goose Agent architecture:\n\n• **Agent-based design** - Modular, extensible architecture\n• **Multi-provider support** - DeepSeek, OpenRouter, Anthropic, OpenAI, Google\n• **Thinking capabilities** - Step-by-step reasoning process\n• **Tool integration** - Function calling and execution\n• **Streaming responses** - Real-time message updates\n\nTry asking me to 'think about' something or 'use tools' to see these features in action!".to_string(),
            timestamp: chrono::Utc::now().format("%H:%M").to_string(),
            message_type: MessageType::Text,
        }
    } else {
        // Default response
        SimpleMessage {
            role: Role::Assistant,
            content: format!(
                "I understand you're interested in: \"{}\"\n\nThis response comes from a simplified Goose Agent implementation. In a full implementation, this would:\n\n1. **Select appropriate model** based on your request\n2. **Process through extensions** for specialized handling\n3. **Use tools** if needed for information gathering\n4. **Think step-by-step** for complex queries\n5. **Stream the response** in real-time\n\nThe actual Goose Agent supports multiple AI providers and can intelligently route your requests to the most suitable model.",
                input
            ),
            timestamp: chrono::Utc::now().format("%H:%M").to_string(),
            message_type: MessageType::Text,
        }
    }
}

/// Simple message bubble component
#[component]
fn SimpleMessageBubble(message: SimpleMessage) -> Element {
    let is_user = matches!(message.role, Role::User);

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

                div { class: "font-semibold text-sm mb-1",
                    if is_user { "You" } else {
                        match message.message_type {
                            MessageType::Thinking => "🧠 Assistant (Thinking)",
                            MessageType::ToolCall => "🔧 Assistant (Tools)",
                            MessageType::ToolResult => "✅ Assistant (Results)",
                            MessageType::Text => "🤖 Assistant",
                        }
                    }
                }

                // Message content with appropriate formatting
                match message.message_type {
                    MessageType::Thinking => {
                        rsx! {
                            div { class: "text-sm",
                                pre { class: "whitespace-pre-wrap font-sans text-purple-800 dark:text-purple-200", "{message.content}" }
                            }
                        }
                    }
                    MessageType::ToolCall => {
                        rsx! {
                            div { class: "text-sm",
                                div { class: "mb-2 p-2 bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-md",
                                    pre { class: "whitespace-pre-wrap font-mono text-orange-800 dark:text-orange-200 text-xs", "{message.content}" }
                                }
                            }
                        }
                    }
                    _ => {
                        rsx! {
                            div { class: "text-sm",
                                if message.content.contains('\n') {
                                    pre { class: "whitespace-pre-wrap font-sans", "{message.content}" }
                                } else {
                                    p { "{message.content}" }
                                }
                            }
                        }
                    }
                }

                // Timestamp
                div { class: "text-xs opacity-75 mt-1", "{message.timestamp}" }
            }
        }
    }
}