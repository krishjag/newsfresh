use crate::model::GkgRecord;

use super::adm1;
use super::fips;
use super::themes;

/// Enriched, canonicalized text fields ready for indexing.
/// Each field contains the original data plus expanded canonical forms.
pub struct EnrichedText {
    pub persons: String,
    pub organizations: String,
    pub themes: String,
    pub locations: String,
    pub quotations: String,
    pub names: String,
    pub source: String,
    pub document_id: String,
}

/// Converts a GkgRecord into enriched, search-friendly text.
///
/// Applies three canonicalization layers:
/// 1. FIPS country codes → full country names (e.g. "US" → "United States")
/// 2. Theme prefixes → human-readable terms (e.g. "TAX_FNCACT_PRESIDENT" → "PRESIDENT")
/// 3. ADM1 codes → state/province names (e.g. "US06" → "California")
pub fn enrich_record(record: &GkgRecord) -> EnrichedText {
    EnrichedText {
        persons: collect_persons(record),
        organizations: collect_organizations(record),
        themes: collect_themes(record),
        locations: collect_locations(record),
        quotations: collect_quotations(record),
        names: collect_names(record),
        source: record.source_common_name.clone(),
        document_id: record.document_identifier.clone(),
    }
}

fn collect_persons(record: &GkgRecord) -> String {
    let mut parts: Vec<&str> = record.v1_persons.iter().map(|s| s.as_str()).collect();
    for e in &record.v2_enhanced_persons {
        parts.push(&e.name);
    }
    parts.join(" ")
}

fn collect_organizations(record: &GkgRecord) -> String {
    let mut parts: Vec<&str> = record.v1_organizations.iter().map(|s| s.as_str()).collect();
    for e in &record.v2_enhanced_organizations {
        parts.push(&e.name);
    }
    parts.join(" ")
}

fn collect_themes(record: &GkgRecord) -> String {
    let mut parts = Vec::new();

    for theme in &record.v1_themes {
        parts.push(themes::canonicalize_theme(theme));
    }
    for e in &record.v2_enhanced_themes {
        parts.push(themes::canonicalize_theme(&e.theme));
    }

    parts.join(" ")
}

fn collect_locations(record: &GkgRecord) -> String {
    let mut parts = Vec::new();

    for loc in &record.v1_locations {
        parts.push(loc.full_name.clone());
        // Layer 1: expand country code
        if let Some(country) = fips::country_name(&loc.country_code) {
            parts.push(country.to_string());
        }
        // Layer 3: expand ADM1 code
        if !loc.adm1_code.is_empty()
            && let Some(adm1) = adm1::adm1_name(&loc.adm1_code)
        {
            parts.push(adm1.to_string());
        }
    }

    for loc in &record.v2_enhanced_locations {
        parts.push(loc.full_name.clone());
        if let Some(country) = fips::country_name(&loc.country_code) {
            parts.push(country.to_string());
        }
        if !loc.adm1_code.is_empty()
            && let Some(adm1) = adm1::adm1_name(&loc.adm1_code)
        {
            parts.push(adm1.to_string());
        }
    }

    parts.join(" ")
}

fn collect_quotations(record: &GkgRecord) -> String {
    record
        .quotations
        .iter()
        .map(|q| q.quote.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_names(record: &GkgRecord) -> String {
    record
        .all_names
        .iter()
        .map(|n| n.name.as_str())
        .collect::<Vec<_>>()
        .join(" ")
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
    fn test_enrich_populated_record() {
        let record = make_test_record();
        let enriched = enrich_record(&record);
        assert!(enriched.persons.contains("donald trump"));
        assert!(enriched.locations.contains("California"));
        assert!(enriched.themes.contains("ELECTION"));
    }

    #[test]
    fn test_enrich_empty_record() {
        let record = GkgRecord {
            gkg_record_id: String::new(),
            date: 0,
            source_collection_id: 0,
            source_common_name: String::new(),
            document_identifier: String::new(),
            v1_persons: vec![],
            v2_enhanced_persons: vec![],
            v1_organizations: vec![],
            v2_enhanced_organizations: vec![],
            v1_themes: vec![],
            v2_enhanced_themes: vec![],
            v1_locations: vec![],
            v2_enhanced_locations: vec![],
            tone: None,
            quotations: vec![],
            sharing_image: None,
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
        };
        let enriched = enrich_record(&record);
        assert!(enriched.persons.is_empty());
        assert!(enriched.organizations.is_empty());
        assert!(enriched.themes.is_empty());
        assert!(enriched.locations.is_empty());
        assert!(enriched.quotations.is_empty());
        assert!(enriched.names.is_empty());
    }
}
