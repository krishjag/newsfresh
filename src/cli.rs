use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "newsfresh", about = "Query and analyze GDELT GKG v2.1 data")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Increase logging verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Download GKG data (latest or historical)
    Fetch(FetchArgs),

    /// Parse a local GKG file and output records
    Parse(ParseArgs),

    /// Fetch + parse + filter in one step
    Query(QueryArgs),

    /// Print GKG type definitions
    Schema(SchemaArgs),

    /// NL search + analyze GKG records
    Analyze(AnalyzeArgs),
}

#[derive(Args)]
pub struct FetchArgs {
    /// Fetch the latest 15-minute update (default behavior)
    #[arg(long, default_value_t = true)]
    pub latest: bool,

    /// Fetch a specific historical file by date (YYYYMMDDHHMMSS)
    #[arg(long, conflicts_with = "latest")]
    pub date: Option<String>,

    /// Fetch translation (non-English) variant
    #[arg(long)]
    pub translation: bool,

    /// Output directory
    #[arg(short, long, default_value = "./data")]
    pub output: PathBuf,

    /// Keep the .zip file after extraction
    #[arg(long)]
    pub keep_zip: bool,
}

#[derive(Args)]
pub struct ParseArgs {
    /// Path to a local .csv or .csv.zip GKG file
    pub file: PathBuf,

    #[command(flatten)]
    pub filter: FilterArgs,

    #[command(flatten)]
    pub output: OutputArgs,
}

#[derive(Args)]
pub struct QueryArgs {
    /// Fetch the latest 15-minute update
    #[arg(long)]
    pub latest: bool,

    /// Fetch a specific historical file by date (YYYYMMDDHHMMSS)
    #[arg(long)]
    pub date: Option<String>,

    /// Fetch translation (non-English) variant
    #[arg(long)]
    pub translation: bool,

    /// Persist downloaded data to persisted-storage/ directory
    #[arg(long)]
    pub persist_data_file: bool,

    #[command(flatten)]
    pub filter: FilterArgs,

    #[command(flatten)]
    pub output: OutputArgs,
}

#[derive(Args)]
pub struct FilterArgs {
    /// Filter by person name (case-insensitive substring)
    #[arg(long)]
    pub person: Option<String>,

    /// Filter by organization name
    #[arg(long)]
    pub org: Option<String>,

    /// Filter by theme (e.g. TAX_POLICY)
    #[arg(long)]
    pub theme: Option<String>,

    /// Filter by location name
    #[arg(long)]
    pub location: Option<String>,

    /// Filter by country FIPS code
    #[arg(long)]
    pub country: Option<String>,

    /// Minimum tone score
    #[arg(long)]
    pub tone_min: Option<f64>,

    /// Maximum tone score
    #[arg(long)]
    pub tone_max: Option<f64>,

    /// Records from this date onward (YYYYMMDD or YYYYMMDDHHMMSS)
    #[arg(long)]
    pub date_from: Option<String>,

    /// Records up to this date
    #[arg(long)]
    pub date_to: Option<String>,

    /// Filter by source name
    #[arg(long)]
    pub source: Option<String>,

    /// Only records with a sharing image
    #[arg(long)]
    pub has_image: bool,

    /// Only records with quotations
    #[arg(long)]
    pub has_quote: bool,
}

#[derive(Args)]
pub struct OutputArgs {
    /// Output format
    #[arg(short, long, default_value = "json", value_enum)]
    pub format: OutputFormat,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Maximum number of records to output
    #[arg(long)]
    pub limit: Option<usize>,

    /// Skip first N records
    #[arg(long)]
    pub offset: Option<usize>,

    /// Comma-separated list of field names to include
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    JsonCompact,
    Tealeaf,
    TealeafCompact,
}

#[derive(Args)]
pub struct SchemaArgs {
    /// Schema output format
    #[arg(short, long, default_value = "tealeaf", value_enum)]
    pub format: SchemaFormat,
}

#[derive(Args)]
pub struct AnalyzeArgs {
    /// Path to a local .csv or .csv.zip GKG file
    pub file: Option<PathBuf>,

    /// Natural language search query
    #[arg(long)]
    pub search: String,

    /// Fetch the latest 15-minute update
    #[arg(long)]
    pub latest: bool,

    /// Fetch a specific historical file by date (YYYYMMDDHHMMSS)
    #[arg(long)]
    pub date: Option<String>,

    /// Fetch translation (non-English) variant
    #[arg(long)]
    pub translation: bool,

    /// Persist downloaded data to persisted-storage/ directory
    #[arg(long)]
    pub persist_data_file: bool,

    /// Maximum number of results (default 20)
    #[arg(long, default_value_t = 20)]
    pub limit: usize,

    /// Show aggregate statistics instead of individual records
    #[arg(long)]
    pub stats: bool,

    /// Number of top entries per frequency table (default 10)
    #[arg(long, default_value_t = 10)]
    pub stats_top_n: usize,

    #[command(flatten)]
    pub filter: FilterArgs,

    #[command(flatten)]
    pub output: AnalyzeOutputArgs,
}

#[derive(Args)]
pub struct AnalyzeOutputArgs {
    /// Output format (defaults to tealeaf)
    #[arg(short, long, default_value = "tealeaf", value_enum)]
    pub format: OutputFormat,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Comma-separated list of field names to include
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

#[derive(Clone, ValueEnum)]
pub enum SchemaFormat {
    Tealeaf,
    JsonSchema,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_fetch_default() {
        let cli = Cli::try_parse_from(["newsfresh", "fetch"]).unwrap();
        assert!(matches!(cli.command, Command::Fetch(_)));
    }

    #[test]
    fn test_parse_fetch_with_date() {
        let cli = Cli::try_parse_from(["newsfresh", "fetch", "--date", "20250217120000"]).unwrap();
        if let Command::Fetch(args) = cli.command {
            assert_eq!(args.date, Some("20250217120000".to_string()));
        } else {
            panic!("Expected Fetch");
        }
    }

    #[test]
    fn test_parse_parse_command() {
        let cli = Cli::try_parse_from(["newsfresh", "parse", "data/test.csv"]).unwrap();
        assert!(matches!(cli.command, Command::Parse(_)));
    }

    #[test]
    fn test_parse_query_with_filters() {
        let cli = Cli::try_parse_from([
            "newsfresh",
            "query",
            "--country",
            "US",
            "--person",
            "Trump",
            "--limit",
            "10",
        ])
        .unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.filter.country, Some("US".to_string()));
            assert_eq!(args.filter.person, Some("Trump".to_string()));
            assert_eq!(args.output.limit, Some(10));
        } else {
            panic!("Expected Query");
        }
    }

    #[test]
    fn test_parse_analyze_command() {
        let cli = Cli::try_parse_from([
            "newsfresh",
            "analyze",
            "--search",
            "US politics",
            "--latest",
            "--stats",
        ])
        .unwrap();
        if let Command::Analyze(args) = cli.command {
            assert_eq!(args.search, "US politics");
            assert!(args.latest);
            assert!(args.stats);
        } else {
            panic!("Expected Analyze");
        }
    }

    #[test]
    fn test_parse_schema_command() {
        let cli = Cli::try_parse_from(["newsfresh", "schema", "-f", "json-schema"]).unwrap();
        assert!(matches!(cli.command, Command::Schema(_)));
    }

    #[test]
    fn test_global_verbose_flag() {
        let cli = Cli::try_parse_from(["newsfresh", "-vv", "fetch"]).unwrap();
        assert_eq!(cli.verbose, 2);
    }
}
