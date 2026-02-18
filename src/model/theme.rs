use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTheme {
    pub theme: String,
    pub char_offset: i64,
}
