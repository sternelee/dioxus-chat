// Agent Configuration UI Components
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use api::{AgentConfig, GooseMode, Tool};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfigState {
    pub goose_mode: GooseMode,
    pub max_iterations: usize,
    pub require_confirmation: bool,
    pub enable_tool_inspection: bool,
    pub enable_auto_compact: bool,
    pub compact_threshold: f32,
    pub max_turns_without_tools: usize,
    pub enable_autopilot: bool,
    pub enable_extensions: bool,
    pub extension_timeout: u64,
}

impl Default for AgentConfigState {
    fn default() -> Self {
        Self {
            goose_mode: GooseMode::Chat,
            max_iterations: 10,
            require_confirmation: false,
            enable_tool_inspection: true,
            enable_auto_compact: true,
            compact_threshold: 0.8,
            max_turns_without_tools: 3,
            enable_autopilot: false,
            enable_extensions: true,
            extension_timeout: 30,
        }
    }
}

impl From<AgentConfigState> for AgentConfig {
    fn from(state: AgentConfigState) -> Self {
        Self {
            goose_mode: state.goose_mode,
            max_iterations: state.max_iterations,
            require_confirmation: state.require_confirmation,
            readonly_tools: vec![],
            enable_tool_inspection: state.enable_tool_inspection,
            enable_auto_compact: state.enable_auto_compact,
            compact_threshold: state.compact_threshold,
            max_turns_without_tools: state.max_turns_without_tools,
            enable_autopilot: state.enable_autopilot,
            enable_extensions: state.enable_extensions,
            extension_timeout: state.extension_timeout,
        }
    }
}

