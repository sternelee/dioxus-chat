// Parameter Management Interface for Agent Configuration
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::ui_components::*;
use crate::agent_config_dialog::{AgentParameter, ParameterType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredefinedParameter {
    pub key: String,
    pub title: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
}

// Predefined parameter templates
pub fn get_predefined_parameters() -> Vec<PredefinedParameter> {
    vec![
        PredefinedParameter {
            key: "temperature".to_string(),
            title: "Temperature".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()),
            description: Some("Controls randomness in responses (0.0-2.0)".to_string()),
        },
        PredefinedParameter {
            key: "max_tokens".to_string(),
            title: "Max Tokens".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(1000)),
            description: Some("Maximum number of tokens in response".to_string()),
        },
        PredefinedParameter {
            key: "top_p".to_string(),
            title: "Top P".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from_f64(0.9).unwrap()),
            description: Some("Controls diversity via nucleus sampling".to_string()),
        },
        PredefinedParameter {
            key: "frequency_penalty".to_string(),
            title: "Frequency Penalty".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            description: Some("Reduces repetition of frequent words".to_string()),
        },
        PredefinedParameter {
            key: "presence_penalty".to_string(),
            title: "Presence Penalty".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            description: Some("Encourages talking about new topics".to_string()),
        },
        PredefinedParameter {
            key: "stream".to_string(),
            title: "Stream".to_string(),
            value: serde_json::Value::Bool(true),
            description: Some("Enable streaming responses".to_string()),
        },
        PredefinedParameter {
            key: "timeout".to_string(),
            title: "Timeout".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(30)),
            description: Some("Request timeout in seconds".to_string()),
        },
        PredefinedParameter {
            key: "retry_attempts".to_string(),
            title: "Retry Attempts".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from(3)),
            description: Some("Number of retry attempts on failure".to_string()),
        },
    ]
}

#[derive(Clone, PartialEq, Props)]
pub struct ParameterManagerProps {
    pub parameters: Vec<AgentParameter>,
    pub on_parameters_change: EventHandler<Vec<AgentParameter>>,
    pub class: Option<String>,
}

