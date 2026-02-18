use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tone {
    pub tone: f64,
    pub positive_score: f64,
    pub negative_score: f64,
    pub polarity: f64,
    pub activity_ref_density: f64,
    pub self_group_ref_density: f64,
    pub word_count: i64,
}
