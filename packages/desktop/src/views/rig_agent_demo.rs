// Rig Agent Demo Page - Complete Integration Test
use dioxus::prelude::*;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agent::{
    Agent, AgentFactory, Conversation, UiChatMessage, MessageContent, Role, AgentConfig,
    GooseMode, Tool, RigAgentService,
};
use api::AgentFactory as ApiAgentFactory;

use ui::{
    AgentConfigPanel, ToolManager, AgentStatus, AgentConfigState, StreamingChatContainer,
    StreamingControls, StreamingMessage, StreamingState, EnhancedStreamChunk, ChunkType,
};

/// State for the Rig Agent Demo
#[derive(Clone)]
pub struct RigAgentDemoState {
    pub agent: Arc<RwLock<Box<dyn Agent>>>,
    pub conversation: Option<Conversation>,
    pub config: AgentConfigState,
    pub available_tools: Vec<Tool>,
    pub selected_tools: Vec<String>,
    pub messages: Vec<StreamingMessage>,
    pub streaming_state: StreamingState,
    pub current_input: String,
    pub is_config_panel_open: bool,
    pub is_tools_panel_open: bool,
    pub current_model: String,
}

impl Default for RigAgentDemoState {
    fn default() -> Self {
        Self {
            agent: Arc::new(RwLock::new(
                async_std::task::block_on(async {
                    AgentFactory::create_default_agent().await.unwrap()
                })
            )),
            conversation: None,
            config: AgentConfigState::default(),
            available_tools: vec![],
            selected_tools: vec![],
            messages: vec![],
            streaming_state: StreamingState::Idle,
            current_input: String::new(),
            is_config_panel_open: false,
            is_tools_panel_open: false,
            current_model: "mock-local".to_string(),
        }
    }
}

/// Main Rig Agent Demo component
#[component]
pub fn RigAgentDemo() -> Element {
    let mut state = use_signal(RigAgentDemoState::default);
    let messages = use_signal(Vec::<StreamingMessage>::new);

    // Initialize agent and load available tools
    use_coroutine(|_| {
        let state = state.clone();
        let messages = messages.clone();
        async move {
            if let Err(e) = initialize_demo(&mut state.write(), &messages).await {
                state.write().streaming_state = StreamingState::Error(format!("Initialization failed: {}", e));
            }
        }
    });

    rsx! {
        div { class: "flex h-screen bg-gray-50 dark:bg-gray-900",
            // Sidebar with controls
            div { class: "w-80 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex flex-col h-full overflow-y-auto",
                DemoSidebar {
                    state: state.clone(),
                    messages: messages.clone(),
                }
            }

            // Main chat area
            div { class: "flex-1 flex flex-col h-full",
                DemoHeader {
                    state: state.clone(),
                }

                StreamingChatContainer {
                    messages: messages.clone(),
                    on_send_message: move |message: String| {
                        spawn(handle_send_message(state.clone(), messages.clone(), message));
                    },
                    streaming_state: state.clone().map(|s| s.streaming_state.clone()),
                    current_input: state.clone().map(|s| s.current_input.clone()),
                    placeholder: Some("Ask the agent anything...".to_string()),
                }
            }

            // Configuration panels (modals/overlays)
            if state.read().is_config_panel_open {
                AgentConfigOverlay {
                    state: state.clone(),
                }
            }

            if state.read().is_tools_panel_open {
                ToolManagerOverlay {
                    state: state.clone(),
                }
            }
        }
    }
}

/// Initialize the demo with tools and agent
async fn initialize_demo(
    state: &mut RigAgentDemoState,
    messages: &Signal<Vec<StreamingMessage>>,
) -> anyhow::Result<()> {
    // Create rig service and get available tools
    let rig_service = RigAgentService::new()?;
    let available_tools = rig_service.list_tools("mock-local").await;

    state.available_tools = available_tools;

    // Add welcome message
    let welcome_msg = StreamingMessage {
        content: "Welcome to the Rig Agent Demo! 🚀\n\nThis demo showcases the complete integration of the Rig AI agent framework with advanced features:\n\n• Multiple agent modes (Chat/Agent/Auto)\n• Tool integration and management\n• Enhanced streaming with thinking process\n• Configurable agent behavior\n\nTry asking the agent to:\n- Tell you the current time\n- Check the weather\n- Help with calculations\n- Use various tools\n\nClick the configuration buttons to customize the agent behavior!".to_string(),
        chunk_type: ChunkType::Content,
        metadata: None,
        timestamp: chrono::Utc::now(),
        is_complete: true,
    };

    messages.set(vec![welcome_msg]);
    state.streaming_state = StreamingState::Idle;

    Ok(())
}

