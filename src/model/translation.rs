use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationInfo {
    pub source_language: String,
    pub engine: String,
}
