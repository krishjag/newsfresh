use std::io::Write;

use crate::error::NewsfreshError;

pub fn print_tealeaf_schema(writer: &mut dyn Write) -> Result<(), NewsfreshError> {
    writeln!(writer, "# GDELT GKG v2.1 TeaLeaf Schema")?;
    writeln!(writer)?;
    writeln!(writer, "@struct tone (tone: float, positive_score: float, negative_score: float, polarity: float, activity_ref_density: float, self_group_ref_density: float, word_count: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct location_v1 (location_type: int, full_name: string, country_code: string, adm1_code: string, latitude: float, longitude: float, feature_id: string)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct enhanced_location (location_type: int, full_name: string, country_code: string, adm1_code: string, adm2_code: string, latitude: float, longitude: float, feature_id: string, char_offset: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct count_v1 (count_type: string, count: int, object_type: string, location: location_v1)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct count_v21 (count_type: string, count: int, object_type: string, location: location_v1, char_offset: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct enhanced_theme (theme: string, char_offset: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct enhanced_entity (name: string, char_offset: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct enhanced_date (resolution: int, month: int, day: int, year: int, char_offset: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct gcam_entry (dimension: string, value: float)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct quotation (offset: int, length: int, verb: string, quote: string)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct name_entry (name: string, char_offset: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct amount_entry (amount: float, object: string, char_offset: int)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct translation_info (source_language: string, engine: string)")?;
    writeln!(writer)?;
    writeln!(writer, "@struct gkg_record (")?;
    writeln!(writer, "  gkg_record_id: string,")?;
    writeln!(writer, "  date: int,")?;
    writeln!(writer, "  source_collection_id: int,")?;
    writeln!(writer, "  source_common_name: string,")?;
    writeln!(writer, "  document_identifier: string,")?;
    writeln!(writer, "  v1_counts: []count_v1,")?;
    writeln!(writer, "  v21_counts: []count_v21,")?;
    writeln!(writer, "  v1_themes: []string,")?;
    writeln!(writer, "  v2_enhanced_themes: []enhanced_theme,")?;
    writeln!(writer, "  v1_locations: []location_v1,")?;
    writeln!(writer, "  v2_enhanced_locations: []enhanced_location,")?;
    writeln!(writer, "  v1_persons: []string,")?;
    writeln!(writer, "  v2_enhanced_persons: []enhanced_entity,")?;
    writeln!(writer, "  v1_organizations: []string,")?;
    writeln!(writer, "  v2_enhanced_organizations: []enhanced_entity,")?;
    writeln!(writer, "  tone: tone?,")?;
    writeln!(writer, "  v21_enhanced_dates: []enhanced_date,")?;
    writeln!(writer, "  gcam: []gcam_entry,")?;
    writeln!(writer, "  sharing_image: string?,")?;
    writeln!(writer, "  related_images: []string,")?;
    writeln!(writer, "  social_image_embeds: []string,")?;
    writeln!(writer, "  social_video_embeds: []string,")?;
    writeln!(writer, "  quotations: []quotation,")?;
    writeln!(writer, "  all_names: []name_entry,")?;
    writeln!(writer, "  amounts: []amount_entry,")?;
    writeln!(writer, "  translation_info: translation_info?,")?;
    writeln!(writer, "  extras_xml: string?")?;
    writeln!(writer, ")")?;
    Ok(())
}

pub fn print_json_schema(writer: &mut dyn Write) -> Result<(), NewsfreshError> {
    let schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "GkgRecord",
        "description": "GDELT Global Knowledge Graph v2.1 Record",
        "type": "object",
        "properties": {
            "gkg_record_id": { "type": "string" },
            "date": { "type": "integer", "description": "YYYYMMDDHHMMSS" },
            "source_collection_id": { "type": "integer", "enum": [0,1,2,3,4,5,6] },
            "source_common_name": { "type": "string" },
            "document_identifier": { "type": "string" },
            "v1_themes": { "type": "array", "items": { "type": "string" } },
            "v1_persons": { "type": "array", "items": { "type": "string" } },
            "v1_organizations": { "type": "array", "items": { "type": "string" } },
            "tone": {
                "type": ["object", "null"],
                "properties": {
                    "tone": { "type": "number" },
                    "positive_score": { "type": "number" },
                    "negative_score": { "type": "number" },
                    "polarity": { "type": "number" },
                    "activity_ref_density": { "type": "number" },
                    "self_group_ref_density": { "type": "number" },
                    "word_count": { "type": "integer" }
                }
            }
        }
    });
    let output = serde_json::to_string_pretty(&schema)?;
    writeln!(writer, "{output}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_tealeaf_schema_writes_output() {
        let mut buf: Vec<u8> = Vec::new();
        print_tealeaf_schema(&mut buf).unwrap();
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_print_json_schema_writes_output() {
        let mut buf: Vec<u8> = Vec::new();
        print_json_schema(&mut buf).unwrap();
        assert!(!buf.is_empty());
    }
}
