use serde::{Deserialize, Serialize};

use super::amount::AmountEntry;
use super::count::{CountV1, CountV21};
use super::date::EnhancedDate;
use super::gcam::GcamEntry;
use super::location::{EnhancedLocation, LocationV1};
use super::name::NameEntry;
use super::person::EnhancedEntity;
use super::quotation::Quotation;
use super::theme::EnhancedTheme;
use super::tone::Tone;
use super::translation::TranslationInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredRecord {
    pub relevance_score: f32,
    #[serde(flatten)]
    pub record: GkgRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GkgRecord {
    pub gkg_record_id: String,
    pub date: i64,
    pub source_collection_id: i32,
    pub source_common_name: String,
    pub document_identifier: String,
    pub v1_counts: Vec<CountV1>,
    pub v21_counts: Vec<CountV21>,
    pub v1_themes: Vec<String>,
    pub v2_enhanced_themes: Vec<EnhancedTheme>,
    pub v1_locations: Vec<LocationV1>,
    pub v2_enhanced_locations: Vec<EnhancedLocation>,
    pub v1_persons: Vec<String>,
    pub v2_enhanced_persons: Vec<EnhancedEntity>,
    pub v1_organizations: Vec<String>,
    pub v2_enhanced_organizations: Vec<EnhancedEntity>,
    pub tone: Option<Tone>,
    pub v21_enhanced_dates: Vec<EnhancedDate>,
    pub gcam: Vec<GcamEntry>,
    pub sharing_image: Option<String>,
    pub related_images: Vec<String>,
    pub social_image_embeds: Vec<String>,
    pub social_video_embeds: Vec<String>,
    pub quotations: Vec<Quotation>,
    pub all_names: Vec<NameEntry>,
    pub amounts: Vec<AmountEntry>,
    pub translation_info: Option<TranslationInfo>,
    pub extras_xml: Option<String>,
}
