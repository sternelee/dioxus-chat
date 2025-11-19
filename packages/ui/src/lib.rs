//! This crate contains all shared UI for the workspace.

// Core UI Components (React-inspired design system)
mod ui_components;
pub use ui_components::{
    Dialog, DialogHeader, DialogTitle, DialogContent, DialogFooter,
    Card, CardItem,
    Button, ButtonVariant, ButtonSize,
    Input, Textarea, Switch,
    Avatar, AvatarSize, Badge, BadgeVariant,
};

// Enhanced Chat Interface
mod enhanced_chat;
pub use enhanced_chat::{
    EnhancedChatContainer, EnhancedChatMessage, EnhancedChatState,
    EnhancedMessageBubble, create_enhanced_chat_request,
};

// Agent Configuration Dialog
mod agent_config_dialog;
pub use agent_config_dialog::{
    AgentConfigDialog, AgentConfigDialogState, AgentData,
    AgentParameter, ParameterType,
};

// Parameter Management Interface
mod parameter_manager;
pub use parameter_manager::{
    ParameterManager, PredefinedParameter, ParameterRowProps,
};

// Original Rig-Integrated Chat Components
mod rig_chat;
pub use rig_chat::{RigChatContainer, RigChatSidebar, RigChatMessage, RigChatState, create_chat_request};

// Re-export Model from model_selector for SimpleModelSelector
mod model_selector;
pub use model_selector::Model;

// Basic components that should work
mod hero;
pub use hero::Hero;

mod echo;
pub use echo::Echo;

// Temporarily disable components with compilation issues - can be re-enabled later
// mod components;
// mod simple_chat;
// mod navbar;

// Legacy components that may need updates
// mod agent_config;
// pub use agent_config::{AgentConfigPanel, ToolManager, AgentStatus, AgentConfigState};

// mod streaming_chat;
// pub use streaming_chat::{
//     StreamingChatContainer, StreamingControls, StreamingMessage, StreamingState,
// };
