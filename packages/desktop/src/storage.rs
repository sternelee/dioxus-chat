use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<StoredMessage>,
    pub last_updated: String,
    pub model: Option<String>,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub selected_model: Option<String>,
    pub theme: String,
    pub window_width: f64,
    pub window_height: f64,
    pub auto_save: bool,
    pub max_conversations: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            selected_model: None,
            theme: "system".to_string(),
            window_width: 1200.0,
            window_height: 800.0,
            auto_save: true,
            max_conversations: 100,
        }
    }
}

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let mut data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
        data_dir.push("dioxus-chat");
        fs::create_dir_all(&data_dir)?;

        Ok(Self { data_dir })
    }

    pub fn save_conversations(&self, conversations: &HashMap<String, StoredConversation>) -> Result<()> {
        let file_path = self.data_dir.join("conversations.json");
        let json = serde_json::to_string_pretty(conversations)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    pub fn load_conversations(&self) -> Result<HashMap<String, StoredConversation>> {
        let file_path = self.data_dir.join("conversations.json");
        if file_path.exists() {
            let json = fs::read_to_string(file_path)?;
            let conversations: HashMap<String, StoredConversation> = serde_json::from_str(&json)?;
            Ok(conversations)
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let file_path = self.data_dir.join("settings.json");
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<AppSettings> {
        let file_path = self.data_dir.join("settings.json");
        if file_path.exists() {
            let json = fs::read_to_string(file_path)?;
            let settings: AppSettings = serde_json::from_str(&json)?;
            Ok(settings)
        } else {
            Ok(AppSettings::default())
        }
    }

    pub fn export_conversation(&self, conversation_id: &str, format: ExportFormat) -> Result<String> {
        let conversations = self.load_conversations()?;
        let conversation = conversations.get(conversation_id)
            .ok_or_else(|| anyhow::anyhow!("Conversation not found"))?;

        match format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(conversation)?),
            ExportFormat::Markdown => self.conversation_to_markdown(conversation),
            ExportFormat::Text => self.conversation_to_text(conversation),
            ExportFormat::Csv => self.conversation_to_csv(conversation),
        }
    }

    pub fn import_conversation(&self, data: &str, format: ExportFormat) -> Result<StoredConversation> {
        match format {
            ExportFormat::Json => {
                let conversation: StoredConversation = serde_json::from_str(data)?;
                Ok(conversation)
            }
            _ => Err(anyhow::anyhow!("Only JSON import is currently supported"))
        }
    }

    fn conversation_to_markdown(&self, conversation: &StoredConversation) -> Result<String> {
        let mut markdown = format!("# {}\n\n", conversation.title);
        markdown.push_str(&format!("**Created:** {}\n", conversation.last_updated));
        if let Some(model) = &conversation.model {
            markdown.push_str(&format!("**Model:** {}\n", model));
        }
        markdown.push_str(&format!("**Messages:** {}\n\n", conversation.messages.len()));

        for message in &conversation.messages {
            let role = if message.is_user { "User" } else { "Assistant" };
            markdown.push_str(&format!("## {}\n\n", role));
            markdown.push_str(&format!("{}\n\n", message.content));
            if let Some(timestamp) = &message.timestamp {
                markdown.push_str(&format!("*{}*\n\n", timestamp));
            }
        }

        markdown.push_str(&format!(
            "\n---\n**Token Usage:** {} ({} prompt + {} completion)\n",
            conversation.token_usage.total_tokens,
            conversation.token_usage.prompt_tokens,
            conversation.token_usage.completion_tokens
        ));

        Ok(markdown)
    }

    fn conversation_to_text(&self, conversation: &StoredConversation) -> Result<String> {
        let mut text = format!("Conversation: {}\n", conversation.title);
        text.push_str(&format!("Date: {}\n\n", conversation.last_updated));

        for (i, message) in conversation.messages.iter().enumerate() {
            let prefix = if message.is_user { "[User]" } else { "[AI]" };
            text.push_str(&format!("{} {}: {}\n", i + 1, prefix, message.content));
        }

        text.push_str(&format!(
            "\nToken Usage: {} ({} prompt + {} completion)\n",
            conversation.token_usage.total_tokens,
            conversation.token_usage.prompt_tokens,
            conversation.token_usage.completion_tokens
        ));

        Ok(text)
    }

    fn conversation_to_csv(&self, conversation: &StoredConversation) -> Result<String> {
        let mut csv = "Role,Content,Timestamp\n".to_string();

        for message in &conversation.messages {
            let role = if message.is_user { "User" } else { "Assistant" };
            let timestamp = message.timestamp.as_deref().unwrap_or("");
            let content = message.content.replace('"', "\"\"");
            csv.push_str(&format!("{},\"{}\",{}\n", role, content, timestamp));
        }

        Ok(csv)
    }

    pub fn cleanup_old_conversations(&self, max_count: usize) -> Result<usize> {
        let mut conversations = self.load_conversations()?;
        if conversations.len() <= max_count {
            return Ok(0);
        }

        // Sort by last updated time
        let mut conv_vec: Vec<_> = conversations.clone().into_iter().collect();
        conv_vec.sort_by(|a, b| b.1.last_updated.cmp(&a.1.last_updated));

        // Keep only the most recent conversations
        let to_remove = conv_vec.len() - max_count;
        for (id, _) in conv_vec.iter().skip(max_count) {
            conversations.remove(id);
        }

        self.save_conversations(&conversations)?;
        Ok(to_remove)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Markdown,
    Text,
    Csv,
}