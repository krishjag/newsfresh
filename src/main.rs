use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use tracing::{debug, warn};

use newsfresh::cli::*;
use newsfresh::error::NewsfreshError;
use newsfresh::fetch::{client, decompress, lastupdate};
use newsfresh::filter::predicates::*;
use newsfresh::filter::{CompositeFilter, RecordFilter};
use newsfresh::model::ScoredRecord;
use newsfresh::output::{self, OutputFormatter};
use newsfresh::parse;
use newsfresh::search;

#[tokio::main]
async fn main() -> Result<(), NewsfreshError> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        Command::Fetch(args) => cmd_fetch(args).await,
        Command::Parse(args) => cmd_parse(args),
        Command::Query(args) => cmd_query(args).await,
        Command::Schema(args) => cmd_schema(args),
        Command::Analyze(args) => cmd_analyze(args).await,
    }
}

async fn cmd_fetch(args: FetchArgs) -> Result<(), NewsfreshError> {
    std::fs::create_dir_all(&args.output)?;

    let url = if let Some(ref date) = args.date {
        client::historical_url(date)
    } else {
        let text = client::fetch_text(client::lastupdate_url(args.translation)).await?;
        let entries = lastupdate::parse_lastupdate(&text);
        lastupdate::find_gkg_url(&entries)?
    };

    let filename = url.rsplit('/').next().unwrap_or("gkg.csv.zip");
    let zip_path = args.output.join(filename);

    eprintln!("Fetching: {url}");
    client::download_file(&url, &zip_path).await?;

    let csv_path = decompress::extract_gkg_from_zip(&zip_path, &args.output)?;
    eprintln!("Extracted: {}", csv_path.display());

    if !args.keep_zip {
        std::fs::remove_file(&zip_path)?;
    }

    Ok(())
}

fn cmd_parse(args: ParseArgs) -> Result<(), NewsfreshError> {
    let filters = build_filters(&args.filter);
    let writer = make_writer(&args.output.output)?;
    let format_str = format_to_str(&args.output.format);
    let mut formatter = output::create_formatter(&format_str, writer, &args.output.fields);

    let reader = open_gkg_file(&args.file)?;
    run_pipeline(
        reader,
        &filters,
        &mut *formatter,
        args.output.offset,
        args.output.limit,
    )
}

async fn cmd_query(args: QueryArgs) -> Result<(), NewsfreshError> {
    let data_dir = resolve_data_dir(args.persist_data_file)?;

    let url = if let Some(ref date) = args.date {
        client::historical_url(date)
    } else {
        let text = client::fetch_text(client::lastupdate_url(args.translation)).await?;
        let entries = lastupdate::parse_lastupdate(&text);
        lastupdate::find_gkg_url(&entries)?
    };

    let filename = url.rsplit('/').next().unwrap_or("gkg.csv.zip");
    let zip_path = data_dir.path().join(filename);

    eprintln!("Fetching: {url}");
    client::download_file(&url, &zip_path).await?;

    let csv_path = decompress::extract_gkg_from_zip(&zip_path, data_dir.path())?;
    eprintln!("Parsing: {}", csv_path.display());

    let filters = build_filters(&args.filter);
    let writer = make_writer(&args.output.output)?;
    let format_str = format_to_str(&args.output.format);
    let mut formatter = output::create_formatter(&format_str, writer, &args.output.fields);

    let reader = open_gkg_file(&csv_path)?;
    run_pipeline(
        reader,
        &filters,
        &mut *formatter,
        args.output.offset,
        args.output.limit,
    )
}

fn cmd_schema(args: SchemaArgs) -> Result<(), NewsfreshError> {
    let mut stdout = std::io::stdout();
    match args.format {
        SchemaFormat::Tealeaf => output::schema::print_tealeaf_schema(&mut stdout)?,
        SchemaFormat::JsonSchema => output::schema::print_json_schema(&mut stdout)?,
    }
    Ok(())
}

