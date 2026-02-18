use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDate {
    pub resolution: i32,
    pub month: i32,
    pub day: i32,
    pub year: i32,
    pub char_offset: i64,
}
