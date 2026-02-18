use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameEntry {
    pub name: String,
    pub char_offset: i64,
}
