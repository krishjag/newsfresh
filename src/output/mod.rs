pub mod field_select;
pub mod json;
pub mod schema;
pub mod tealeaf;

use std::io::Write;

use crate::error::NewsfreshError;
use crate::model::{GkgRecord, ScoredRecord};

pub trait OutputFormatter {
    fn begin(&mut self) -> Result<(), NewsfreshError>;
    fn write_record(&mut self, record: &GkgRecord) -> Result<(), NewsfreshError>;
    fn write_scored_record(&mut self, scored: &ScoredRecord) -> Result<(), NewsfreshError> {
        self.write_record(&scored.record)
    }
    fn finish(&mut self) -> Result<(), NewsfreshError>;
}

pub fn create_formatter(
    format: &str,
    writer: Box<dyn Write>,
    fields: &Option<Vec<String>>,
) -> Box<dyn OutputFormatter> {
    match format {
        "tealeaf" => Box::new(tealeaf::TealeafFormatter::new(writer, false)),
        "tealeaf-compact" => Box::new(tealeaf::TealeafFormatter::new(writer, true)),
        _ => Box::new(json::JsonFormatter::new(writer, format != "json-compact", fields.clone())),
    }
}