#[component]
pub fn AgentConfigPanel(
    config: Signal<AgentConfigState>,
    on_update: Option<EventHandler<AgentConfigState>>,
) -> Element {
    rsx! {
        div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 space-y-4",
            h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                "Agent Configuration"
            }

            // Agent Mode Selection
            div { class: "space-y-2",
                label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300",
                    "Agent Mode"
                }
                select {
                    class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                    value: "{config.read().goose_mode:?}",
                    onchange: move |evt| {
                        let mode = match evt.value.as_str() {
                            "Agent" => GooseMode::Agent,
                            "Auto" => GooseMode::Auto,
                            _ => GooseMode::Chat,
                        };
                        config.write().goose_mode = mode;
                        if let Some(ref handler) = on_update {
                            handler.call(config.read().clone());
                        }
                    },
                    option { value: "Chat", "Chat - Natural conversation" }
                    option { value: "Agent", "Agent - Tool using assistant" }
                    option { value: "Auto", "Auto - Autonomous agent" }
                }
                p { class: "text-xs text-gray-500 dark:text-gray-400 mt-1",
                    match config.read().goose_mode {
                        GooseMode::Chat => "Optimized for natural conversation",
                        GooseMode::Agent => "Can use tools and perform complex tasks",
                        GooseMode::Auto => "Autonomous agent that takes initiative",
                    }
                }
            }

            // Iterations and Timeout
            div { class: "grid grid-cols-2 gap-4",
                div { class: "space-y-2",
                    label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Max Iterations"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{config.read().max_iterations}",
                        min: "1",
                        max: "100",
                        onchange: move |evt| {
                            if let Ok(value) = evt.value.parse::<usize>() {
                                config.write().max_iterations = value;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-2",
                    label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Extension Timeout (sec)"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{config.read().extension_timeout}",
                        min: "5",
                        max: "300",
                        onchange: move |evt| {
                            if let Ok(value) = evt.value.parse::<u64>() {
                                config.write().extension_timeout = value;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                    }
                }
            }

            // Feature Toggles
            div { class: "space-y-3",
                h4 { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                    "Features"
                }

                // Enable Tool Inspection
                div { class: "flex items-center justify-between",
                    label { class: "flex items-center cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "mr-2 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: config.read().enable_tool_inspection,
                            onchange: move |evt| {
                                config.write().enable_tool_inspection = evt.checked;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                        span { class: "text-sm text-gray-700 dark:text-gray-300",
                            "Enable Tool Inspection"
                        }
                    }
                    span { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Allows examining tool usage"
                    }
                }

                // Enable Auto Compact
                div { class: "flex items-center justify-between",
                    label { class: "flex items-center cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "mr-2 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: config.read().enable_auto_compact,
                            onchange: move |evt| {
                                config.write().enable_auto_compact = evt.checked;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                        span { class: "text-sm text-gray-700 dark:text-gray-300",
                            "Enable Auto Compact"
                        }
                    }
                    span { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Automatically manage conversation size"
                    }
                }

                // Require Confirmation
                div { class: "flex items-center justify-between",
                    label { class: "flex items-center cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "mr-2 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: config.read().require_confirmation,
                            onchange: move |evt| {
                                config.write().require_confirmation = evt.checked;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                        span { class: "text-sm text-gray-700 dark:text-gray-300",
                            "Require Confirmation"
                        }
                    }
                    span { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Ask for confirmation on actions"
                    }
                }

                // Enable Autopilot
                div { class: "flex items-center justify-between",
                    label { class: "flex items-center cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "mr-2 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: config.read().enable_autopilot,
                            onchange: move |evt| {
                                config.write().enable_autopilot = evt.checked;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                        span { class: "text-sm text-gray-700 dark:text-gray-300",
                            "Enable Autopilot"
                        }
                    }
                    span { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Agent takes autonomous actions"
                    }
                }

                // Enable Extensions
                div { class: "flex items-center justify-between",
                    label { class: "flex items-center cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "mr-2 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: config.read().enable_extensions,
                            onchange: move |evt| {
                                config.write().enable_extensions = evt.checked;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                        span { class: "text-sm text-gray-700 dark:text-gray-300",
                            "Enable Extensions"
                        }
                    }
                    span { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Load agent extensions"
                    }
                }
            }

            // Advanced Settings
            div { class: "space-y-3",
                h4 { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                    "Advanced Settings"
                }

                // Compact Threshold
                div { class: "space-y-2",
                    label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Compact Threshold: {config.read().compact_threshold:.1}"
                    }
                    input {
                        r#type: "range",
                        class: "w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700",
                        min: "0.1",
                        max: "1.0",
                        step: "0.1",
                        value: "{config.read().compact_threshold}",
                        onchange: move |evt| {
                            if let Ok(value) = evt.value.parse::<f32>() {
                                config.write().compact_threshold = value;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                    }
                    div { class: "flex justify-between text-xs text-gray-500 dark:text-gray-400",
                        span { "Aggressive" }
                        span { "Conservative" }
                    }
                }

                // Max Turns Without Tools
                div { class: "space-y-2",
                    label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Max Turns Without Tools: {config.read().max_turns_without_tools}"
                    }
                    input {
                        r#type: "range",
                        class: "w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700",
                        min: "1",
                        max: "20",
                        step: "1",
                        value: "{config.read().max_turns_without_tools}",
                        onchange: move |evt| {
                            if let Ok(value) = evt.value.parse::<usize>() {
                                config.write().max_turns_without_tools = value;
                                if let Some(ref handler) = on_update {
                                    handler.call(config.read().clone());
                                }
                            }
                        }
                    }
                    div { class: "flex justify-between text-xs text-gray-500 dark:text-gray-400",
                        span { "Frequent" }
                        span { "Infrequent" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ToolManager(
    tools: Signal<Vec<Tool>>,
    selected_tools: Signal<Vec<String>>,
    on_tool_toggle: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
            h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                "Tool Manager"
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                // Available Tools
                div { class: "space-y-2",
                    h4 { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Available Tools"
                    }
                    div { class: "space-y-2 max-h-60 overflow-y-auto",
                        {tools.read().iter().map(|tool| {
                            let is_selected = selected_tools.read().contains(&tool.name);
                            rsx! {
                                div {
                                    key: "{tool.name}",
                                    class: "flex items-center p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700",

                                    input {
                                        r#type: "checkbox",
                                        class: "mr-3 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                                        checked: is_selected,
                                        onchange: move |evt| {
                                            if let Some(ref handler) = on_tool_toggle {
                                                handler.call(tool.name.clone());
                                            }
                                        }
                                    }

                                    div { class: "flex-1",
                                        div { class: "font-medium text-sm text-gray-900 dark:text-gray-100",
                                            "{tool.name}"
                                        }
                                        div { class: "text-xs text-gray-500 dark:text-gray-400 mt-1",
                                            "{tool.description}"
                                        }
                                        if tool.is_mcp {
                                            span { class: "inline-block px-2 py-1 text-xs bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200 rounded-full mt-1",
                                                "MCP Tool"
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }

                // Selected Tools Summary
                div { class: "space-y-2",
                    h4 { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Selected Tools ({selected_tools.read().len()})"
                    }
                    if selected_tools.read().is_empty() {
                        div { class: "text-gray-500 dark:text-gray-400 text-sm",
                            "No tools selected"
                        }
                    } else {
                        div { class="space-y-1 max-h-60 overflow-y-auto",
                            {selected_tools.read().iter().map(|tool_name| {
                                rsx! {
                                    div {
                                        key: "{tool_name}",
                                        class: "flex items-center justify-between p-2 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded",
                                        span { class: "text-sm font-medium text-blue-900 dark:text-blue-100",
                                            "{tool_name}"
                                        }
                                        button {
                                            class: "text-red-500 hover:text-red-700 text-sm",
                                            onclick: move |_| {
                                                if let Some(ref handler) = on_tool_toggle {
                                                    handler.call(tool_name.clone());
                                                }
                                            },
                                            "Remove"
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            }

            // Tool Configuration
            if !selected_tools.read().is_empty() {
                div { class: "mt-4 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg",
                    h4 { class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                        "Tool Configuration"
                    }
                    p { class: "text-xs text-gray-600 dark:text-gray-400",
                        "Selected tools will be available to the agent during conversations. The agent can use these tools to gather information, perform calculations, or interact with external systems."
                    }
                }
            }
        }
    }
}

#[component]
pub fn AgentStatus(
    config: AgentConfigState,
    is_active: bool,
    current_model: String,
) -> Element {
    rsx! {
        div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
            h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                "Agent Status"
            }

            div { class: "grid grid-cols-2 gap-4",
                div { class: "space-y-2",
                    div { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Mode"
                    }
                    div { class: "flex items-center",
                        span {
                            class: if is_active {
                                "inline-block w-2 h-2 bg-green-500 rounded-full mr-2"
                            } else {
                                "inline-block w-2 h-2 bg-gray-400 rounded-full mr-2"
                            }
                        }
                        span { class: "text-gray-900 dark:text-gray-100",
                            "{config.goose_mode:?}"
                        }
                    }
                }

                div { class: "space-y-2",
                    div { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                        "Model"
                    }
                    div { class: "text-gray-900 dark:text-gray-100",
                        "{current_model}"
                    }
                }
            }

            div { class: "mt-4 grid grid-cols-3 gap-4 text-center",
                div { class: "space-y-1",
                    div { class: "text-2xl font-bold text-blue-600",
                        "{config.max_iterations}"
                    }
                    div { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Max Iterations"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-2xl font-bold text-green-600",
                        "{config.max_turns_without_tools}"
                    }
                    div { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Max Turns"
                    }
                }
                div { class: "space-y-1",
                    div { class: "text-2xl font-bold text-purple-600",
                        "{config.extension_timeout}s"
                    }
                    div { class: "text-xs text-gray-500 dark:text-gray-400",
                        "Timeout"
                    }
                }
            }

            // Feature Status
            div { class: "mt-4 space-y-2",
                div { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                    "Enabled Features"
                }
                div { class = "flex flex-wrap gap-2",
                    if config.enable_tool_inspection {
                        span { class: "px-2 py-1 text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 rounded-full",
                            "Tool Inspection"
                        }
                    }
                    if config.enable_auto_compact {
                        span { class: "px-2 py-1 text-xs bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 rounded-full",
                            "Auto Compact"
                        }
                    }
                    if config.require_confirmation {
                        span { class: "px-2 py-1 text-xs bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 rounded-full",
                            "Confirmation"
                        }
                    }
                    if config.enable_autopilot {
                        span { class: "px-2 py-1 text-xs bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200 rounded-full",
                            "Autopilot"
                        }
                    }
                    if config.enable_extensions {
                        span { class: "px-2 py-1 text-xs bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200 rounded-full",
                            "Extensions"
                        }
                    }
                }
            }
        }
    }
}