use dioxus::prelude::*;
#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gray-50 dark:bg-gray-900",
            div {
                class: "max-w-7xl mx-auto py-12 px-4 sm:px-6 lg:px-8",
                div {
                    class: "text-center",
                        h1 {
                            class: "text-4xl font-bold text-gray-900 dark:text-gray-100 mb-4",
                            "Welcome to Dioxus Chat"
                        }
                    p {
                        class: "text-xl text-gray-600 dark:text-gray-400 mb-8",
                        "A modern Rust-based chat application"
                    }
                    div {
                        class: "flex justify-center gap-4",
                        button {
                            class: "px-6 py-3 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors",
                            "Get Started"
                        }
                        a {
                            href: "/chat",
                            class: "px-6 py-3 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded-lg transition-colors",
                            "Go to Chat"
                        }
                    }
                }
            }
        }
    }
}