#[component]
pub fn ParameterManager(props: ParameterManagerProps) -> Element {
    rsx! {
        div { class: format!("space-y-4 {}", props.class.unwrap_or_default()),
            // Header with Add Parameter Button
            div { class: "flex items-center justify-between",
                h3 { class: "text-lg font-medium text-gray-900 dark:text-gray-100",
                    "Parameters"
                }
                Button {
                    onclick: move |_| {
                        let mut new_params = props.parameters.clone();
                        new_params.push(AgentParameter {
                            key: String::new(),
                            value: serde_json::Value::String(String::new()),
                            param_type: ParameterType::String,
                        });
                        props.on_parameters_change.call(new_params);
                    },
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    "Add Parameter"
                }
            }

            // Predefined Parameters Quick Add
            div {
                h4 { class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                    "Quick Add Templates"
                }
                div { class: "flex flex-wrap gap-2",
                    for param in get_predefined_parameters() {
                        button {
                            class: "text-xs bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 px-3 py-1 rounded-full transition-colors border border-gray-300 dark:border-gray-600",
                            onclick: move |_| {
                                let mut new_params = props.parameters.clone();
                                // Check if parameter already exists
                                if !new_params.iter().any(|p| p.key == param.key) {
                                    new_params.push(AgentParameter {
                                        key: param.key.clone(),
                                        value: param.value.clone(),
                                        param_type: determine_parameter_type(&param.value),
                                    });
                                    props.on_parameters_change.call(new_params);
                                }
                            },
                            "{param.title}"
                        }
                        if let Some(desc) = param.description {
                            span {
                                class: "text-xs text-gray-500 dark:text-gray-400 ml-1",
                                title: desc.as_str(),
                                "ℹ️"
                            }
                        }
                    }
                }
            }

            // Parameters List
            div { class: "space-y-3",
                if props.parameters.is_empty() {
                    div { class: "text-center py-8 text-gray-500 dark:text-gray-400",
                        p { "No parameters configured" }
                        p { class: "text-sm mt-1",
                            "Add parameters from templates or create custom ones"
                        }
                    }
                } else {
                    for (index, param) in props.parameters.iter().enumerate() {
                        ParameterRow {
                            parameter: param.clone(),
                            index,
                            on_parameter_change: move |index, new_param| {
                                let mut new_params = props.parameters.clone();
                                new_params[index] = new_param;
                                props.on_parameters_change.call(new_params);
                            },
                            on_remove: move |index| {
                                let mut new_params = props.parameters.clone();
                                new_params.remove(index);
                                props.on_parameters_change.call(new_params);
                            },
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct ParameterRowProps {
    pub parameter: AgentParameter,
    pub index: usize,
    pub on_parameter_change: EventHandler<(usize, AgentParameter)>,
    pub on_remove: EventHandler<usize>,
}

#[component]
pub fn ParameterRow(props: ParameterRowProps) -> Element {
    let param_type = props.parameter.param_type.clone();
    let mut type_dropdown_open = use_signal(|| false);

    let handle_type_change = move |new_type: ParameterType| {
        let mut new_param = props.parameter.clone();
        new_param.param_type = new_type.clone();

        // Convert value to new type
        new_param.value = match new_type {
            ParameterType::String => serde_json::Value::String(format!("{:?}", new_param.value)),
            ParameterType::Number => serde_json::Value::Number(
                serde_json::Number::from_f64(new_param.value.as_f64().unwrap_or(0.0)).unwrap_or_else(|| serde_json::Number::from(0))
            ),
            ParameterType::Boolean => serde_json::Value::Bool(new_param.value.as_bool().unwrap_or(false)),
            ParameterType::Json => new_param.value,
        };

        props.on_parameter_change.call((props.index, new_param));
        type_dropdown_open.set(false);
    };

    let handle_key_change = move |key: String| {
        let mut new_param = props.parameter.clone();
        new_param.key = key;
        props.on_parameter_change.call((props.index, new_param));
    };

    let handle_value_change = move |value_str: String| {
        let mut new_param = props.parameter.clone();
        new_param.value = match param_type {
            ParameterType::String => serde_json::Value::String(value_str),
            ParameterType::Number => {
                if let Ok(num) = value_str.parse::<f64>() {
                    serde_json::Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0)))
                } else {
                    serde_json::Value::String(value_str)
                }
            },
            ParameterType::Boolean => {
                serde_json::Value::Bool(value_str.parse().unwrap_or(false))
            },
            ParameterType::Json => {
                if let Ok(json_val) = serde_json::from_str(&value_str) {
                    json_val
                } else {
                    serde_json::Value::String(value_str)
                }
            },
        };
        props.on_parameter_change.call((props.index, new_param));
    };

    rsx! {
        div { class: "flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
            // Parameter Key
            div { class: "flex-1",
                Input {
                    value: props.parameter.key.clone(),
                    oninput: handle_key_change,
                    placeholder: "Parameter key...",
                    class: "text-sm",
                }
            }

            // Parameter Type Selector
            div { class: "relative",
                button {
                    class: "flex items-center gap-1 px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700",
                    onclick: move |_| type_dropdown_open.set(!type_dropdown_open()),
                    span {
                        "{param_type:?}"
                    }
                    span { class: "text-xs", "▼" }
                }

                if *type_dropdown_open.read() {
                    div {
                        class: "absolute top-full left-0 mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md shadow-lg z-10",
                        button {
                            class: "block w-full text-left px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700",
                            onclick: move |_| handle_type_change(ParameterType::String),
                            "String"
                        }
                        button {
                            class: "block w-full text-left px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700",
                            onclick: move |_| handle_type_change(ParameterType::Number),
                            "Number"
                        }
                        button {
                            class: "block w-full text-left px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700",
                            onclick: move |_| handle_type_change(ParameterType::Boolean),
                            "Boolean"
                        }
                        button {
                            class: "block w-full text-left px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700",
                            onclick: move |_| handle_type_change(ParameterType::Json),
                            "JSON"
                        }
                    }
                }
            }

            // Parameter Value
            div { class: "flex-2",
                match param_type {
                    ParameterType::Boolean => {
                        Switch {
                            checked: props.parameter.value.as_bool().unwrap_or(false),
                            on_checked_change: move |checked| {
                                let mut new_param = props.parameter.clone();
                                new_param.value = serde_json::Value::Bool(checked);
                                props.on_parameter_change.call((props.index, new_param));
                            },
                        }
                    },
                    ParameterType::Number => {
                        Input {
                            value: props.parameter.value.as_f64().unwrap_or(0.0).to_string(),
                            r#type: "number".to_string(),
                            oninput: handle_value_change,
                            placeholder: "Number value...",
                            class: "text-sm",
                        }
                    },
                    ParameterType::Json => {
                        Textarea {
                            value: props.parameter.value.to_string(),
                            oninput: handle_value_change,
                            placeholder: "JSON value...",
                            rows: 2,
                            class: "text-sm font-mono",
                        }
                    },
                    _ => {
                        Input {
                            value: props.parameter.value.as_str().unwrap_or("").to_string(),
                            oninput: handle_value_change,
                            placeholder: "String value...",
                            class: "text-sm",
                        }
                    },
                }
            }

            // Remove Button
            button {
                class: "p-1 text-red-500 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors",
                onclick: move |_| props.on_remove.call(props.index),
                title: "Remove parameter",
                "×"
            }
        }
    }
}

fn determine_parameter_type(value: &serde_json::Value) -> ParameterType {
    match value {
        serde_json::Value::String(_) => ParameterType::String,
        serde_json::Value::Number(_) => ParameterType::Number,
        serde_json::Value::Bool(_) => ParameterType::Boolean,
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => ParameterType::Json,
        serde_json::Value::Null => ParameterType::String,
    }
}