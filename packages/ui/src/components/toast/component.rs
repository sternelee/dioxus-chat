use dioxus::prelude::*;
use dioxus_primitives::toast::{self, ToastProviderProps};

#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        toast::ToastProvider {
            default_duration: props.default_duration,
            max_toasts: props.max_toasts,
            render_toast: props.render_toast,
            {props.children}
        }
    }
}

#[component]
pub fn Toast(
    #[props(default)] class: String,
    #[props(default)] title: Option<String>,
    #[props(default)] description: Option<String>,
    #[props(default)] variant: ToastVariant,
    #[props(default)] attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        div {
            class: "toast toast-{variant:?} {class}",
            {attributes}
            if let Some(title) = title {
                div {
                    class: "toast-title",
                    "{title}"
                }
            }
            if let Some(description) = description {
                div {
                    class: "toast-description",
                    "{description}"
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ToastVariant {
    #[default]
    Default,
    Success,
    Error,
    Warning,
    Info,
}
