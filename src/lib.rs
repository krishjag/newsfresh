//! # newsfresh
//!
//! A CLI tool and Rust library for querying, filtering, and analyzing
//! [GDELT Global Knowledge Graph (GKG) v2.1](https://blog.gdeltproject.org/gdelt-2-0-our-global-world-in-realtime/)
//! data — the world's largest open dataset of global news events, updated every 15 minutes.
//!
//! ## What is GDELT GKG?
//!
//! The GDELT Project monitors news media worldwide — print, broadcast, and web — in over
//! 100 languages. Every 15 minutes it publishes a new batch of structured records
//! representing news articles, each annotated with:
//!
//! - **People and organizations** mentioned
//! - **Themes** (e.g., `TAX_POLICY`, `TERROR`, `CLIMATE_CHANGE`)
//! - **Locations** with geocoordinates (country, state, city)
//! - **Tone/sentiment** analysis (positive, negative, polarity)
//! - **Quotations** extracted from the text
//! - **Event counts** (protests, arrests, killings, etc.)
//! - **GCAM** scores across dozens of content-analysis dimensions
//! - **Source URLs**, images, and translation metadata
//!
//! Each GKG record is a single tab-delimited line with 27 fields — this crate handles
//! parsing, filtering, full-text search, and multi-format output.
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                      newsfresh CLI                           │
//! │                                                              │
//! │  fetch ──► parse ──► filter ──► search ──► output            │
//! │    │         │          │          │          │               │
//! │  HTTP     27-field   10 filter   Tantivy   JSON / TeaLeaf   │
//! │  + ZIP    GKG v2.1   predicates  BM25 FTS  + field select   │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## CLI Commands
//!
//! | Command | Description |
//! |---------|-------------|
//! | `newsfresh fetch` | Download the latest (or historical) GKG data file |
//! | `newsfresh parse <file>` | Parse a local `.csv` or `.csv.zip` GKG file |
//! | `newsfresh query` | Fetch + parse + filter in one step |
//! | `newsfresh analyze --search "..."` | Natural-language full-text search over GKG data |
//! | `newsfresh schema` | Print GKG type definitions (TeaLeaf or JSON Schema) |
//!
//! ## Quick Start (CLI)
//!
//! ```bash
//! # Fetch the latest 15-minute GKG update
//! newsfresh fetch
//!
//! # Parse a local file, filter by country and person
//! newsfresh parse data/20250217150000.gkg.csv \
//!   --country US --person "Trump" --limit 10 -f json
//!
//! # One-shot: fetch latest + filter + output
//! newsfresh query --country US --theme "CLIMATE_CHANGE" --limit 5
//!
//! # Full-text search with natural language
//! newsfresh analyze --latest --search "elections Congress US economy" --limit 20
//!
//! # Print the GKG schema
//! newsfresh schema -f tealeaf
//! ```
//!
//! ## Library Usage
//!
//! The crate can also be used as a library. The main entry points are:
//!
//! ```rust,no_run
//! use newsfresh::parse::{self, GkgReader};
//! use newsfresh::filter::{CompositeFilter, RecordFilter};
//! use newsfresh::filter::predicates::*;
//! use newsfresh::search;
//!
//! // Parse records from a file
//! let file = std::fs::File::open("data/gkg.csv").unwrap();
//! let reader = std::io::BufReader::new(file);
//! let gkg_reader = GkgReader::new(reader);
//!
//! let mut records = Vec::new();
//! for result in gkg_reader {
//!     let (line_num, line) = result.unwrap();
//!     if let Ok(record) = parse::parse_record(&line, line_num) {
//!         records.push(record);
//!     }
//! }
//!
//! // Apply filters
//! let mut filters = CompositeFilter::new();
//! filters.add(Box::new(CountryFilter { code: "US".to_string() }));
//! filters.add(Box::new(PersonFilter { pattern: "Trump".to_string() }));
//!
//! let filtered: Vec<_> = records.iter()
//!     .filter(|r| filters.matches(r))
//!     .collect();
//!
//! // Full-text search
//! let mut engine = search::create_engine();
//! engine.build(&records).unwrap();
//! let hits = engine.search("climate policy carbon", 20).unwrap();
//! // hits[i].record_index, hits[i].score
//! ```
//!
//! ## Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`cli`] | CLI argument definitions (clap derive) — structs for all 5 subcommands |
//! | [`error`] | [`NewsfreshError`](error::NewsfreshError) enum covering HTTP, I/O, parse, ZIP, and JSON errors |
//! | [`fetch`] | HTTP client for downloading GKG data, ZIP extraction, lastupdate.txt parsing |
//! | [`filter`] | [`RecordFilter`](filter::RecordFilter) trait and 10 composable predicate filters |
//! | [`model`] | Complete GKG v2.1 data model — [`GkgRecord`](model::GkgRecord) with 27 fields and 14 sub-types |
//! | [`output`] | [`OutputFormatter`](output::OutputFormatter) trait with JSON and TeaLeaf formatters, field projection, schema printing |
//! | [`parse`] | Streaming GKG parser — [`GkgReader`](parse::GkgReader) iterator and tab-delimited field parsing |
//! | [`search`] | Full-text search via Tantivy — [`SearchEngine`](search::SearchEngine) trait with BM25 ranking, FIPS/ADM1/theme enrichment |
//!
//! ## Output Formats
//!
//! - **`json`** — Pretty-printed JSON array of records
//! - **`json-compact`** — Minified single-line JSON
//! - **`tealeaf`** — [TeaLeaf](https://github.com/krishjag/tealeaf) schema-driven format (~47% fewer tokens than JSON)
//! - **`tealeaf-compact`** — Minified TeaLeaf (additional ~21% savings)
//!
//! The `--fields` flag enables field projection — output only the fields you need:
//!
//! ```bash
//! newsfresh query --limit 5 -f json --fields document_identifier,source_common_name,tone
//! ```
//!
//! ## Filter Options
//!
//! All filters compose with AND logic. Available filters:
//!
//! | Flag | Description | Example |
//! |------|-------------|---------|
//! | `--person` | Person name (case-insensitive substring) | `--person "Trump"` |
//! | `--org` | Organization name | `--org "United Nations"` |
//! | `--theme` | GKG theme code | `--theme "TAX_POLICY"` |
//! | `--location` | Location name | `--location "Washington"` |
//! | `--country` | FIPS country code | `--country US` |
//! | `--tone-min` / `--tone-max` | Tone score range | `--tone-min -5 --tone-max 5` |
//! | `--date-from` / `--date-to` | Date range (YYYYMMDD) | `--date-from 20250201` |
//! | `--source` | Source name | `--source "bbc"` |
//! | `--has-image` | Only records with a sharing image | `--has-image` |
//! | `--has-quote` | Only records with quotations | `--has-quote` |
//!
//! ## Search Enrichment
//!
//! The `analyze` command builds an in-memory Tantivy full-text index with several
//! enrichment layers for better recall:
//!
//! - **FIPS country codes** → expanded to full country names (240+ countries)
//!   - e.g., a record with country code `US` becomes searchable by "United States"
//! - **ADM1 state/province codes** → expanded to readable names
//!   - e.g., `US06` → "California", `US36` → "New York"
//! - **Theme canonicalization** → taxonomy prefixes stripped, underscores → spaces
//!   - e.g., `TAX_FNCACT_PRESIDENT` → "PRESIDENT"
//!
//! This means a search for "California President" will match records that only contain
//! the raw codes `US06` and `TAX_FNCACT_PRESIDENT`.
//!
//! ## GKG v2.1 Record Fields
//!
//! The [`GkgRecord`](model::GkgRecord) struct maps all 27 tab-delimited fields:
//!
//! | # | Field | Type | Description |
//! |---|-------|------|-------------|
//! | 0 | `gkg_record_id` | `String` | Unique record identifier |
//! | 1 | `date` | `i64` | Publication date (YYYYMMDDHHMMSS) |
//! | 2 | `source_collection_id` | `i32` | Source type (1=Web, 2=Citation, 3=Core, ...) |
//! | 3 | `source_common_name` | `String` | Human-readable source name |
//! | 4 | `document_identifier` | `String` | Article URL |
//! | 5 | `v1_counts` | `Vec<CountV1>` | Event counts (protests, arrests, etc.) |
//! | 6 | `v21_counts` | `Vec<CountV21>` | V2.1 counts with character offsets |
//! | 7 | `v1_themes` | `Vec<String>` | Theme codes (e.g., `TAX_POLICY`) |
//! | 8 | `v2_enhanced_themes` | `Vec<EnhancedTheme>` | Themes with character offsets |
//! | 9 | `v1_locations` | `Vec<LocationV1>` | Geocoded locations |
//! | 10 | `v2_enhanced_locations` | `Vec<EnhancedLocation>` | V2 locations with ADM2 codes |
//! | 11 | `v1_persons` | `Vec<String>` | Person names |
//! | 12 | `v2_enhanced_persons` | `Vec<EnhancedEntity>` | Persons with character offsets |
//! | 13 | `v1_organizations` | `Vec<String>` | Organization names |
//! | 14 | `v2_enhanced_organizations` | `Vec<EnhancedEntity>` | Organizations with offsets |
//! | 15 | `tone` | `Option<Tone>` | Sentiment analysis (tone, polarity, word count) |
//! | 16 | `v21_enhanced_dates` | `Vec<EnhancedDate>` | Dates mentioned in the article |
//! | 17 | `gcam` | `Vec<GcamEntry>` | GCAM content-analysis scores |
//! | 18 | `sharing_image` | `Option<String>` | Primary sharing image URL |
//! | 19 | `related_images` | `Vec<String>` | Related image URLs |
//! | 20 | `social_image_embeds` | `Vec<String>` | Social media image embeds |
//! | 21 | `social_video_embeds` | `Vec<String>` | Social media video embeds |
//! | 22 | `quotations` | `Vec<Quotation>` | Direct quotes with attribution verbs |
//! | 23 | `all_names` | `Vec<NameEntry>` | All named entities |
//! | 24 | `amounts` | `Vec<AmountEntry>` | Monetary/numerical amounts |
//! | 25 | `translation_info` | `Option<TranslationInfo>` | Translation source language and engine |
//! | 26 | `extras_xml` | `Option<String>` | Extra XML content |
//!
//! ## Data Source
//!
//! GDELT data is freely available at <http://data.gdeltproject.org/gdeltv2/>.
//! The `lastupdate.txt` file lists the latest 15-minute update files.
//! Historical files are available by date in `YYYYMMDDHHMMSS` format.

pub mod cli;
pub mod error;
pub mod fetch;
pub mod filter;
pub mod model;
pub mod output;
pub mod parse;
pub mod search;
#[cfg(feature = "stats")]
pub mod stats;
