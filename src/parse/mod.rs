pub mod delimiters;
pub mod reader;

mod amounts;
mod counts;
mod dates;
mod gcam;
mod locations;
mod names;
mod persons;
mod quotations;
mod themes;
mod tone;
mod translation;

use crate::error::NewsfreshError;
use crate::model::GkgRecord;

pub use reader::GkgReader;

pub fn parse_record(line: &str, line_number: usize) -> Result<GkgRecord, NewsfreshError> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() < 5 {
        return Err(NewsfreshError::Parse {
            line: line_number,
            message: format!("Expected at least 5 tab-delimited fields, got {}", fields.len()),
        });
    }

    let get = |i: usize| -> &str { fields.get(i).copied().unwrap_or("") };

    Ok(GkgRecord {
        gkg_record_id: get(0).to_string(),
        date: get(1).parse::<i64>().unwrap_or(0),
        source_collection_id: get(2).parse::<i32>().unwrap_or(0),
        source_common_name: get(3).to_string(),
        document_identifier: get(4).to_string(),
        v1_counts: counts::parse_counts_v1(get(5)),
        v21_counts: counts::parse_counts_v21(get(6)),
        v1_themes: themes::parse_themes_v1(get(7)),
        v2_enhanced_themes: themes::parse_enhanced_themes(get(8)),
        v1_locations: locations::parse_locations_v1(get(9)),
        v2_enhanced_locations: locations::parse_enhanced_locations(get(10)),
        v1_persons: persons::parse_persons_v1(get(11)),
        v2_enhanced_persons: persons::parse_enhanced_entities(get(12)),
        v1_organizations: persons::parse_persons_v1(get(13)),
        v2_enhanced_organizations: persons::parse_enhanced_entities(get(14)),
        tone: tone::parse_tone(get(15)),
        v21_enhanced_dates: dates::parse_enhanced_dates(get(16)),
        gcam: gcam::parse_gcam(get(17)),
        sharing_image: delimiters::non_empty(get(18)),
        related_images: delimiters::split_semicolon_list(get(19)),
        social_image_embeds: delimiters::split_semicolon_list(get(20)),
        social_video_embeds: delimiters::split_semicolon_list(get(21)),
        quotations: quotations::parse_quotations(get(22)),
        all_names: names::parse_names(get(23)),
        amounts: amounts::parse_amounts(get(24)),
        translation_info: translation::parse_translation_info(get(25)),
        extras_xml: delimiters::non_empty(get(26)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_fields_returns_err() {
        // Only 3 fields (2 tabs) — well under the 5-field minimum
        let line = "field1\tfield2\tfield3";
        let result = parse_record(line, 1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("at least 5"), "error should mention field count: {msg}");
    }

    #[test]
    fn minimal_five_field_record_parses_ok() {
        let line = "rec-001\t20250217120000\t1\tnytimes.com\thttps://nytimes.com/article";
        let record = parse_record(line, 1).expect("should parse with exactly 5 fields");
        assert_eq!(record.gkg_record_id, "rec-001");
        assert_eq!(record.date, 20250217120000);
        assert_eq!(record.source_collection_id, 1);
        assert_eq!(record.source_common_name, "nytimes.com");
        assert_eq!(record.document_identifier, "https://nytimes.com/article");
        // Remaining fields should be empty/default
        assert!(record.v1_themes.is_empty());
        assert!(record.v1_persons.is_empty());
        assert!(record.tone.is_none());
    }

    #[test]
    fn full_27_field_record_parses_key_fields() {
        // Build a tab-delimited string with 27 fields (indices 0..26)
        let fields = [
            "20250217120000-T1",                                        // 0: gkg_record_id
            "20250217120000",                                           // 1: date
            "1",                                                        // 2: source_collection_id
            "nytimes.com",                                              // 3: source_common_name
            "https://nytimes.com/test",                                 // 4: document_identifier
            "",                                                         // 5: v1_counts
            "",                                                         // 6: v21_counts
            "ELECTION;LEADER",                                          // 7: v1_themes
            "",                                                         // 8: v2_enhanced_themes
            "1#United States#US#US06#38.0#-97.0#US",                    // 9: v1_locations
            "",                                                         // 10: v2_enhanced_locations
            "donald trump",                                             // 11: v1_persons
            "",                                                         // 12: v2_enhanced_persons
            "congress",                                                 // 13: v1_organizations
            "",                                                         // 14: v2_enhanced_organizations
            "-1.5,2.0,3.5,5.5,10.0,0.5,500",                           // 15: tone
            "",                                                         // 16: v21_enhanced_dates
            "",                                                         // 17: gcam
            "",                                                         // 18: sharing_image
            "",                                                         // 19: related_images
            "",                                                         // 20: social_image_embeds
            "",                                                         // 21: social_video_embeds
            "",                                                         // 22: quotations
            "",                                                         // 23: all_names
            "",                                                         // 24: amounts
            "",                                                         // 25: translation_info
            "",                                                         // 26: extras_xml
        ];
        let line = fields.join("\t");

        let record = parse_record(&line, 42).expect("should parse full 27-field record");

        // Basic identity fields
        assert_eq!(record.gkg_record_id, "20250217120000-T1");
        assert_eq!(record.date, 20250217120000);
        assert_eq!(record.source_collection_id, 1);
        assert_eq!(record.source_common_name, "nytimes.com");
        assert_eq!(record.document_identifier, "https://nytimes.com/test");

        // Themes
        assert_eq!(record.v1_themes, vec!["ELECTION", "LEADER"]);

        // Locations
        assert_eq!(record.v1_locations.len(), 1);
        assert_eq!(record.v1_locations[0].full_name, "United States");
        assert_eq!(record.v1_locations[0].country_code, "US");

        // Persons and organizations
        assert_eq!(record.v1_persons, vec!["donald trump"]);
        assert_eq!(record.v1_organizations, vec!["congress"]);

        // Tone
        let tone = record.tone.expect("tone should be present");
        assert!((tone.tone - (-1.5)).abs() < 0.01);
    }
}