/// Demo sidebar with controls
#[component]
fn DemoSidebar(
    state: Signal<RigAgentDemoState>,
    messages: Signal<Vec<StreamingMessage>>,
) -> Element {
    rsx! {
        div { class: "p-4 space-y-4",
            // Title
            div { class: "text-center pb-4 border-b border-gray-200 dark:border-gray-700",
                h1 { class: "text-xl font-bold text-gray-900 dark:text-gray-100",
                    "Rig Agent Demo"
                }
                p { class: "text-xs text-gray-500 dark:text-gray-400 mt-1",
                    "Complete AI Agent Integration"
                }
            }

            // Quick Actions
            div { class: "space-y-2",
                h3 { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                    "Quick Actions"
                }
                button {
                    class: "w-full px-3 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors text-sm",
                    onclick: move |_| {
                        state.write().is_config_panel_open = true;
                    },
                    "⚙️ Agent Configuration"
                }
                button {
                    class: "w-full px-3 py-2 bg-green-500 hover:bg-green-600 text-white rounded-lg transition-colors text-sm",
                    onclick: move |_| {
                        state.write().is_tools_panel_open = true;
                    },
                    "🔧 Tool Manager"
                }
                button {
                    class: "w-full px-3 py-2 bg-purple-500 hover:bg-purple-600 text-white rounded-lg transition-colors text-sm",
                    onclick: move |_| {
                        spawn(handle_clear_chat(state.clone(), messages.clone()));
                    },
                    "🗑️ Clear Chat"
                }
            }

            // Agent Status
            AgentStatus {
                config: state.read().config.clone(),
                is_active: matches!(*state.read().streaming_state, StreamingState::Idle | StreamingState::Complete),
                current_model: state.read().current_model.clone(),
            }

            // Streaming Controls
            StreamingControls {
                on_toggle_streaming: move |enabled| {
                    // Could be implemented to enable/disable enhanced streaming
                },
                on_clear_history: move |_| {
                    spawn(handle_clear_chat(state.clone(), messages.clone()));
                },
                is_streaming_enabled: true,
            }

            // Test Scenarios
            div { class: "space-y-2",
                h3 { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                    "Test Scenarios"
                }
                div { class="space-y-1",
                    TestScenarioButton {
                        label: "📅 Get Current Time",
                        message: "What time is it now?".to_string(),
                        on_click: move |_| {
                            spawn(handle_send_message(state.clone(), messages.clone(), "What time is it now?".to_string()));
                        }
                    }
                    TestScenarioButton {
                        label: "🌤️ Check Weather",
                        message: "What's the weather like in Beijing?".to_string(),
                        on_click: move |_| {
                            spawn(handle_send_message(state.clone(), messages.clone(), "What's the weather like in Beijing?".to_string()));
                        }
                    }
                    TestScenarioButton {
                        label: "🧮 Calculate 25 * 4",
                        message: "Calculate 25 * 4 for me".to_string(),
                        on_click: move |_| {
                            spawn(handle_send_message(state.clone(), messages.clone(), "Calculate 25 * 4 for me".to_string()));
                        }
                    }
                    TestScenarioButton {
                        label: "🤔 Complex Reasoning",
                        message: "Think step by step: If I have 10 apples and give away 3, then buy 5 more, how many do I have? Show your thinking process.".to_string(),
                        on_click: move |_| {
                            spawn(handle_send_message(state.clone(), messages.clone(), "Think step by step: If I have 10 apples and give away 3, then buy 5 more, how many do I have? Show your thinking process.".to_string()));
                        }
                    }
                }
            }

            // Agent Mode Info
            div { class="mt-4 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg",
                h3 { class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                    "Current Mode"
                }
                div { class="text-xs text-gray-600 dark:text-gray-400",
                    match state.read().config.goose_mode {
                        GooseMode::Chat => "Chat: Natural conversation focused",
                        GooseMode::Agent => "Agent: Tool-enabled assistant",
                        GooseMode::Auto => "Auto: Autonomous agent with initiative",
                    }
                }
            }
        }
    }
}

#[component]
fn TestScenarioButton(
    label: String,
    message: String,
    on_click: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            class: "w-full px-2 py-1 text-left text-xs bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors",
            onclick: move |_| {
                on_click.call(());
            },
            "{label}"
        }
    }
}

