use serde::{Deserialize, Serialize};

/// Shared type for enhanced persons and organizations (name + character offset).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedEntity {
    pub name: String,
    pub char_offset: i64,
}
