use dioxus::prelude::*;

// Basic HTML components only - no complex UI dependencies
use views::{Blog, Home, SimpleGoose, RigAgentDemo};
use views::chat_simple::SimpleChat as Chat;

mod views;
mod storage;
mod agent; // Re-enabled with rig integration

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopNavbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/chat")]
    Chat {},
    #[route("/goose")]
    SimpleGoose {},
    #[route("/rig-demo")]
    RigAgentDemo {},
}

const MAIN_CSS: Asset = asset!("/assets/main.css");
const DX_COMPONENTS_THEME: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }

        Router::<Route> {}
    }
}

/// A desktop-specific Router around a simple navbar
/// which allows us to use the desktop-specific `Route` enum.
#[component]
fn DesktopNavbar() -> Element {
    rsx! {
        nav {
            class: "bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4",
            div {
                class: "flex items-center gap-6",
                Link {
                    to: Route::Home {},
                    class: "text-gray-700 dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 font-medium",
                    "Home"
                }
                Link {
                    to: Route::Chat {},
                    class: "text-gray-700 dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 font-medium",
                    "Chat"
                }
                Link {
                    to: Route::Blog { id: 1 },
                    class: "text-gray-700 dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 font-medium",
                    "Blog"
                }
                Link {
                    to: Route::SimpleGoose {},
                    class: "text-gray-700 dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 font-medium",
                    "Goose Chat"
                }
                Link {
                    to: Route::RigAgentDemo {},
                    class: "text-gray-700 dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 font-medium bg-blue-100 dark:bg-blue-900 px-2 py-1 rounded",
                    "🚀 Rig Demo"
                }
            }
        }

        Outlet::<Route> {}
    }
}
