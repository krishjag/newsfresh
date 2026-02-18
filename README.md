# NewsFresh

[![Rust CI](https://github.com/krishjag/newsfresh/actions/workflows/ci.yml/badge.svg)](https://github.com/krishjag/newsfresh/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/newsfresh.svg)](https://crates.io/crates/newsfresh)
[![codecov](https://codecov.io/gh/krishjag/newsfresh/graph/badge.svg)](https://codecov.io/gh/krishjag/newsfresh)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast CLI tool for querying, filtering, and analyzing [GDELT Global Knowledge Graph (GKG) v2.1](https://blog.gdeltproject.org/gdelt-2-0-our-global-world-in-realtime/) data — the world's largest open dataset of global news events, updated every 15 minutes. Includes an LLM-powered agent that turns GDELT into a personalized agentic news feed, delivering real-time email alerts tailored to your interests.

## What is GDELT?

The [GDELT Project](https://www.gdeltproject.org/) monitors news media worldwide — print, broadcast, and web — in over 100 languages. Every 15 minutes it publishes structured records representing news articles, each annotated with people, organizations, themes, locations, sentiment, quotations, event counts, and more.

Each GKG record is a tab-delimited line with 27 fields. **newsfresh** handles the full pipeline: downloading, decompressing, parsing, filtering, searching, and outputting results in JSON or [TeaLeaf](https://github.com/krishjag/tealeaf) format.

## Features

### Personalized Agentic News Feed

An LLM-powered monitoring agent runs 24/7, polling GDELT every 15 minutes and delivering email alerts for topics you care about. Define interest profiles in a simple JSON config and the agent handles the rest — search, deduplication, summarization, and notification.

- **Multi-provider LLM support** — swap between Claude (Haiku, Sonnet) and OpenAI (GPT-4o, GPT-4o-mini) with a single config change
- **Interest profiles** — define multiple search topics with country, person, theme, and tone filters
- **Smart deduplication** — 24-hour rolling window prevents duplicate alerts
- **Email notifications** — HTML summaries with who/what/where/sentiment via SendGrid
- **Live config reload** — change profiles or switch LLM providers without restarting

See [`agent/README.md`](agent/README.md) for setup and usage.

### Full-Text Search & Analysis

- **Natural language search** over GKG records using an in-memory [Tantivy](https://github.com/quickwit-oss/tantivy) index with BM25 ranking
- **Search enrichment** — FIPS country codes, ADM1 state/province codes, and theme taxonomy prefixes are expanded so searching "United States" or "California" matches raw codes like `US` or `US06`
- **Aggregate statistics** — frequency tables and tone analysis over search results using [Polars](https://pola.rs/) DataFrames (`--stats`)

### Flexible Data Pipeline

- **Fetch, parse, filter, search, output** — each stage works independently or composed via `query` and `analyze` commands
- **12 composable filters** — person, organization, theme, location, country, tone range, date range, source, image, and quotation filters, all AND-composed
- **4 output formats** — JSON, JSON-compact, TeaLeaf (~47% fewer tokens), TeaLeaf-compact (~60% fewer tokens)
- **Field projection** — output only the fields you need with `--fields`
- **Streaming parser** — handles large GKG files without loading everything into memory

## Installation

### Pre-built binaries

Download the latest release from [GitHub Releases](https://github.com/krishjag/newsfresh/releases):

**Linux / macOS:**

```bash
# Linux (x64)
curl -L https://github.com/krishjag/newsfresh/releases/latest/download/newsfresh-linux-x64.tar.gz | tar xz
sudo mv newsfresh /usr/local/bin/

# Linux (arm64)
curl -L https://github.com/krishjag/newsfresh/releases/latest/download/newsfresh-linux-arm64.tar.gz | tar xz
sudo mv newsfresh /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/krishjag/newsfresh/releases/latest/download/newsfresh-macos-arm64.tar.gz | tar xz
sudo mv newsfresh /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/krishjag/newsfresh/releases/latest/download/newsfresh-macos-x64.tar.gz | tar xz
sudo mv newsfresh /usr/local/bin/
```

**Windows (PowerShell):**

```powershell
# Windows (x64)
Invoke-WebRequest -Uri https://github.com/krishjag/newsfresh/releases/latest/download/newsfresh-windows-x64.zip -OutFile newsfresh.zip
Expand-Archive newsfresh.zip -DestinationPath .
Move-Item newsfresh.exe "$env:USERPROFILE\.cargo\bin\"   # or any directory in your PATH
Remove-Item newsfresh.zip

# Windows (arm64)
Invoke-WebRequest -Uri https://github.com/krishjag/newsfresh/releases/latest/download/newsfresh-windows-arm64.zip -OutFile newsfresh.zip
Expand-Archive newsfresh.zip -DestinationPath .
Move-Item newsfresh.exe "$env:USERPROFILE\.cargo\bin\"   # or any directory in your PATH
Remove-Item newsfresh.zip
```

### From crates.io

```bash
cargo install newsfresh
```

### From source

```bash
git clone https://github.com/krishjag/newsfresh
cd newsfresh
cargo build --release
# Binary at target/release/newsfresh
```

### Agent setup

The monitoring agent requires the `newsfresh` CLI and Python 3.10+:

```bash
# Install the CLI (any method above), then:
cd agent
pip install -r requirements.txt
cp config.json.example config.json  # edit with your profiles
python main.py --once               # test a single cycle
```

See [`agent/README.md`](agent/README.md) for full setup instructions.

<!-- COMMANDS:START -->
## Commands

### `fetch` — Download GKG data

```bash
# Latest 15-minute update (default)
newsfresh fetch

# Historical file by date
newsfresh fetch --date 20250217150000

# Non-English variant, custom output directory, keep zip
newsfresh fetch --translation -o ./my-data --keep-zip

```

### `parse` — Parse a local GKG file

```bash
# Parse a CSV file with filters
newsfresh parse data/20250217150000.gkg.csv \
  --country US --person "Trump" --limit 10

# Parse directly from a zip file
newsfresh parse data/20250217150000.gkg.csv.zip -f json

# Output specific fields only
newsfresh parse data/gkg.csv \
  -f json --fields document_identifier,source_common_name,tone

```

### `query` — Fetch + parse + filter in one step

```bash
# Fetch latest and filter by theme
newsfresh query --country US --theme "CLIMATE_CHANGE" --limit 5

# Historical data with tone filter
newsfresh query --date 20250201120000 --tone-min=-10 --tone-max=-2

# Persist downloaded files for reuse
newsfresh query --persist-data-file --country US --limit 20

# Output in compact TeaLeaf format
newsfresh query --country UK --has-quote -f tealeaf-compact

```

### `analyze` — Natural language full-text search

```bash
# Search the latest GDELT data with natural language
newsfresh analyze --latest \
  --search "elections Congress US economy" --limit 20

# Search with additional structured filters
newsfresh analyze --latest \
  --search "climate carbon emissions policy" \
  --country US --limit 10

# From a local file
newsfresh analyze data/gkg.csv \
  --search "Ukraine Russia ceasefire negotiations" --limit 15

# Compact TeaLeaf output for LLM consumption (~47% fewer tokens)
newsfresh analyze --latest \
  --search "AI regulation technology" --limit 10 -f tealeaf-compact

# Aggregate statistics with Polars DataFrames
newsfresh analyze --latest \
  --search "US politics" --limit 50 --stats

# Top 5 per category
newsfresh analyze data/gkg.csv \
  --search "climate change" --stats --stats-top-n 5 --limit 100

```

The `analyze` command builds an in-memory [Tantivy](https://github.com/quickwit-oss/tantivy) full-text index with BM25 ranking. Search enrichment layers expand raw codes into readable text for better recall:

- **FIPS country codes** expanded to full names (240+ countries) — searching "United States" matches records with code `US`
- **ADM1 state/province codes** expanded — searching "California" matches code `US06`
- **Theme codes** canonicalized — `TAX_FNCACT_PRESIDENT` becomes searchable as "President"

#### `--stats` — Aggregate statistics

When `--stats` is passed, instead of outputting individual records, `analyze` computes frequency tables and statistics over the matched results using [Polars](https://pola.rs/) DataFrames:

```
=== GDELT Analysis Stats (50 records) ===

--- Top Themes ---
   1. LEADER                        24  (2.0%)
   2. GENERAL GOVERNMENT            24  (2.0%)
   3. GOVERNMENT                    21  (1.8%)

--- Top Countries ---
   1. United States (US)            48  (31.2%)
   2. United Kingdom (UK)            9  (5.8%)

--- Tone ---
  Mean: -0.82  Std: 3.45  Range: [-8.12, 4.91]
  Most positive: [4.91] https://example.com/positive-article
  Most negative: [-8.12] https://example.com/negative-article

--- Top Persons ---
   1. donald trump                    9  (4.9%)
   2. elon musk                       3  (1.6%)

--- Top Organizations ---
   1. congress                       3  (1.7%)

--- Top Sources ---
   1. nytimes.com                   16  (32.0%)
```

| Flag | Description | Default |
|------|-------------|---------|
| `--stats` | Show aggregate statistics instead of records | off |
| `--stats-top-n` | Number of entries per frequency table | 10 |

### `schema` — Print GKG type definitions

```bash
# TeaLeaf schema format
newsfresh schema

# JSON Schema format
newsfresh schema -f json-schema

```

## Filter Options

All filters compose with AND logic. Available on `parse`, `query`, and `analyze`:

| Flag | Description | Example |
|------|-------------|---------|
| `--person` | Person name (case-insensitive substring) | `--person "Trump"` |
| `--org` | Organization name | `--org "United Nations"` |
| `--theme` | GKG theme code | `--theme "TAX_POLICY"` |
| `--location` | Location name | `--location "Washington"` |
| `--country` | FIPS country code | `--country US` |
| `--tone-min / --tone-max` | Tone score range | `--tone-min -5 --tone-max 5` |
| `--date-from / --date-to` | Date range (YYYYMMDD) | `--date-from 20250201` |
| `--source` | Source name | `--source "bbc"` |
| `--has-image` | Only records with a sharing image | `--has-image` |
| `--has-quote` | Only records with quotations | `--has-quote` |

## Output Formats

| Format | Flag | Description |
|--------|------|-------------|
| JSON (pretty) | `-f json` | Pretty-printed JSON array (default) |
| JSON (compact) | `-f json-compact` | Minified single-line JSON |
| TeaLeaf | `-f tealeaf` | Schema-driven format, ~47% fewer tokens than JSON |
| TeaLeaf (compact) | `-f tealeaf-compact` | Minified TeaLeaf, additional ~21% savings |

Use `--fields` for field projection — output only the fields you need:

```bash
newsfresh query --limit 5 -f json --fields document_identifier,source_common_name,tone
```

<!-- COMMANDS:END -->

## GKG v2.1 Record Fields

Each record contains up to 27 fields:

| Field | Type | Description |
|-------|------|-------------|
| `gkg_record_id` | String | Unique record identifier |
| `date` | Integer | Publication date (YYYYMMDDHHMMSS) |
| `source_collection_id` | Integer | Source type (1=Web, 2=Citation, 3=Core, ...) |
| `source_common_name` | String | Human-readable source name |
| `document_identifier` | String | Article URL |
| `v1_counts` | Array | Event counts (protests, arrests, etc.) |
| `v21_counts` | Array | V2.1 counts with character offsets |
| `v1_themes` | Array | Theme codes (e.g., TAX_POLICY) |
| `v2_enhanced_themes` | Array | Themes with character offsets |
| `v1_locations` | Array | Geocoded locations (country, lat/lon) |
| `v2_enhanced_locations` | Array | V2 locations with ADM2 codes |
| `v1_persons` | Array | Person names mentioned |
| `v2_enhanced_persons` | Array | Persons with character offsets |
| `v1_organizations` | Array | Organization names |
| `v2_enhanced_organizations` | Array | Organizations with offsets |
| `tone` | Object | Sentiment (tone, polarity, pos/neg scores, word count) |
| `v21_enhanced_dates` | Array | Dates mentioned in the article |
| `gcam` | Array | GCAM content-analysis dimension scores |
| `sharing_image` | String? | Primary sharing image URL |
| `related_images` | Array | Related image URLs |
| `social_image_embeds` | Array | Social media image embeds |
| `social_video_embeds` | Array | Social media video embeds |
| `quotations` | Array | Direct quotes with attribution verbs |
| `all_names` | Array | All named entities |
| `amounts` | Array | Monetary/numerical amounts |
| `translation_info` | Object? | Translation source language and engine |
| `extras_xml` | String? | Extra XML content |

## Library Usage

**newsfresh** can also be used as a Rust library:

```rust
use newsfresh::parse::{self, GkgReader};
use newsfresh::filter::{CompositeFilter, RecordFilter};
use newsfresh::filter::predicates::*;
use newsfresh::search;

// Parse records from a file
let file = std::fs::File::open("data/gkg.csv")?;
let reader = std::io::BufReader::new(file);
let gkg_reader = GkgReader::new(reader);

let mut records = Vec::new();
for result in gkg_reader {
    let (line_num, line) = result?;
    if let Ok(record) = parse::parse_record(&line, line_num) {
        records.push(record);
    }
}

// Apply filters
let mut filters = CompositeFilter::new();
filters.add(Box::new(CountryFilter { code: "US".to_string() }));
filters.add(Box::new(PersonFilter { pattern: "Trump".to_string() }));

let filtered: Vec<_> = records.iter()
    .filter(|r| filters.matches(r))
    .collect();

// Full-text search with BM25 ranking
let mut engine = search::create_engine();
engine.build(&records)?;
let hits = engine.search("climate policy carbon", 20)?;

for hit in &hits {
    let record = &records[hit.record_index];
    println!("[{:.2}] {}", hit.score, record.document_identifier);
}
```

## Module Overview

| Module | Description |
|--------|-------------|
| `cli` | CLI argument definitions (clap derive) |
| `error` | `NewsfreshError` — HTTP, I/O, parse, ZIP, and JSON errors |
| `fetch` | HTTP client, ZIP extraction, lastupdate.txt parsing |
| `filter` | `RecordFilter` trait and 10 composable predicate filters |
| `model` | Complete GKG v2.1 data model — `GkgRecord` with 27 fields, 14 sub-types |
| `output` | `OutputFormatter` trait — JSON and TeaLeaf formatters, field projection, schema printing |
| `parse` | Streaming GKG parser — `GkgReader` iterator, tab-delimited field parsing |
| `search` | Tantivy full-text search — `SearchEngine` trait, BM25, FIPS/ADM1/theme enrichment |
| `stats` | Polars DataFrame aggregation — theme/country/person/org/source frequency, tone statistics |

## Data Source

GDELT data is freely available at <http://data.gdeltproject.org/gdeltv2/>. The project publishes new data every 15 minutes. See the [GDELT documentation](https://blog.gdeltproject.org/gdelt-2-0-our-global-world-in-realtime/) for details on the GKG v2.1 format.

## License

MIT
