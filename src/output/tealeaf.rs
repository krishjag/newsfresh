use std::io::Write;

use tealeaf::{FieldType, Schema, TeaLeafBuilder, Value as TlValue, ObjectMap};

use crate::error::NewsfreshError;
use crate::model::{GkgRecord, ScoredRecord};
use super::OutputFormatter;

pub struct TealeafFormatter {
    writer: Box<dyn Write>,
    records: Vec<TlValue>,
    compact: bool,
}

impl TealeafFormatter {
    pub fn new(writer: Box<dyn Write>, compact: bool) -> Self {
        Self {
            writer,
            records: Vec::new(),
            compact,
        }
    }
}

impl OutputFormatter for TealeafFormatter {
    fn begin(&mut self) -> Result<(), NewsfreshError> {
        Ok(())
    }

    fn write_record(&mut self, record: &GkgRecord) -> Result<(), NewsfreshError> {
        let json = serde_json::to_value(record)
            .map_err(|e| NewsfreshError::Other(e.to_string()))?;
        self.records.push(json_to_tealeaf(&json));
        Ok(())
    }

    fn write_scored_record(&mut self, scored: &ScoredRecord) -> Result<(), NewsfreshError> {
        let json = serde_json::to_value(scored)
            .map_err(|e| NewsfreshError::Other(e.to_string()))?;
        self.records.push(json_to_tealeaf(&json));
        Ok(())
    }

    fn finish(&mut self) -> Result<(), NewsfreshError> {
        let mut builder = TeaLeafBuilder::new();
        for schema in build_schemas() {
            builder = builder.add_schema(schema);
        }

        let records = std::mem::take(&mut self.records);
        builder = builder.add_value("records", TlValue::Array(records));

        let doc = builder.build();
        let output = if self.compact {
            doc.to_tl_with_schemas_compact()
        } else {
            doc.to_tl_with_schemas()
        };
        write!(self.writer, "{output}")?;
        self.writer.flush()?;
        Ok(())
    }
}

fn json_to_tealeaf(val: &serde_json::Value) -> TlValue {
    match val {
        serde_json::Value::Null => TlValue::Null,
        serde_json::Value::Bool(b) => TlValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                TlValue::Int(i)
            } else if let Some(u) = n.as_u64() {
                TlValue::UInt(u)
            } else {
                TlValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => TlValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            TlValue::Array(arr.iter().map(json_to_tealeaf).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: ObjectMap<String, TlValue> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_tealeaf(v)))
                .collect();
            TlValue::Object(map)
        }
    }
}

