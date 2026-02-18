use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotation {
    pub offset: i64,
    pub length: i64,
    pub verb: String,
    pub quote: String,
}
