use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct ModelSelectorProps {
    pub models: Vec<Model>,
    pub selected_model: Option<String>,
    pub on_select_model: EventHandler<String>,
    pub loading: Option<bool>,
}

#[derive(Clone, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
}

#[component]
pub fn ModelSelector(props: ModelSelectorProps) -> Element {
    let loading = props.loading.unwrap_or(false);

    rsx! {
        div {
            class: "relative",
            select {
                class: "px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors min-w-0",
                disabled: loading,
                onchange: move |event| {
                    props.on_select_model.call(event.value());
                },

                option {
                    value: "",
                    disabled: true,
                    selected: props.selected_model.is_none(),
                    if loading { "Loading models..." } else { "Select model" }
                }

                for model in &props.models {
                    option {
                        value: "{model.id}",
                        selected: props.selected_model.as_ref() == Some(&model.id),
                        "{model.name} - {model.provider}"
                    }
                }
            }
        }
    }
}

