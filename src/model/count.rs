use serde::{Deserialize, Serialize};

use super::location::LocationV1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountV1 {
    pub count_type: String,
    pub count: i64,
    pub object_type: String,
    pub location: LocationV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountV21 {
    pub count_type: String,
    pub count: i64,
    pub object_type: String,
    pub location: LocationV1,
    pub char_offset: i64,
}
