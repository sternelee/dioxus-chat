// Improved UI Components based on React design patterns
use dioxus::prelude::*;

// Dialog Components
#[derive(Clone, PartialEq, Props)]
pub struct DialogProps {
    pub open: bool,
    pub on_open_change: EventHandler<bool>,
    pub children: Element,
    pub class: Option<String>,
    pub show_close_button: Option<bool>,
    pub max_width: Option<String>,
}

#[component]
pub fn Dialog(props: DialogProps) -> Element {
    if !props.open {
        return rsx! { {props.children} };
    }

    rsx! {
        div {
            class: format!(
                "fixed inset-0 z-50 flex items-center justify-center {}",
                props.class.unwrap_or_default()
            ),
            onclick: move |evt| {
                if evt.target() == evt.current_target() {
                    props.on_open_change.call(false);
                }
            },

            // Backdrop
            div {
                class: "fixed inset-0 bg-black/50",
                "aria-hidden": "true"
            }

            // Dialog content
            div {
                class: format!(
                    "relative bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 w-full max-w-md mx-4 {}",
                    props.max_width.unwrap_or_default()
                ),
                onclick: move |evt| evt.stop_propagation(),

                if props.show_close_button.unwrap_or(true) {
                    button {
                        class: "absolute right-4 top-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300",
                        onclick: move |_| props.on_open_change.call(false),
                        "×"
                    }
                }

                {props.children}
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogHeaderProps {
    pub children: Element,
    pub class: Option<String>,
}

#[component]
pub fn DialogHeader(props: DialogHeaderProps) -> Element {
    rsx! {
        div {
            class: format!("mb-4 {}", props.class.unwrap_or_default()),
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogTitleProps {
    pub children: Element,
    pub class: Option<String>,
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    rsx! {
        h2 {
            class: format!("text-lg font-semibold text-gray-900 dark:text-gray-100 {}", props.class.unwrap_or_default()),
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogContentProps {
    pub children: Element,
    pub class: Option<String>,
}

#[component]
pub fn DialogContent(props: DialogContentProps) -> Element {
    rsx! {
        div {
            class: format!("{}", props.class.unwrap_or_default()),
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogFooterProps {
    pub children: Element,
    pub class: Option<String>,
}

#[component]
pub fn DialogFooter(props: DialogFooterProps) -> Element {
    rsx! {
        div {
            class: format!("flex justify-end gap-2 mt-6 {}", props.class.unwrap_or_default()),
            {props.children}
        }
    }
}

// Card Components
#[derive(Clone, PartialEq, Props)]
pub struct CardProps {
    pub title: Option<String>,
    pub children: Element,
    pub class: Option<String>,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    rsx! {
        div {
            class: format!("bg-white dark:bg-gray-800 rounded-lg shadow-md p-4 {}", props.class.unwrap_or_default()),

            if let Some(title) = props.title {
                h3 {
                    class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3",
                    "{title}"
                }
            }

            {props.children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct CardItemProps {
    pub title: Option<String>,
    pub description: Option<String>,
    pub actions: Option<Element>,
    pub children: Option<Element>,
    pub class: Option<String>,
}

#[component]
pub fn CardItem(props: CardItemProps) -> Element {
    rsx! {
        div {
            class: format!("flex items-center justify-between py-3 border-b border-gray-200 dark:border-gray-700 last:border-0 {}", props.class.unwrap_or_default()),

            div {
                class: "flex-1",

                if let Some(title) = props.title {
                    h4 {
                        class: "text-sm font-medium text-gray-900 dark:text-gray-100",
                        "{title}"
                    }
                }

                if let Some(description) = props.description {
                    p {
                        class: "text-xs text-gray-500 dark:text-gray-400 mt-1",
                        "{description}"
                    }
                }

                if let Some(children) = props.children {
                    {children}
                }
            }

            if let Some(actions) = props.actions {
                div {
                    class: "ml-4",
                    {actions}
                }
            }
        }
    }
}

// Button Components
#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    pub onclick: EventHandler,
    pub children: Element,
    pub variant: Option<ButtonVariant>,
    pub size: Option<ButtonSize>,
    pub disabled: Option<bool>,
    pub class: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

#[derive(Clone, PartialEq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

impl Default for ButtonVariant {
    fn default() -> Self { ButtonVariant::Primary }
}

impl Default for ButtonSize {
    fn default() -> Self { ButtonSize::Md }
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let variant = props.variant.unwrap_or_default();
    let size = props.size.unwrap_or_default();

    let base_classes = "inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none";

    let variant_classes = match variant {
        ButtonVariant::Primary => "bg-blue-600 text-white hover:bg-blue-700",
        ButtonVariant::Secondary => "bg-gray-100 text-gray-900 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-100 dark:hover:bg-gray-700",
        ButtonVariant::Destructive => "bg-red-600 text-white hover:bg-red-700",
        ButtonVariant::Outline => "border border-gray-300 bg-white text-gray-900 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-900 dark:text-gray-100 dark:hover:bg-gray-800",
        ButtonVariant::Ghost => "hover:bg-gray-100 hover:text-gray-900 dark:hover:bg-gray-800 dark:hover:text-gray-100",
        ButtonVariant::Link => "text-blue-600 underline-offset-4 hover:underline",
    };

    let size_classes = match size {
        ButtonSize::Sm => "h-8 px-3 text-xs",
        ButtonSize::Md => "h-10 px-4 py-2",
        ButtonSize::Lg => "h-12 px-8 text-lg",
    };

    rsx! {
        button {
            class: format!(
                "{} {} {} {}",
                base_classes,
                variant_classes,
                size_classes,
                props.class.unwrap_or_default()
            ),
            onclick: props.onclick,
            disabled: props.disabled.unwrap_or(false),
            {props.children}
        }
    }
}

// Input Components
#[derive(Clone, PartialEq, Props)]
pub struct InputProps {
    pub value: String,
    pub oninput: EventHandler<String>,
    pub placeholder: Option<String>,
    pub r#type: Option<String>,
    pub disabled: Option<bool>,
    pub class: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    rsx! {
        input {
            id: props.id.as_deref(),
            name: props.name.as_deref(),
            r#type: props.r#type.unwrap_or("text".to_string()),
            class: format!(
                "flex h-9 w-full rounded-md border border-gray-300 bg-white px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:placeholder:text-gray-400 {}",
                props.class.unwrap_or_default()
            ),
            value: "{props.value}",
            placeholder: props.placeholder.as_deref(),
            oninput: move |evt| props.oninput.call(evt.value()),
            disabled: props.disabled.unwrap_or(false),
        }
    }
}

// Textarea Component
#[derive(Clone, PartialEq, Props)]
pub struct TextareaProps {
    pub value: String,
    pub oninput: EventHandler<String>,
    pub placeholder: Option<String>,
    pub rows: Option<u32>,
    pub disabled: Option<bool>,
    pub class: Option<String>,
    pub resize: Option<TextareaResize>,
}

#[derive(Clone, PartialEq)]
pub enum TextareaResize {
    None,
    Vertical,
    Horizontal,
    Both,
}

impl Default for TextareaResize {
    fn default() -> Self { TextareaResize::Vertical }
}

#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let resize_class = match props.resize.unwrap_or_default() {
        TextareaResize::None => "resize-none",
        TextareaResize::Vertical => "resize-y",
        TextareaResize::Horizontal => "resize-x",
        TextareaResize::Both => "resize",
    };

    rsx! {
        textarea {
            class: format!(
                "flex min-h-[60px] w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm shadow-sm placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:placeholder:text-gray-400 {} {}",
                props.class.unwrap_or_default(),
                resize_class
            ),
            value: "{props.value}",
            placeholder: props.placeholder.as_deref(),
            rows: props.rows.unwrap_or(4),
            oninput: move |evt| props.oninput.call(evt.value()),
            disabled: props.disabled.unwrap_or(false),
        }
    }
}

// Switch Component
#[derive(Clone, PartialEq, Props)]
pub struct SwitchProps {
    pub checked: bool,
    pub on_checked_change: EventHandler<bool>,
    pub disabled: Option<bool>,
    pub id: Option<String>,
    pub class: Option<String>,
}

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    rsx! {
        button {
            id: props.id.as_deref(),
            r#type: "button",
            role: "switch",
            "aria-checked": props.checked,
            class: format!(
                "inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 {} {}",
                if props.checked {
                    "bg-blue-600"
                } else {
                    "bg-gray-200 dark:bg-gray-700"
                },
                props.class.unwrap_or_default()
            ),
            onclick: move |_| props.on_checked_change.call(!props.checked),
            disabled: props.disabled.unwrap_or(false),

            span {
                class: format!(
                    "pointer-events-none block h-4 w-4 rounded-full bg-white shadow-lg ring-0 transition-transform {}",
                    if props.checked {
                        "translate-x-4"
                    } else {
                        "translate-x-0"
                    }
                )
            }
        }
    }
}

// Avatar Component
#[derive(Clone, PartialEq, Props)]
pub struct AvatarProps {
    pub src: Option<String>,
    pub alt: Option<String>,
    pub fallback: Option<String>,
    pub size: Option<AvatarSize>,
    pub class: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum AvatarSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl Default for AvatarSize {
    fn default() -> Self { AvatarSize::Md }
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let size = props.size.unwrap_or_default();
    let size_classes = match size {
        AvatarSize::Sm => "w-6 h-6 text-xs",
        AvatarSize::Md => "w-8 h-8 text-sm",
        AvatarSize::Lg => "w-12 h-12 text-lg",
        AvatarSize::Xl => "w-16 h-16 text-xl",
    };

    rsx! {
        div {
            class: format!(
                "relative inline-flex items-center justify-center rounded-full bg-gray-200 dark:bg-gray-700 overflow-hidden {} {}",
                size_classes,
                props.class.unwrap_or_default()
            ),

            if let Some(src) = props.src {
                img {
                    src: "{src}",
                    alt: props.alt.as_deref().unwrap_or("Avatar"),
                    class: "h-full w-full object-cover",
                }
            } else {
                span {
                    class: "font-medium text-gray-600 dark:text-gray-300 uppercase",
                    "{props.fallback.as_ref().map_or(String::new(), |s| s.clone())}"
                }
            }
        }
    }
}

// Badge Component
#[derive(Clone, PartialEq, Props)]
pub struct BadgeProps {
    pub children: Element,
    pub variant: Option<BadgeVariant>,
    pub class: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

impl Default for BadgeVariant {
    fn default() -> Self { BadgeVariant::Default }
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let variant = props.variant.unwrap_or_default();
    let variant_classes = match variant {
        BadgeVariant::Default => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
        BadgeVariant::Secondary => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200",
        BadgeVariant::Destructive => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
        BadgeVariant::Outline => "border border-gray-300 bg-transparent text-gray-800 dark:border-gray-600 dark:text-gray-200",
    };

    rsx! {
        span {
            class: format!(
                "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 {} {}",
                variant_classes,
                props.class.unwrap_or_default()
            ),
            {props.children}
        }
    }
}