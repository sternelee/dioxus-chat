mod home;
pub use home::Home;

mod blog;
pub use blog::Blog;

pub mod chat_simple;
pub use chat_simple::SimpleChat as Chat;

pub mod goose_simple;
pub use goose_simple::GooseSimpleChat as SimpleGoose;

pub mod goose_chat;
pub use goose_chat::GooseChat;

pub mod rig_agent_demo;
pub use rig_agent_demo::RigAgentDemo;
