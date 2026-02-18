use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, ReloadPolicy};

use crate::error::NewsfreshError;
use crate::model::GkgRecord;

use super::enrich;
use super::ScoredHit;

struct GkgSearchSchema {
    schema: Schema,
    record_idx: Field,
    persons: Field,
    organizations: Field,
    themes: Field,
    locations: Field,
    quotations: Field,
    names: Field,
    source: Field,
    document_id: Field,
}

impl GkgSearchSchema {
    fn new() -> Self {
        let mut builder = Schema::builder();
        let record_idx = builder.add_u64_field("record_idx", STORED);
        let persons = builder.add_text_field("persons", TEXT);
        let organizations = builder.add_text_field("organizations", TEXT);
        let themes = builder.add_text_field("themes", TEXT);
        let locations = builder.add_text_field("locations", TEXT);
        let quotations = builder.add_text_field("quotations", TEXT);
        let names = builder.add_text_field("names", TEXT);
        let source = builder.add_text_field("source", TEXT);
        let document_id = builder.add_text_field("document_id", TEXT);
        let schema = builder.build();

        Self {
            schema,
            record_idx,
            persons,
            organizations,
            themes,
            locations,
            quotations,
            names,
            source,
            document_id,
        }
    }

    fn all_text_fields(&self) -> Vec<Field> {
        vec![
            self.persons,
            self.organizations,
            self.themes,
            self.locations,
            self.quotations,
            self.names,
            self.source,
            self.document_id,
        ]
    }
}

pub struct TantivyEngine {
    search_schema: GkgSearchSchema,
    index: Option<Index>,
}

impl TantivyEngine {
    pub fn new() -> Self {
        Self {
            search_schema: GkgSearchSchema::new(),
            index: None,
        }
    }
}

impl super::SearchEngine for TantivyEngine {
    fn build(&mut self, records: &[GkgRecord]) -> Result<(), NewsfreshError> {
        let index = Index::create_in_ram(self.search_schema.schema.clone());
        let mut writer = index
            .writer_with_num_threads(1, 50_000_000)
            .map_err(|e| NewsfreshError::Other(format!("Failed to create index writer: {e}")))?;

        for (idx, record) in records.iter().enumerate() {
            let enriched = enrich::enrich_record(record);
            writer
                .add_document(doc!(
                    self.search_schema.record_idx => idx as u64,
                    self.search_schema.persons => enriched.persons,
                    self.search_schema.organizations => enriched.organizations,
                    self.search_schema.themes => enriched.themes,
                    self.search_schema.locations => enriched.locations,
                    self.search_schema.quotations => enriched.quotations,
                    self.search_schema.names => enriched.names,
                    self.search_schema.source => enriched.source,
                    self.search_schema.document_id => enriched.document_id,
                ))
                .map_err(|e| NewsfreshError::Other(format!("Failed to add document: {e}")))?;
        }

        writer
            .commit()
            .map_err(|e| NewsfreshError::Other(format!("Failed to commit index: {e}")))?;

        self.index = Some(index);
        Ok(())
    }

    fn search(&self, query_str: &str, limit: usize) -> Result<Vec<ScoredHit>, NewsfreshError> {
        let index = self
            .index
            .as_ref()
            .ok_or_else(|| NewsfreshError::Other("Index not built yet".to_string()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e: tantivy::TantivyError| {
                NewsfreshError::Other(format!("Failed to create reader: {e}"))
            })?;

        let searcher = reader.searcher();

        let query_parser =
            QueryParser::for_index(index, self.search_schema.all_text_fields());
        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| NewsfreshError::Other(format!("Failed to parse query: {e}")))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| NewsfreshError::Other(format!("Search failed: {e}")))?;

        let mut hits = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| NewsfreshError::Other(format!("Failed to retrieve doc: {e}")))?;

            if let Some(idx_value) = doc.get_first(self.search_schema.record_idx) {
                if let Some(idx) = idx_value.as_u64() {
                    hits.push(ScoredHit {
                        record_index: idx as usize,
                        score,
                    });
                }
            }
        }

        Ok(hits)
    }
}
