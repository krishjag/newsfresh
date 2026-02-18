use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountEntry {
    pub amount: f64,
    pub object: String,
    pub char_offset: i64,
}