fn build_schemas() -> Vec<Schema> {
    vec![
        Schema::new("tone")
            .field("tone", FieldType::new("float"))
            .field("positive_score", FieldType::new("float"))
            .field("negative_score", FieldType::new("float"))
            .field("polarity", FieldType::new("float"))
            .field("activity_ref_density", FieldType::new("float"))
            .field("self_group_ref_density", FieldType::new("float"))
            .field("word_count", FieldType::new("int")),
        Schema::new("location_v1")
            .field("location_type", FieldType::new("int"))
            .field("full_name", FieldType::new("string"))
            .field("country_code", FieldType::new("string"))
            .field("adm1_code", FieldType::new("string"))
            .field("latitude", FieldType::new("float"))
            .field("longitude", FieldType::new("float"))
            .field("feature_id", FieldType::new("string")),
        Schema::new("enhanced_location")
            .field("location_type", FieldType::new("int"))
            .field("full_name", FieldType::new("string"))
            .field("country_code", FieldType::new("string"))
            .field("adm1_code", FieldType::new("string"))
            .field("adm2_code", FieldType::new("string"))
            .field("latitude", FieldType::new("float"))
            .field("longitude", FieldType::new("float"))
            .field("feature_id", FieldType::new("string"))
            .field("char_offset", FieldType::new("int")),
        Schema::new("count_v1")
            .field("count_type", FieldType::new("string"))
            .field("count", FieldType::new("int"))
            .field("object_type", FieldType::new("string"))
            .field("location", FieldType::new("location_v1")),
        Schema::new("count_v21")
            .field("count_type", FieldType::new("string"))
            .field("count", FieldType::new("int"))
            .field("object_type", FieldType::new("string"))
            .field("location", FieldType::new("location_v1"))
            .field("char_offset", FieldType::new("int")),
        Schema::new("enhanced_theme")
            .field("theme", FieldType::new("string"))
            .field("char_offset", FieldType::new("int")),
        Schema::new("enhanced_entity")
            .field("name", FieldType::new("string"))
            .field("char_offset", FieldType::new("int")),
        Schema::new("enhanced_date")
            .field("resolution", FieldType::new("int"))
            .field("month", FieldType::new("int"))
            .field("day", FieldType::new("int"))
            .field("year", FieldType::new("int"))
            .field("char_offset", FieldType::new("int")),
        Schema::new("gcam_entry")
            .field("dimension", FieldType::new("string"))
            .field("value", FieldType::new("float")),
        Schema::new("quotation")
            .field("offset", FieldType::new("int"))
            .field("length", FieldType::new("int"))
            .field("verb", FieldType::new("string"))
            .field("quote", FieldType::new("string")),
        Schema::new("name_entry")
            .field("name", FieldType::new("string"))
            .field("char_offset", FieldType::new("int")),
        Schema::new("amount_entry")
            .field("amount", FieldType::new("float"))
            .field("object", FieldType::new("string"))
            .field("char_offset", FieldType::new("int")),
        Schema::new("translation_info")
            .field("source_language", FieldType::new("string"))
            .field("engine", FieldType::new("string")),
        Schema::new("gkg_record")
            .field("gkg_record_id", FieldType::new("string"))
            .field("date", FieldType::new("int"))
            .field("source_collection_id", FieldType::new("int"))
            .field("source_common_name", FieldType::new("string"))
            .field("document_identifier", FieldType::new("string"))
            .field("v1_counts", FieldType::new("count_v1").array())
            .field("v21_counts", FieldType::new("count_v21").array())
            .field("v1_themes", FieldType::new("string").array())
            .field("v2_enhanced_themes", FieldType::new("enhanced_theme").array())
            .field("v1_locations", FieldType::new("location_v1").array())
            .field("v2_enhanced_locations", FieldType::new("enhanced_location").array())
            .field("v1_persons", FieldType::new("string").array())
            .field("v2_enhanced_persons", FieldType::new("enhanced_entity").array())
            .field("v1_organizations", FieldType::new("string").array())
            .field("v2_enhanced_organizations", FieldType::new("enhanced_entity").array())
            .field("tone", FieldType::new("tone").nullable())
            .field("v21_enhanced_dates", FieldType::new("enhanced_date").array())
            .field("gcam", FieldType::new("gcam_entry").array())
            .field("sharing_image", FieldType::new("string").nullable())
            .field("related_images", FieldType::new("string").array())
            .field("social_image_embeds", FieldType::new("string").array())
            .field("social_video_embeds", FieldType::new("string").array())
            .field("quotations", FieldType::new("quotation").array())
            .field("all_names", FieldType::new("name_entry").array())
            .field("amounts", FieldType::new("amount_entry").array())
            .field("translation_info", FieldType::new("translation_info").nullable())
            .field("extras_xml", FieldType::new("string").nullable()),
        Schema::new("scored_gkg_record")
            .field("relevance_score", FieldType::new("float"))
            .field("gkg_record_id", FieldType::new("string"))
            .field("date", FieldType::new("int"))
            .field("source_collection_id", FieldType::new("int"))
            .field("source_common_name", FieldType::new("string"))
            .field("document_identifier", FieldType::new("string"))
            .field("v1_counts", FieldType::new("count_v1").array())
            .field("v21_counts", FieldType::new("count_v21").array())
            .field("v1_themes", FieldType::new("string").array())
            .field("v2_enhanced_themes", FieldType::new("enhanced_theme").array())
            .field("v1_locations", FieldType::new("location_v1").array())
            .field("v2_enhanced_locations", FieldType::new("enhanced_location").array())
            .field("v1_persons", FieldType::new("string").array())
            .field("v2_enhanced_persons", FieldType::new("enhanced_entity").array())
            .field("v1_organizations", FieldType::new("string").array())
            .field("v2_enhanced_organizations", FieldType::new("enhanced_entity").array())
            .field("tone", FieldType::new("tone").nullable())
            .field("v21_enhanced_dates", FieldType::new("enhanced_date").array())
            .field("gcam", FieldType::new("gcam_entry").array())
            .field("sharing_image", FieldType::new("string").nullable())
            .field("related_images", FieldType::new("string").array())
            .field("social_image_embeds", FieldType::new("string").array())
            .field("social_video_embeds", FieldType::new("string").array())
            .field("quotations", FieldType::new("quotation").array())
            .field("all_names", FieldType::new("name_entry").array())
            .field("amounts", FieldType::new("amount_entry").array())
            .field("translation_info", FieldType::new("translation_info").nullable())
            .field("extras_xml", FieldType::new("string").nullable()),
    ]
}
