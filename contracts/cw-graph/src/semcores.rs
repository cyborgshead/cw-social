use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TypeDefinition {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<serde_json::Value>,
}

pub enum SemanticCore {
    Social,
    Chat,
    // Lens,
    Project,
    Deep,
    ChatGPT,
}

impl SemanticCore {
    pub fn get_types(&self) -> Vec<TypeDefinition> {
        let json_str = match self {
            SemanticCore::Social => include_str!("../semcores/social.json"),
            SemanticCore::Chat => include_str!("../semcores/chat.json"),
            // SemanticCore::Lens => include_str!("../semcores/lens.json"),
            SemanticCore::Project => include_str!("../semcores/project.json"),
            SemanticCore::Deep => include_str!("../semcores/deep.json"),
            SemanticCore::ChatGPT => include_str!("../semcores/chatgpt.json"),
        };

        // Parse JSON string into RawTypeDefinition entries
        let raw_definitions: Vec<TypeDefinition> = serde_json::from_str(json_str)
            .expect("Failed to parse semantic core JSON");

        // Filter only Type definitions that have an ID field
        raw_definitions
            .into_iter()
            .filter(|def| def.id.is_some() && def.type_ == "Type")
            .collect()
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "social" => Some(SemanticCore::Social),
            "chat" => Some(SemanticCore::Chat),
            "project" => Some(SemanticCore::Project),
            // "lens" => Some(SemanticCore::Lens),
            "deep" => Some(SemanticCore::Deep),
            "chatgpt" => Some(SemanticCore::ChatGPT),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_social_types() {
        let core = SemanticCore::Social;
        let types = core.get_types();
        assert!(!types.is_empty(), "Should load social types");
        
        // Verify some expected types
        let has_account = types.iter().any(|t| t.id.as_ref() == Some(&"Account".to_string()));
        let has_post = types.iter().any(|t| t.id.as_ref() == Some(&"Post".to_string()));
        assert!(has_account, "Should have Account type");
        assert!(has_post, "Should have Post type");
    }

    #[test]
    fn test_load_chat_types() {
        let core = SemanticCore::Chat;
        let types = core.get_types();
        assert!(!types.is_empty(), "Should load chat types");
        
        // Verify some expected types
        let has_chat = types.iter().any(|t| t.id.as_ref() == Some(&"Chat".to_string()));
        let has_message = types.iter().any(|t| t.id.as_ref() == Some(&"Message".to_string()));
        assert!(has_chat, "Should have Chat type");
        assert!(has_message, "Should have Message type");
    }
}
