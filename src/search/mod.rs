mod adm1;
mod enrich;
pub(crate) mod fips;
mod tantivy;
mod themes;

use crate::error::NewsfreshError;
use crate::model::GkgRecord;

pub struct ScoredHit {
    pub record_index: usize,
    pub score: f32,
}

pub trait SearchEngine {
    fn build(&mut self, records: &[GkgRecord]) -> Result<(), NewsfreshError>;
    fn search(&self, query_str: &str, limit: usize) -> Result<Vec<ScoredHit>, NewsfreshError>;
}

pub fn create_engine() -> Box<dyn SearchEngine> {
    Box::new(tantivy::TantivyEngine::new())
}
