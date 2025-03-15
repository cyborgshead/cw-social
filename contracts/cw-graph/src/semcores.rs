use serde::{Deserialize, Serialize};
use cosmwasm_std::{Timestamp};
use serde_json;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TypeDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<serde_json::Value>,
}

pub enum SemanticCore {
    Social,
    Chat,
    Lens,
}

impl SemanticCore {
    pub fn get_types(&self) -> Vec<TypeDefinition> {
        let json_str = match self {
            SemanticCore::Social => include_str!("../semcores/social_example.json"),
            SemanticCore::Chat => include_str!("../semcores/chat_example.json"),
            SemanticCore::Lens => include_str!("../semcores/lens.json"),
        };

        // Parse JSON string into Vec<TypeDefinition>
        let definitions: Vec<TypeDefinition> = serde_json::from_str(json_str)
            .expect("Failed to parse semantic core JSON");

        // Filter only Type definitions
        definitions
            .into_iter()
            .filter(|def| def.type_ == "Type")
            .collect()
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "social" => Some(SemanticCore::Social),
            "chat" => Some(SemanticCore::Chat),
            "lens" => Some(SemanticCore::Lens),
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
        let has_account = types.iter().any(|t| t.id == "Account");
        let has_post = types.iter().any(|t| t.id == "Post");
        assert!(has_account, "Should have Account type");
        assert!(has_post, "Should have Post type");
    }

    #[test]
    fn test_load_chat_types() {
        let core = SemanticCore::Chat;
        let types = core.get_types();
        assert!(!types.is_empty(), "Should load chat types");
        
        // Verify some expected types
        let has_chat = types.iter().any(|t| t.id == "Chat");
        let has_message = types.iter().any(|t| t.id == "Message");
        assert!(has_chat, "Should have Chat type");
        assert!(has_message, "Should have Message type");
    }
}
