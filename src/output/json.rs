use std::io::Write;

use super::OutputFormatter;
use super::field_select::project_record;
use crate::error::NewsfreshError;
use crate::model::{GkgRecord, ScoredRecord};

pub struct JsonFormatter {
    writer: Box<dyn Write>,
    pretty: bool,
    first: bool,
    fields: Option<Vec<String>>,
}

impl JsonFormatter {
    pub fn new(writer: Box<dyn Write>, pretty: bool, fields: Option<Vec<String>>) -> Self {
        Self {
            writer,
            pretty,
            first: true,
            fields,
        }
    }
}

impl OutputFormatter for JsonFormatter {
    fn begin(&mut self) -> Result<(), NewsfreshError> {
        writeln!(self.writer, "[")?;
        Ok(())
    }

    fn write_record(&mut self, record: &GkgRecord) -> Result<(), NewsfreshError> {
        if !self.first {
            writeln!(self.writer, ",")?;
        }
        self.first = false;

        let json_str = if let Some(ref fields) = self.fields {
            let projected = project_record(record, fields)?;
            if self.pretty {
                serde_json::to_string_pretty(&projected)?
            } else {
                serde_json::to_string(&projected)?
            }
        } else if self.pretty {
            serde_json::to_string_pretty(record)?
        } else {
            serde_json::to_string(record)?
        };

        write!(self.writer, "{json_str}")?;
        Ok(())
    }

    fn write_scored_record(&mut self, scored: &ScoredRecord) -> Result<(), NewsfreshError> {
        if !self.first {
            writeln!(self.writer, ",")?;
        }
        self.first = false;

        let json_str = if self.pretty {
            serde_json::to_string_pretty(scored)?
        } else {
            serde_json::to_string(scored)?
        };

        write!(self.writer, "{json_str}")?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), NewsfreshError> {
        writeln!(self.writer, "\n]")?;
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_test_record() -> GkgRecord {
        GkgRecord {
            gkg_record_id: "20250217-1".into(),
            date: 20250217120000,
            source_collection_id: 1,
            source_common_name: "nytimes.com".into(),
            document_identifier: "https://nytimes.com/article".into(),
            v1_persons: vec!["donald trump".into()],
            v2_enhanced_persons: vec![EnhancedEntity {
                name: "elon musk".into(),
                char_offset: 100,
            }],
            v1_organizations: vec!["congress".into()],
            v2_enhanced_organizations: vec![],
            v1_themes: vec!["LEADER".into(), "TAX_FNCACT_PRESIDENT".into()],
            v2_enhanced_themes: vec![EnhancedTheme {
                theme: "ELECTION".into(),
                char_offset: 50,
            }],
            v1_locations: vec![LocationV1 {
                location_type: 1,
                full_name: "United States".into(),
                country_code: "US".into(),
                adm1_code: "US06".into(),
                latitude: 38.0,
                longitude: -97.0,
                feature_id: "US".into(),
            }],
            v2_enhanced_locations: vec![],
            tone: Some(Tone {
                tone: -1.5,
                positive_score: 2.0,
                negative_score: 3.5,
                polarity: 5.5,
                activity_ref_density: 10.0,
                self_group_ref_density: 0.5,
                word_count: 500,
            }),
            quotations: vec![Quotation {
                offset: 10,
                length: 50,
                verb: "said".into(),
                quote: "test quote".into(),
            }],
            sharing_image: Some("https://img.example.com/photo.jpg".into()),
            v1_counts: vec![],
            v21_counts: vec![],
            v21_enhanced_dates: vec![],
            gcam: vec![],
            related_images: vec![],
            social_image_embeds: vec![],
            social_video_embeds: vec![],
            all_names: vec![],
            amounts: vec![],
            translation_info: None,
            extras_xml: None,
        }
    }

    #[test]
    fn test_json_pretty_no_error() {
        let record = make_test_record();
        let mut fmt = JsonFormatter::new(Box::new(Vec::new()), true, None);
        fmt.begin().unwrap();
        fmt.write_record(&record).unwrap();
        fmt.finish().unwrap();
    }

    #[test]
    fn test_json_compact_no_error() {
        let record = make_test_record();
        let mut fmt = JsonFormatter::new(Box::new(Vec::new()), false, None);
        fmt.begin().unwrap();
        fmt.write_record(&record).unwrap();
        fmt.finish().unwrap();
    }
}