async fn cmd_analyze(args: AnalyzeArgs) -> Result<(), NewsfreshError> {
    // Phase 0: Resolve data source
    let reader: Box<dyn BufRead> = if let Some(ref file) = args.file {
        open_gkg_file(file)?
    } else {
        let data_dir = resolve_data_dir(args.persist_data_file)?;
        let url = if let Some(ref date) = args.date {
            client::historical_url(date)
        } else {
            let text = client::fetch_text(client::lastupdate_url(args.translation)).await?;
            let entries = lastupdate::parse_lastupdate(&text);
            lastupdate::find_gkg_url(&entries)?
        };
        let filename = url.rsplit('/').next().unwrap_or("gkg.csv.zip");
        let zip_path = data_dir.path().join(filename);
        eprintln!("Fetching: {url}");
        client::download_file(&url, &zip_path).await?;
        let csv_path = decompress::extract_gkg_from_zip(&zip_path, data_dir.path())?;
        eprintln!("Parsing: {}", csv_path.display());
        // Read entire file into memory since data_dir (if temp) will be dropped
        let content = std::fs::read_to_string(&csv_path)?;
        Box::new(BufReader::new(std::io::Cursor::new(content)))
    };

    // Phase 1: Parse all records
    let gkg_reader = parse::GkgReader::new(reader);
    let mut records = Vec::new();
    let mut errors: usize = 0;

    for result in gkg_reader {
        let (line_num, line) = result?;
        match parse::parse_record(&line, line_num) {
            Ok(record) => records.push(record),
            Err(e) => {
                warn!("Skipping line {line_num}: {e}");
                errors += 1;
            }
        }
    }
    eprintln!("Parsed {} records ({errors} errors skipped)", records.len());

    // Phase 2: Build index and search (over-fetch 3x for filter headroom)
    let mut engine = search::create_engine();
    engine.build(&records)?;
    let fetch_limit = args.limit * 3;
    let hits = engine.search(&args.search, fetch_limit)?;
    eprintln!("Search returned {} candidates", hits.len());

    // Phase 3: Apply structured filters and collect top-N
    let filters = build_filters(&args.filter);

    if args.stats {
        let mut filtered = Vec::new();
        for hit in &hits {
            let record = &records[hit.record_index];
            if !filters.matches(record) {
                continue;
            }
            filtered.push((hit.score, record));
            if filtered.len() >= args.limit {
                break;
            }
        }
        let stats = newsfresh::stats::compute_stats(&filtered, args.stats_top_n)?;
        let mut stdout = std::io::stdout();
        newsfresh::stats::print_stats(&stats, &mut stdout)?;
        eprintln!(
            "Stats computed over {} records ({errors} parse errors)",
            filtered.len()
        );
    } else {
        let writer = make_writer(&args.output.output)?;
        let format_str = format_to_str(&args.output.format);
        let mut formatter = output::create_formatter(&format_str, writer, &args.output.fields);

        formatter.begin()?;

        let mut count: usize = 0;
        for hit in &hits {
            let record = &records[hit.record_index];
            if !filters.matches(record) {
                continue;
            }
            let scored = ScoredRecord {
                relevance_score: hit.score,
                record: record.clone(),
            };
            formatter.write_scored_record(&scored)?;
            count += 1;
            if count >= args.limit {
                break;
            }
        }

        formatter.finish()?;
        eprintln!(
            "Output {count} results (top {}, {errors} parse errors)",
            args.limit
        );
    }
    Ok(())
}

/// Holds the download directory — either a temp dir (auto-cleaned) or a persisted path.
enum DataDir {
    Temp(tempfile::TempDir),
    Persisted(PathBuf),
}

impl DataDir {
    fn path(&self) -> &Path {
        match self {
            DataDir::Temp(t) => t.path(),
            DataDir::Persisted(p) => p,
        }
    }
}

const PERSISTED_STORAGE_DIR: &str = "persisted-storage";

/// Resolve the download directory. If `persist` is true, try to create/use
/// `persisted-storage/`. On any failure, silently fall back to a temp dir.
fn resolve_data_dir(persist: bool) -> Result<DataDir, NewsfreshError> {
    if persist {
        let dir = PathBuf::from(PERSISTED_STORAGE_DIR);
        match std::fs::create_dir_all(&dir) {
            Ok(()) => {
                debug!("Using persisted storage: {}", dir.display());
                return Ok(DataDir::Persisted(dir));
            }
            Err(e) => {
                eprintln!(
                    "Warning: could not create {PERSISTED_STORAGE_DIR}/: {e}, using temp dir"
                );
            }
        }
    }
    Ok(DataDir::Temp(tempfile::tempdir()?))
}