/// Demo header with model selection
#[component]
fn DemoHeader(state: Signal<RigAgentDemoState>) -> Element {
    rsx! {
        div { class: "border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 p-4",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center space-x-4",
                    h1 { class: "text-xl font-semibold text-gray-900 dark:text-gray-100", "Rig Agent Demo" }

                    // Model selector
                    select {
                        class: "px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 text-sm",
                        value: "{state.read().current_model}",
                        onchange: move |evt| {
                            state.write().current_model = evt.value.clone();
                        },
                        option { value: "mock-local", "Mock Local" }
                        option { value: "deepseek-chat", "DeepSeek Chat" }
                        option { value: "deepseek-r1-distill-llama-70b", "DeepSeek R1" }
                        option { value: "openai/gpt-4o", "GPT-4o" }
                        option { value: "anthropic/claude-3.5-sonnet", "Claude 3.5" }
                        option { value: "google/gemini-1.5-pro", "Gemini Pro" }
                    }
                }

                // Status indicator
                div { class: "flex items-center space-x-2",
                    match *state.read().streaming_state {
                        StreamingState::Idle => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-green-500 rounded-full" }
                                span { class: "text-sm text-green-600 dark:text-green-400", "Ready" }
                            }
                        },
                        StreamingState::Streaming => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-blue-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-blue-600 dark:text-blue-400", "Processing..." }
                            }
                        },
                        StreamingState::Error(ref error) => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-red-500 rounded-full" }
                                span { class: "text-sm text-red-600 dark:text-red-400", "Error" }
                            }
                        },
                        _ => {
                            rsx! {
                                span { class: "inline-block w-2 h-2 bg-yellow-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-yellow-600 dark:text-yellow-400", "Working..." }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Agent configuration overlay
#[component]
fn AgentConfigOverlay(state: Signal<RigAgentDemoState>) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl max-h-[90vh] overflow-y-auto m-4",
                div { class: "p-6",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-2xl font-bold text-gray-900 dark:text-gray-100",
                            "Agent Configuration"
                        }
                        button {
                            class: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                            onclick: move |_| {
                                state.write().is_config_panel_open = false;
                            },
                            "✕"
                        }
                    }

                    AgentConfigPanel {
                        config: state.clone().map(|s| s.config.clone()),
                        on_update: Some(move |new_config: AgentConfigState| {
                            state.write().config = new_config;
                            // Could trigger agent reconfiguration here
                        })
                    }

                    div { class="mt-6 flex justify-end space-x-2",
                        button {
                            class: "px-4 py-2 bg-gray-300 hover:bg-gray-400 text-gray-700 rounded-lg",
                            onclick: move |_| {
                                state.write().is_config_panel_open = false;
                            },
                            "Close"
                        }
                        button {
                            class: "px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg",
                            onclick: move |_| {
                                state.write().is_config_panel_open = false;
                                // Apply configuration and recreate agent
                                spawn(apply_agent_config(state.clone()));
                            },
                            "Apply & Restart"
                        }
                    }
                }
            }
        }
    }
}

/// Tool manager overlay
#[component]
fn ToolManagerOverlay(state: Signal<RigAgentDemoState>) -> Element {
    let tools = use_signal(|| state.read().available_tools.clone());
    let selected_tools = use_signal(|| state.read().selected_tools.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl max-h-[90vh] overflow-y-auto m-4",
                div { class: "p-6",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-2xl font-bold text-gray-900 dark:text-gray-100",
                            "Tool Manager"
                        }
                        button {
                            class: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                            onclick: move |_| {
                                state.write().is_tools_panel_open = false;
                            },
                            "✕"
                        }
                    }

                    ToolManager {
                        tools: tools.clone(),
                        selected_tools: selected_tools.clone(),
                        on_tool_toggle: Some(move |tool_name: String| {
                            let mut current_selected = selected_tools.read().clone();
                            if current_selected.contains(&tool_name) {
                                current_selected.retain(|t| t != &tool_name);
                            } else {
                                current_selected.push(tool_name);
                            }
                            selected_tools.set(current_selected);
                            state.write().selected_tools = selected_tools.read().clone();
                        })
                    }

                    div { class="mt-6 flex justify-end space-x-2",
                        button {
                            class: "px-4 py-2 bg-gray-300 hover:bg-gray-400 text-gray-700 rounded-lg",
                            onclick: move |_| {
                                state.write().is_tools_panel_open = false;
                            },
                            "Close"
                        }
                        button {
                            class: "px-4 py-2 bg-green-500 hover:bg-green-600 text-white rounded-lg",
                            onclick: move |_| {
                                state.write().is_tools_panel_open = false;
                                state.write().selected_tools = selected_tools.read().clone();
                            },
                            "Apply Tools"
                        }
                    }
                }
            }
        }
    }
}

