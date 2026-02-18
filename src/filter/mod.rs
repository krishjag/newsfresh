pub mod predicates;

use crate::model::GkgRecord;

pub trait RecordFilter: Send + Sync {
    fn matches(&self, record: &GkgRecord) -> bool;
}

pub struct CompositeFilter {
    filters: Vec<Box<dyn RecordFilter>>,
}

impl CompositeFilter {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn add(&mut self, filter: Box<dyn RecordFilter>) {
        self.filters.push(filter);
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

impl RecordFilter for CompositeFilter {
    fn matches(&self, record: &GkgRecord) -> bool {
        self.filters.iter().all(|f| f.matches(record))
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

    /// A simple filter that always returns the given value.
    struct ConstFilter(bool);
    impl RecordFilter for ConstFilter {
        fn matches(&self, _record: &GkgRecord) -> bool {
            self.0
        }
    }

    #[test]
    fn composite_new_is_empty() {
        let cf = CompositeFilter::new();
        assert!(cf.is_empty());
    }

    #[test]
    fn empty_composite_matches_any_record() {
        let cf = CompositeFilter::new();
        let record = make_test_record();
        assert!(cf.matches(&record));
    }

    #[test]
    fn composite_and_logic_all_pass() {
        let mut cf = CompositeFilter::new();
        cf.add(Box::new(ConstFilter(true)));
        cf.add(Box::new(ConstFilter(true)));
        assert!(!cf.is_empty());
        let record = make_test_record();
        assert!(cf.matches(&record));
    }

    #[test]
    fn composite_and_logic_one_fails() {
        let mut cf = CompositeFilter::new();
        cf.add(Box::new(ConstFilter(true)));
        cf.add(Box::new(ConstFilter(false)));
        let record = make_test_record();
        assert!(!cf.matches(&record));
    }

    #[test]
    fn composite_and_logic_all_fail() {
        let mut cf = CompositeFilter::new();
        cf.add(Box::new(ConstFilter(false)));
        cf.add(Box::new(ConstFilter(false)));
        let record = make_test_record();
        assert!(!cf.matches(&record));
    }
}