fn build_filters(args: &FilterArgs) -> CompositeFilter {
    let mut composite = CompositeFilter::new();

    if let Some(ref person) = args.person {
        composite.add(Box::new(PersonFilter {
            pattern: person.clone(),
        }));
    }
    if let Some(ref org) = args.org {
        composite.add(Box::new(OrgFilter {
            pattern: org.clone(),
        }));
    }
    if let Some(ref theme) = args.theme {
        composite.add(Box::new(ThemeFilter {
            pattern: theme.clone(),
        }));
    }
    if let Some(ref location) = args.location {
        composite.add(Box::new(LocationFilter {
            pattern: location.clone(),
        }));
    }
    if let Some(ref country) = args.country {
        composite.add(Box::new(CountryFilter {
            code: country.clone(),
        }));
    }
    if args.tone_min.is_some() || args.tone_max.is_some() {
        composite.add(Box::new(ToneRangeFilter {
            min: args.tone_min,
            max: args.tone_max,
        }));
    }
    if args.date_from.is_some() || args.date_to.is_some() {
        composite.add(Box::new(DateRangeFilter {
            from: args.date_from.as_ref().and_then(|d| d.parse().ok()),
            to: args.date_to.as_ref().and_then(|d| d.parse().ok()),
        }));
    }
    if let Some(ref source) = args.source {
        composite.add(Box::new(SourceFilter {
            pattern: source.clone(),
        }));
    }
    if args.has_image {
        composite.add(Box::new(HasImageFilter));
    }
    if args.has_quote {
        composite.add(Box::new(HasQuoteFilter));
    }

    composite
}

fn open_gkg_file(path: &Path) -> Result<Box<dyn BufRead>, NewsfreshError> {
    if path.extension().and_then(|e| e.to_str()) == Some("zip") {
        let content = decompress::read_gkg_from_zip(path)?;
        Ok(Box::new(BufReader::new(std::io::Cursor::new(content))))
    } else {
        let file = std::fs::File::open(path)?;
        Ok(Box::new(BufReader::new(file)))
    }
}

fn run_pipeline(
    reader: Box<dyn BufRead>,
    filters: &CompositeFilter,
    formatter: &mut dyn OutputFormatter,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<(), NewsfreshError> {
    let gkg_reader = parse::GkgReader::new(reader);

    formatter.begin()?;

    let mut count: usize = 0;
    let mut skipped: usize = 0;
    let mut errors: usize = 0;
    let skip_n = offset.unwrap_or(0);

    for result in gkg_reader {
        let (line_num, line) = result?;
        match parse::parse_record(&line, line_num) {
            Ok(record) => {
                if !filters.matches(&record) {
                    continue;
                }
                if skipped < skip_n {
                    skipped += 1;
                    continue;
                }
                formatter.write_record(&record)?;
                count += 1;
                if let Some(lim) = limit
                    && count >= lim
                {
                    break;
                }
            }
            Err(e) => {
                warn!("Skipping line {line_num}: {e}");
                errors += 1;
            }
        }
    }

    formatter.finish()?;

    eprintln!("Output {count} records ({errors} parse errors skipped)");
    Ok(())
}

fn make_writer(output_path: &Option<std::path::PathBuf>) -> Result<Box<dyn Write>, NewsfreshError> {
    match output_path {
        Some(path) => {
            let file = std::fs::File::create(path)?;
            Ok(Box::new(std::io::BufWriter::new(file)))
        }
        None => Ok(Box::new(std::io::BufWriter::new(std::io::stdout()))),
    }
}

fn format_to_str(format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => "json".to_string(),
        OutputFormat::JsonCompact => "json-compact".to_string(),
        OutputFormat::Tealeaf => "tealeaf".to_string(),
        OutputFormat::TealeafCompact => "tealeaf-compact".to_string(),
    }
}