/// Handle sending a message
async fn handle_send_message(
    state: Signal<RigAgentDemoState>,
    messages: Signal<Vec<StreamingMessage>>,
    message: String,
) {
    if message.trim().is_empty() {
        return;
    }

    // Update state to streaming
    state.write().streaming_state = StreamingState::Connecting;
    state.write().current_input = String::new();

    // Add user message
    let user_message = StreamingMessage {
        content: message.clone(),
        chunk_type: ChunkType::Content,
        metadata: None,
        timestamp: chrono::Utc::now(),
        is_complete: true,
    };

    let mut current_messages = messages.read().clone();
    current_messages.push(user_message);

    // Create or get conversation
    let conversation = if let Some(ref conv) = state.read().conversation {
        conv.clone()
    } else {
        Conversation::new(Vec::new()).unwrap()
    };

    // Get agent and send message
    let agent = state.read().agent.clone();
    let mut agent_guard = agent.write().await;

    state.write().streaming_state = StreamingState::Streaming;

    match agent_guard.reply(conversation, None, None).await {
        Ok(mut event_stream) => {
            let mut current_chunk_content = String::new();
            let mut current_chunk_type = ChunkType::Content;
            let mut agent_message = None;

            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        match event {
                            crate::agent::AgentEvent::Message(msg) => {
                                if !matches!(msg.role, Role::User) {
                                    match msg.content {
                                        MessageContent::Text(text) => {
                                            current_chunk_content += &text;
                                            current_chunk_type = ChunkType::Content;
                                        },
                                        MessageContent::Thinking(thinking) => {
                                            current_chunk_content = thinking;
                                            current_chunk_type = ChunkType::Thinking;
                                            state.write().streaming_state = StreamingState::Thinking;
                                        },
                                        _ => {}
                                    }

                                    let streaming_msg = StreamingMessage {
                                        content: current_chunk_content.clone(),
                                        chunk_type: current_chunk_type,
                                        metadata: None,
                                        timestamp: chrono::Utc::now(),
                                        is_complete: false,
                                    };

                                    current_messages.push(streaming_msg);
                                    messages.set(current_messages.clone());
                                }
                            },
                            crate::agent::AgentEvent::ToolCall(tool_call) => {
                                state.write().streaming_state = StreamingState::ToolCall;
                                let tool_msg = StreamingMessage {
                                    content: format!("Calling tool: {}", tool_call.name),
                                    chunk_type: ChunkType::ToolCall,
                                    metadata: None,
                                    timestamp: chrono::Utc::now(),
                                    is_complete: false,
                                };
                                current_messages.push(tool_msg);
                                messages.set(current_messages.clone());
                            },
                            crate::agent::AgentEvent::ToolResult(tool_result) => {
                                state.write().streaming_state = StreamingState::ToolResult;
                                let result_msg = StreamingMessage {
                                    content: format!("Tool result: {}", tool_result.result),
                                    chunk_type: ChunkType::ToolResult,
                                    metadata: None,
                                    timestamp: chrono::Utc::now(),
                                    is_complete: false,
                                };
                                current_messages.push(result_msg);
                                messages.set(current_messages.clone());
                            },
                            crate::agent::AgentEvent::Done => {
                                // Mark the last message as complete
                                if let Some(last_msg) = current_messages.last_mut() {
                                    last_msg.is_complete = true;
                                }
                                messages.set(current_messages.clone());
                                state.write().streaming_state = StreamingState::Complete;
                                break;
                            },
                            crate::agent::AgentEvent::Error(error) => {
                                state.write().streaming_state = StreamingState::Error(error);
                                break;
                            },
                            _ => {}
                        }
                    },
                    Err(e) => {
                        state.write().streaming_state = StreamingState::Error(format!("Stream error: {}", e));
                        break;
                    }
                }
            }
        },
        Err(e) => {
            state.write().streaming_state = StreamingState::Error(format!("Agent error: {}", e));
        }
    }
}

/// Apply agent configuration
async fn apply_agent_config(state: Signal<RigAgentDemoState>) {
    let config = AgentConfig::from(state.read().config.clone());

    match state.read().config.goose_mode {
        GooseMode::Chat => {
            if let Ok(agent) = AgentFactory::create_agent_with_config(config).await {
                state.write().agent = Arc::new(RwLock::new(agent));
            }
        },
        GooseMode::Agent => {
            if let Ok(agent) = AgentFactory::create_agent_with_config(config).await {
                state.write().agent = Arc::new(RwLock::new(agent));
            }
        },
        GooseMode::Auto => {
            if let Ok(agent) = AgentFactory::create_agent_with_config(config).await {
                state.write().agent = Arc::new(RwLock::new(agent));
            }
        },
    }
}

/// Clear chat history
async fn handle_clear_chat(
    state: Signal<RigAgentDemoState>,
    messages: Signal<Vec<StreamingMessage>>,
) {
    messages.set(Vec::new());
    state.write().conversation = None;
    state.write().streaming_state = StreamingState::Idle;
}