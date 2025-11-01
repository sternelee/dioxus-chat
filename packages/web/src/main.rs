use dioxus::prelude::*;

use ui::{Navbar, ChatContainer, ChatMessage, Sidebar, ConversationItem, ModelSelector, Model};
use views::{Blog, Home, Chat};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebNavbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/chat")]
    Chat {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
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
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME }

        Router::<Route> {}
    }
}

/// A web-specific Router around the shared `Navbar` component
/// which allows us to use the web-specific `Route` enum.
#[component]
fn WebNavbar() -> Element {
    rsx! {
        Navbar {
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Chat {},
                "Chat"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}
