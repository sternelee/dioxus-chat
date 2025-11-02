//! This crate contains all shared UI for the workspace.

// Only export the essential components that work correctly
pub mod components;

// Simple Chat Components - these are the working components we need
mod simple_chat;
pub use simple_chat::{SimpleChatContainer, SimpleSidebar, SimpleModelSelector, SimpleChatMessage, SimpleConversationItem};

// Basic components that should work
mod hero;
pub use hero::Hero;

mod navbar;
pub use navbar::Navbar;

mod echo;
pub use echo::Echo;

// Re-export Model from model_selector for SimpleModelSelector
mod model_selector;
pub use model_selector::Model;
