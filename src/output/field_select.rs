use crate::error::NewsfreshError;
use crate::model::GkgRecord;

pub fn project_record(
    record: &GkgRecord,
    fields: &[String],
) -> Result<serde_json::Value, NewsfreshError> {
    let full = serde_json::to_value(record)?;
    let Some(obj) = full.as_object() else {
        return Ok(full);
    };
    let mut projected = serde_json::Map::new();
    for field in fields {
        if let Some(val) = obj.get(field) {
            projected.insert(field.clone(), val.clone());
        }
    }
    Ok(serde_json::Value::Object(projected))
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
            v2_enhanced_persons: vec![EnhancedEntity { name: "elon musk".into(), char_offset: 100 }],
            v1_organizations: vec!["congress".into()],
            v2_enhanced_organizations: vec![],
            v1_themes: vec!["LEADER".into(), "TAX_FNCACT_PRESIDENT".into()],
            v2_enhanced_themes: vec![EnhancedTheme { theme: "ELECTION".into(), char_offset: 50 }],
            v1_locations: vec![LocationV1 {
                location_type: 1, full_name: "United States".into(),
                country_code: "US".into(), adm1_code: "US06".into(),
                latitude: 38.0, longitude: -97.0, feature_id: "US".into(),
            }],
            v2_enhanced_locations: vec![],
            tone: Some(Tone { tone: -1.5, positive_score: 2.0, negative_score: 3.5, polarity: 5.5, activity_ref_density: 10.0, self_group_ref_density: 0.5, word_count: 500 }),
            quotations: vec![Quotation { offset: 10, length: 50, verb: "said".into(), quote: "test quote".into() }],
            sharing_image: Some("https://img.example.com/photo.jpg".into()),
            v1_counts: vec![], v21_counts: vec![], v21_enhanced_dates: vec![],
            gcam: vec![], related_images: vec![], social_image_embeds: vec![],
            social_video_embeds: vec![], all_names: vec![], amounts: vec![],
            translation_info: None, extras_xml: None,
        }
    }

    #[test]
    fn test_project_specific_fields() {
        let record = make_test_record();
        let fields = vec!["document_identifier".to_string(), "source_common_name".to_string()];
        let result = project_record(&record, &fields).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("document_identifier"));
        assert!(obj.contains_key("source_common_name"));
    }

    #[test]
    fn test_project_nonexistent_field() {
        let record = make_test_record();
        let fields = vec!["nonexistent".to_string()];
        let result = project_record(&record, &fields).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.is_empty());
    }
}
