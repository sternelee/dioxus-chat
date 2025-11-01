//! This crate contains all shared UI for the workspace.

mod hero;
pub use hero::Hero;

mod navbar;
pub use navbar::Navbar;

mod echo;
pub use echo::Echo;

// Dioxus Components
pub mod components;

// Chat components
mod message;
pub use message::Message;

mod chat_input;
pub use chat_input::ChatInput;

mod chat_container;
pub use chat_container::{ChatContainer, ChatMessage};

mod sidebar;
pub use sidebar::{ConversationItem, Sidebar};

mod model_selector;
pub use model_selector::{Model, ModelSelector};

mod settings_menu;
pub use settings_menu::{SettingsMenu, Theme};

// Enhanced Thread components
mod thread_list;
pub use thread_list::{ThreadList, Thread as ThreadItem};

mod thread_content;
pub use thread_content::{ThreadContent, ChatMessage as ThreadChatMessage, MessageRole, ToolCall, ToolCallStatus};

mod left_panel;
pub use left_panel::{LeftPanel, LeftPanelView};

mod settings_panel_simple;
pub use settings_panel_simple::{SettingsPanel, SettingsPanelSimple, SettingsTab, Model as SettingsModel};

mod settings_panel_complete;
pub use settings_panel_complete::{SettingsPanelComplete, SettingsTab as SettingsTabComplete, Model as SettingsModelComplete, Provider, DataSource, Shortcut, PerformanceSettings, ModelPricing, RateLimit, DataSourceType, SyncFrequency, ShortcutCategory, LogLevel};

// Chat App Example
mod chat_app;
pub use chat_app::{ChatApp, example_chat_app};

// Settings Example
mod settings_example;
pub use settings_example::SettingsExample;

// Core Settings Example
mod settings_core_example;
pub use settings_core_example::SettingsCoreExample;

// Core Settings Panel
mod settings_panel_core;
pub use settings_panel_core::{SettingsPanelCore, CoreSettingsTab, AIProvider, MCPServer, Theme, AgentConfig};
