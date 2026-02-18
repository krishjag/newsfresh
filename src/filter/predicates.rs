use crate::model::GkgRecord;
use super::RecordFilter;

pub struct PersonFilter {
    pub pattern: String,
}

impl RecordFilter for PersonFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        let pat = self.pattern.to_lowercase();
        record
            .v1_persons
            .iter()
            .any(|p| p.to_lowercase().contains(&pat))
            || record
                .v2_enhanced_persons
                .iter()
                .any(|p| p.name.to_lowercase().contains(&pat))
    }
}

pub struct OrgFilter {
    pub pattern: String,
}

impl RecordFilter for OrgFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        let pat = self.pattern.to_lowercase();
        record
            .v1_organizations
            .iter()
            .any(|o| o.to_lowercase().contains(&pat))
            || record
                .v2_enhanced_organizations
                .iter()
                .any(|o| o.name.to_lowercase().contains(&pat))
    }
}

pub struct ThemeFilter {
    pub pattern: String,
}

impl RecordFilter for ThemeFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        let pat = self.pattern.to_uppercase();
        record.v1_themes.iter().any(|t| t.to_uppercase().contains(&pat))
            || record
                .v2_enhanced_themes
                .iter()
                .any(|t| t.theme.to_uppercase().contains(&pat))
    }
}

pub struct LocationFilter {
    pub pattern: String,
}

impl RecordFilter for LocationFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        let pat = self.pattern.to_lowercase();
        record
            .v1_locations
            .iter()
            .any(|l| l.full_name.to_lowercase().contains(&pat))
            || record
                .v2_enhanced_locations
                .iter()
                .any(|l| l.full_name.to_lowercase().contains(&pat))
    }
}

pub struct CountryFilter {
    pub code: String,
}

impl RecordFilter for CountryFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        let code = self.code.to_uppercase();
        record
            .v1_locations
            .iter()
            .any(|l| l.country_code.to_uppercase() == code)
            || record
                .v2_enhanced_locations
                .iter()
                .any(|l| l.country_code.to_uppercase() == code)
    }
}

pub struct ToneRangeFilter {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl RecordFilter for ToneRangeFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        let Some(ref tone) = record.tone else {
            return false;
        };
        if let Some(min) = self.min {
            if tone.tone < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if tone.tone > max {
                return false;
            }
        }
        true
    }
}

pub struct DateRangeFilter {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

impl RecordFilter for DateRangeFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        if let Some(from) = self.from {
            if record.date < from {
                return false;
            }
        }
        if let Some(to) = self.to {
            if record.date > to {
                return false;
            }
        }
        true
    }
}

pub struct SourceFilter {
    pub pattern: String,
}

impl RecordFilter for SourceFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        record
            .source_common_name
            .to_lowercase()
            .contains(&self.pattern.to_lowercase())
    }
}

pub struct HasImageFilter;

impl RecordFilter for HasImageFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        record.sharing_image.is_some()
    }
}

pub struct HasQuoteFilter;

impl RecordFilter for HasQuoteFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        !record.quotations.is_empty()
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

    // ---- PersonFilter ----

    #[test]
    fn person_filter_matches_case_insensitive() {
        let filter = PersonFilter { pattern: "trump".into() };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn person_filter_no_match() {
        let filter = PersonFilter { pattern: "obama".into() };
        assert!(!filter.matches(&make_test_record()));
    }

    // ---- OrgFilter ----

    #[test]
    fn org_filter_matches() {
        let filter = OrgFilter { pattern: "congress".into() };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn org_filter_no_match() {
        let filter = OrgFilter { pattern: "pentagon".into() };
        assert!(!filter.matches(&make_test_record()));
    }

    // ---- ThemeFilter ----

    #[test]
    fn theme_filter_matches() {
        let filter = ThemeFilter { pattern: "LEADER".into() };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn theme_filter_no_match() {
        let filter = ThemeFilter { pattern: "CLIMATE".into() };
        assert!(!filter.matches(&make_test_record()));
    }

    // ---- LocationFilter ----

    #[test]
    fn location_filter_matches() {
        let filter = LocationFilter { pattern: "United States".into() };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn location_filter_no_match() {
        let filter = LocationFilter { pattern: "London".into() };
        assert!(!filter.matches(&make_test_record()));
    }

    // ---- CountryFilter ----

    #[test]
    fn country_filter_matches() {
        let filter = CountryFilter { code: "US".into() };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn country_filter_no_match() {
        let filter = CountryFilter { code: "UK".into() };
        assert!(!filter.matches(&make_test_record()));
    }

    // ---- ToneRangeFilter ----

    #[test]
    fn tone_range_filter_matches_in_range() {
        let filter = ToneRangeFilter { min: Some(-5.0), max: Some(0.0) };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn tone_range_filter_no_match_out_of_range() {
        let filter = ToneRangeFilter { min: Some(0.0), max: Some(5.0) };
        assert!(!filter.matches(&make_test_record()));
    }

    #[test]
    fn tone_range_filter_no_tone_returns_false() {
        let filter = ToneRangeFilter { min: Some(-5.0), max: Some(0.0) };
        let mut record = make_test_record();
        record.tone = None;
        assert!(!filter.matches(&record));
    }

    // ---- DateRangeFilter ----

    #[test]
    fn date_range_filter_matches_in_range() {
        let filter = DateRangeFilter { from: Some(20250101000000), to: Some(20250301000000) };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn date_range_filter_no_match_out_of_range() {
        let filter = DateRangeFilter { from: Some(20250301000000), to: None };
        assert!(!filter.matches(&make_test_record()));
    }

    // ---- SourceFilter ----

    #[test]
    fn source_filter_matches() {
        let filter = SourceFilter { pattern: "nytimes".into() };
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn source_filter_no_match() {
        let filter = SourceFilter { pattern: "bbc".into() };
        assert!(!filter.matches(&make_test_record()));
    }

    // ---- HasImageFilter ----

    #[test]
    fn has_image_filter_matches() {
        let filter = HasImageFilter;
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn has_image_filter_no_match() {
        let filter = HasImageFilter;
        let mut record = make_test_record();
        record.sharing_image = None;
        assert!(!filter.matches(&record));
    }

    // ---- HasQuoteFilter ----

    #[test]
    fn has_quote_filter_matches() {
        let filter = HasQuoteFilter;
        assert!(filter.matches(&make_test_record()));
    }

    #[test]
    fn has_quote_filter_no_match() {
        let filter = HasQuoteFilter;
        let mut record = make_test_record();
        record.quotations = vec![];
        assert!(!filter.matches(&record));
    }
}
