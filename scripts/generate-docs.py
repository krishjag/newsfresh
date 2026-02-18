#!/usr/bin/env python3
"""
Unified docs generator for newsfresh.

Generates the shared command reference sections for both README.md (crates.io/GitHub)
and summary.html (GitHub Pages) from a single data source, keeping them permanently in sync.

Usage:
    python scripts/generate-docs.py
"""

from pathlib import Path
import html as html_mod
import textwrap

# ============================================================================
# Single source of truth — command data
# ============================================================================

COMMANDS = [
    {
        "name": "fetch",
        "title_md": "`fetch` — Download GKG data",
        "title_html": '<code>fetch</code> &mdash; Download GKG Data',
        "desc": "Downloads a GKG data file (latest 15-minute update or historical) from GDELT servers. Automatically extracts the CSV from the ZIP archive.",
        "section_class": "section-fetch",
        "args": [
            {"flag": "--latest", "type": "bool", "desc": "Fetch the latest 15-minute update", "default": "true"},
            {"flag": "--date <DATE>", "type": "string", "desc": "Fetch a specific historical file (YYYYMMDDHHMMSS)"},
            {"flag": "--translation", "type": "bool", "desc": "Fetch non-English (translation) variant", "default": "false"},
            {"flag": "-o, --output <DIR>", "type": "path", "desc": "Output directory", "default": "./data"},
            {"flag": "--keep-zip", "type": "bool", "desc": "Keep the .zip file after extraction", "default": "false"},
        ],
        "examples": [
            {"comment": "Latest 15-minute update (default)", "cmd": "newsfresh fetch"},
            {"comment": "Historical file by date", "cmd": "newsfresh fetch --date 20250217150000"},
            {"comment": "Non-English variant, custom output directory, keep zip", "cmd": "newsfresh fetch --translation -o ./my-data --keep-zip"},
        ],
        "sample_output": "Fetching: http://data.gdeltproject.org/gdeltv2/20260216054500.gkg.csv.zip\nExtracted: data/20260216054500.gkg.csv",
    },
    {
        "name": "parse",
        "title_md": "`parse` — Parse a local GKG file",
        "title_html": '<code>parse</code> &mdash; Parse a Local GKG File',
        "desc": "Parses a local .csv or .csv.zip GKG file. Supports all filters, output formats, field projection, and offset/limit pagination.",
        "section_class": "section-parse",
        "args": [
            {"flag": "<FILE>", "type": "path", "desc": "Path to a local .csv or .csv.zip GKG file", "required": True},
            {"flag": "-f, --format", "type": "enum", "desc": "Output format: json, json-compact, tealeaf, tealeaf-compact", "default": "json"},
            {"flag": "-o, --output", "type": "path", "desc": "Output file (stdout if omitted)", "default": "stdout"},
            {"flag": "--limit <N>", "type": "int", "desc": "Maximum number of records to output", "default": "all"},
            {"flag": "--offset <N>", "type": "int", "desc": "Skip first N records", "default": "0"},
            {"flag": "--fields <LIST>", "type": "string", "desc": "Comma-separated field names for projection", "default": "all fields"},
            {"flag": "+ filters", "type": "", "desc": "All filter options (see below)", "link_filters": True},
        ],
        "examples": [
            {"comment": "Parse a CSV file with filters", "cmd": 'newsfresh parse data/20250217150000.gkg.csv \\\n  --country US --person "Trump" --limit 10'},
            {"comment": "Parse directly from a zip file", "cmd": "newsfresh parse data/20250217150000.gkg.csv.zip -f json"},
            {"comment": "Output specific fields only", "cmd": "newsfresh parse data/gkg.csv \\\n  -f json --fields document_identifier,source_common_name,tone"},
        ],
        "sample_output_label": "Sample Output — JSON with field projection",
        "sample_output": textwrap.dedent("""\
            [
            {
              "document_identifier": "https://www.washingtonpost.com/politics/2025/02/17/congress-budget-...",
              "source_common_name": "washingtonpost.com",
              "tone": {
                "tone": -1.82,
                "positive_score": 3.12,
                "negative_score": 4.94,
                "polarity": 8.06,
                "activity_ref_density": 15.43,
                "self_group_ref_density": 0.22,
                "word_count": 612
              }
            }
            ]"""),
    },
    {
        "name": "query",
        "title_md": "`query` — Fetch + parse + filter in one step",
        "title_html": '<code>query</code> &mdash; Fetch + Parse + Filter',
        "desc": "Downloads a GKG data file, parses it, applies filters, and outputs results — all in one step. Combines fetch + parse.",
        "section_class": "section-query",
        "args": [
            {"flag": "--latest", "type": "bool", "desc": "Fetch the latest 15-minute update", "default": "false"},
            {"flag": "--date <DATE>", "type": "string", "desc": "Fetch a specific historical file (YYYYMMDDHHMMSS)"},
            {"flag": "--translation", "type": "bool", "desc": "Fetch non-English variant", "default": "false"},
            {"flag": "--persist-data-file", "type": "bool", "desc": "Persist downloaded data to persisted-storage/", "default": "false"},
            {"flag": "-f, --format", "type": "enum", "desc": "Output format", "default": "json"},
            {"flag": "-o, --output", "type": "path", "desc": "Output file", "default": "stdout"},
            {"flag": "--limit <N>", "type": "int", "desc": "Maximum records", "default": "all"},
            {"flag": "--offset <N>", "type": "int", "desc": "Skip first N records", "default": "0"},
            {"flag": "--fields <LIST>", "type": "string", "desc": "Comma-separated field projection", "default": "all fields"},
            {"flag": "+ filters", "type": "", "desc": "All filter options (see below)", "link_filters": True},
        ],
        "examples": [
            {"comment": "Fetch latest and filter by theme", "cmd": 'newsfresh query --country US --theme "CLIMATE_CHANGE" --limit 5'},
            {"comment": "Historical data with tone filter", "cmd": "newsfresh query --date 20250201120000 --tone-min=-10 --tone-max=-2"},
            {"comment": "Persist downloaded files for reuse", "cmd": "newsfresh query --persist-data-file --country US --limit 20"},
            {"comment": "Output in compact TeaLeaf format", "cmd": "newsfresh query --country UK --has-quote -f tealeaf-compact"},
        ],
    },
    {
        "name": "analyze",
        "title_md": "`analyze` — Natural language full-text search",
        "title_html": '<code>analyze</code> &mdash; Full-Text Search &amp; Statistics',
        "desc": "Builds an in-memory Tantivy full-text index with BM25 ranking over GKG records, then runs natural language search queries. Optionally computes aggregate statistics using Polars DataFrames.",
        "section_class": "section-analyze",
        "search_enrichment": [
            {"enrichment": "FIPS country codes expanded to full names (240+ countries)", "example": 'Searching "United States" matches code US'},
            {"enrichment": "ADM1 state/province codes expanded to readable names", "example": 'Searching "California" matches US06'},
            {"enrichment": "Theme code canonicalization", "example": "TAX_FNCACT_PRESIDENT becomes searchable as \"President\""},
        ],
        "args": [
            {"flag": "[FILE]", "type": "path", "desc": "Local .csv or .csv.zip file (optional — use --latest or --date instead)"},
            {"flag": "--search <QUERY>", "type": "string", "desc": "Natural language search query", "required": True},
            {"flag": "--latest", "type": "bool", "desc": "Fetch the latest 15-minute update", "default": "false"},
            {"flag": "--date <DATE>", "type": "string", "desc": "Fetch historical file (YYYYMMDDHHMMSS)"},
            {"flag": "--translation", "type": "bool", "desc": "Non-English variant", "default": "false"},
            {"flag": "--persist-data-file", "type": "bool", "desc": "Persist downloaded data", "default": "false"},
            {"flag": "--limit <N>", "type": "int", "desc": "Maximum number of results", "default": "20"},
            {"flag": "--stats", "type": "bool", "desc": "Show aggregate statistics instead of records", "default": "false"},
            {"flag": "--stats-top-n <N>", "type": "int", "desc": "Number of top entries per frequency table", "default": "10"},
            {"flag": "-f, --format", "type": "enum", "desc": "Output format (when not using --stats)", "default": "tealeaf"},
            {"flag": "-o, --output", "type": "path", "desc": "Output file", "default": "stdout"},
            {"flag": "--fields <LIST>", "type": "string", "desc": "Comma-separated field projection", "default": "all fields"},
            {"flag": "+ filters", "type": "", "desc": "All filter options (see below)", "link_filters": True},
        ],
        "examples": [
            {"comment": "Search the latest GDELT data with natural language", "cmd": 'newsfresh analyze --latest \\\n  --search "elections Congress US economy" --limit 20'},
            {"comment": "Search with additional structured filters", "cmd": 'newsfresh analyze --latest \\\n  --search "climate carbon emissions policy" \\\n  --country US --limit 10'},
            {"comment": "From a local file", "cmd": 'newsfresh analyze data/gkg.csv \\\n  --search "Ukraine Russia ceasefire negotiations" --limit 15'},
            {"comment": "Compact TeaLeaf output for LLM consumption (~47% fewer tokens)", "cmd": 'newsfresh analyze --latest \\\n  --search "AI regulation technology" --limit 10 -f tealeaf-compact'},
            {"comment": "Aggregate statistics with Polars DataFrames", "cmd": 'newsfresh analyze --latest \\\n  --search "US politics" --limit 50 --stats'},
            {"comment": "Top 5 per category", "cmd": 'newsfresh analyze data/gkg.csv \\\n  --search "climate change" --stats --stats-top-n 5 --limit 100'},
        ],
        "extra_md": textwrap.dedent("""\

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
            | `--stats-top-n` | Number of entries per frequency table | 10 |"""),
        "stats_sample_output": textwrap.dedent("""\
            === GDELT Analysis Stats (50 records) ===

            --- Top Themes ---
               1. GENERAL GOVERNMENT            24  (2.0%)
               2. LEADER                        24  (2.0%)
               3. GENERAL1                      23  (2.0%)
               4. GOVERNMENT                    21  (1.8%)
               5. UNGP FORESTS RIVERS OCEANS    20  (1.7%)

            --- Top Countries ---
               1. United States (US)                48  (31.2%)
               2. United Kingdom (UK)                9  (5.8%)
               3. Canada (CA)                        7  (4.5%)
               4. China (CH)                         7  (4.5%)
               5. Australia (AS)                     6  (3.9%)

            --- Tone ---
              Mean: -0.82  Std: 3.45  Range: [-8.12, 4.91]
              Most positive: [4.91] https://www.nytimes.com/.../economy-jobs-report...
              Most negative: [-8.12] https://www.washingtonpost.com/.../congress-budget-crisis...

            --- Top Persons ---
               1. donald trump         9  (4.9%)
               2. elon musk            5  (2.7%)
               3. kamala harris        3  (1.6%)
               4. marco rubio          3  (1.6%)
               5. jerome powell        3  (1.6%)

            --- Top Organizations ---
               1. congress                         3  (1.7%)
               2. federal reserve                  3  (1.7%)
               3. microsoft                        3  (1.7%)
               4. pentagon                         2  (1.2%)
               5. state department                 2  (1.2%)

            --- Top Sources ---
               1. nytimes.com           16  (32.0%)
               2. washingtonpost.com     6  (12.0%)
               3. cnn.com               6  (12.0%)
               4. foxnews.com           5  (10.0%)
               5. politico.com          4  (8.0%)"""),
    },
    {
        "name": "schema",
        "title_md": "`schema` — Print GKG type definitions",
        "title_html": '<code>schema</code> &mdash; Print GKG Type Definitions',
        "desc": "Prints the complete GKG v2.1 record schema in TeaLeaf or JSON Schema format.",
        "section_class": "section-schema",
        "args": [
            {"flag": "-f, --format", "type": "enum", "desc": "tealeaf or json-schema", "default": "tealeaf"},
        ],
        "examples": [
            {"comment": "TeaLeaf schema format", "cmd": "newsfresh schema"},
            {"comment": "JSON Schema format", "cmd": "newsfresh schema -f json-schema"},
        ],
    },
]

FILTERS = [
    {"flag": "--person", "desc": "Person name (case-insensitive substring)", "example": '--person "Trump"'},
    {"flag": "--org", "desc": "Organization name", "example": '--org "United Nations"'},
    {"flag": "--theme", "desc": "GKG theme code", "example": '--theme "TAX_POLICY"'},
    {"flag": "--location", "desc": "Location name", "example": '--location "Washington"'},
    {"flag": "--country", "desc": "FIPS country code", "example": "--country US"},
    {"flag": "--tone-min / --tone-max", "desc": "Tone score range", "example": "--tone-min -5 --tone-max 5"},
    {"flag": "--date-from / --date-to", "desc": "Date range (YYYYMMDD)", "example": "--date-from 20250201"},
    {"flag": "--source", "desc": "Source name", "example": '--source "bbc"'},
    {"flag": "--has-image", "desc": "Only records with a sharing image", "example": "--has-image"},
    {"flag": "--has-quote", "desc": "Only records with quotations", "example": "--has-quote"},
]

FORMATS = [
    {"name": "JSON (pretty)", "flag": "-f json", "desc": "Pretty-printed JSON array (default)"},
    {"name": "JSON (compact)", "flag": "-f json-compact", "desc": "Minified single-line JSON"},
    {"name": "TeaLeaf", "flag": "-f tealeaf", "desc": "Schema-driven format, ~47% fewer tokens than JSON"},
    {"name": "TeaLeaf (compact)", "flag": "-f tealeaf-compact", "desc": "Minified TeaLeaf, additional ~21% savings"},
]


# ============================================================================
# Markdown renderer
# ============================================================================

def render_markdown() -> str:
    lines = []

    lines.append("## Commands")
    lines.append("")

    for cmd in COMMANDS:
        lines.append(f"### {cmd['title_md']}")
        lines.append("")
        lines.append("```bash")
        for ex in cmd["examples"]:
            lines.append(f"# {ex['comment']}")
            lines.append(ex["cmd"])
            lines.append("")
        lines.append("```")

        # Extra markdown content (e.g. analyze enrichment + stats)
        if "extra_md" in cmd:
            lines.append(cmd["extra_md"])

        lines.append("")

    # Filter Options
    lines.append("## Filter Options")
    lines.append("")
    lines.append("All filters compose with AND logic. Available on `parse`, `query`, and `analyze`:")
    lines.append("")
    lines.append("| Flag | Description | Example |")
    lines.append("|------|-------------|---------|")
    for f in FILTERS:
        lines.append(f"| `{f['flag']}` | {f['desc']} | `{f['example']}` |")
    lines.append("")

    # Output Formats
    lines.append("## Output Formats")
    lines.append("")
    lines.append("| Format | Flag | Description |")
    lines.append("|--------|------|-------------|")
    for fmt in FORMATS:
        lines.append(f"| {fmt['name']} | `{fmt['flag']}` | {fmt['desc']} |")
    lines.append("")
    lines.append("Use `--fields` for field projection — output only the fields you need:")
    lines.append("")
    lines.append("```bash")
    lines.append('newsfresh query --limit 5 -f json --fields document_identifier,source_common_name,tone')
    lines.append("```")
    lines.append("")

    return "\n".join(lines)


# ============================================================================
# HTML renderer
# ============================================================================

def h(text: str) -> str:
    """HTML-escape text."""
    return html_mod.escape(text, quote=True)


def render_html() -> str:
    parts = []

    # Global options section
    parts.append("""\
<!-- ==================== GLOBAL OPTIONS ==================== -->
<section id="global" class="section-global">
  <h2>Global Options</h2>
  <p class="cmd-desc">These options are available on every subcommand.</p>
  <table>
    <tr><th>Flag</th><th>Description</th></tr>
    <tr><td><code>-v, --verbose</code></td><td>Increase logging verbosity. Stackable: <code>-v</code> (info), <code>-vv</code> (debug), <code>-vvv</code> (trace)</td></tr>
    <tr><td><code>-q, --quiet</code></td><td>Suppress non-error output</td></tr>
    <tr><td><code>-h, --help</code></td><td>Print help for any command</td></tr>
  </table>
  <pre><span class="prompt">$</span> newsfresh --help
Query and analyze GDELT GKG v2.1 data

Usage: newsfresh [OPTIONS] &lt;COMMAND&gt;

Commands:
  fetch    Download GKG data (latest or historical)
  parse    Parse a local GKG file and output records
  query    Fetch + parse + filter in one step
  schema   Print GKG type definitions
  analyze  NL search + analyze GKG records
  help     Print this message or the help of the given subcommand(s)</pre>
</section>""")

    # Command sections
    for cmd in COMMANDS:
        parts.append("")
        parts.append(f'<!-- ==================== {cmd["name"].upper()} ==================== -->')
        parts.append(f'<section id="{cmd["name"]}" class="{cmd["section_class"]}">')
        parts.append(f'  <h2>{cmd["title_html"]}</h2>')
        parts.append(f'  <p class="cmd-desc">{h(cmd["desc"])}</p>')

        # Search enrichment table (analyze only)
        if "search_enrichment" in cmd:
            parts.append("")
            parts.append("  <h3>Search Enrichment</h3>")
            parts.append("  <table>")
            parts.append("    <tr><th>Enrichment</th><th>Example</th></tr>")
            for se in cmd["search_enrichment"]:
                parts.append(f'    <tr><td>{h(se["enrichment"])}</td><td>{h(se["example"])}</td></tr>')
            parts.append("  </table>")

        # Args table
        parts.append("")
        parts.append("  <h3>Arguments</h3>")
        parts.append("  <table>")
        parts.append("    <tr><th>Flag</th><th>Type</th><th>Description</th><th>Default</th></tr>")
        for arg in cmd["args"]:
            if arg.get("link_filters"):
                parts.append(f'    <tr><td colspan="4" style="color:var(--text-muted)">+ all <a href="#filters" style="color:var(--accent)">filter options</a></td></tr>')
                continue
            flag_html = f'<code>{h(arg["flag"])}</code>'
            type_html = h(arg.get("type", ""))
            desc_html = h(arg["desc"])
            if arg.get("required"):
                default_html = '<span class="tag tag-required">required</span>'
            elif "default" in arg:
                default_html = f'<code>{h(arg["default"])}</code>'
            else:
                default_html = "&mdash;"
            parts.append(f"    <tr><td>{flag_html}</td><td>{type_html}</td><td>{desc_html}</td><td>{default_html}</td></tr>")
        parts.append("  </table>")

        # Examples
        parts.append("")
        if cmd["name"] == "analyze":
            parts.append("  <h3>Examples &mdash; Record Output</h3>")
        else:
            parts.append("  <h3>Examples</h3>")

        pre_lines = []
        for ex in cmd["examples"]:
            if cmd["name"] == "analyze" and "stats" in ex["cmd"]:
                continue  # stats examples go in a separate section
            pre_lines.append(f'<span class="comment"># {h(ex["comment"])}</span>')
            pre_lines.append(f'<span class="prompt">$</span> {h(ex["cmd"])}')
            pre_lines.append("")
        if pre_lines and pre_lines[-1] == "":
            pre_lines.pop()
        parts.append(f'  <pre>{"".join(line + chr(10) for line in pre_lines).rstrip()}</pre>')

        # Sample output
        if "sample_output" in cmd and cmd["name"] != "analyze":
            label = cmd.get("sample_output_label", "Sample Output")
            parts.append("")
            parts.append(f"  <h3>{h(label)}</h3>")
            parts.append(f'  <pre><span class="output">{h(cmd["sample_output"])}</span></pre>')

        # Analyze-specific: stats examples + stats sample output
        if cmd["name"] == "analyze":
            stats_examples = [ex for ex in cmd["examples"] if "stats" in ex["cmd"]]
            if stats_examples:
                parts.append("")
                parts.append('  <h3>Examples &mdash; Aggregate Statistics (<code>--stats</code>)</h3>')
                pre_lines = []
                for ex in stats_examples:
                    pre_lines.append(f'<span class="comment"># {h(ex["comment"])}</span>')
                    pre_lines.append(f'<span class="prompt">$</span> {h(ex["cmd"])}')
                    pre_lines.append("")
                if pre_lines and pre_lines[-1] == "":
                    pre_lines.pop()
                parts.append(f'  <pre>{"".join(line + chr(10) for line in pre_lines).rstrip()}</pre>')

            if "stats_sample_output" in cmd:
                parts.append("")
                parts.append('  <h3>Sample <code>--stats</code> Output</h3>')
                parts.append(f'  <pre class="stats-output"><span class="output">{h(cmd["stats_sample_output"])}</span></pre>')

        parts.append("</section>")

    # Filters section
    parts.append("")
    parts.append('<!-- ==================== FILTERS ==================== -->')
    parts.append('<section id="filters">')
    parts.append("  <h2>Filter Options</h2>")
    parts.append('  <p class="cmd-desc">Available on <code>parse</code>, <code>query</code>, and <code>analyze</code>. All filters compose with <strong>AND</strong> logic.</p>')
    parts.append("  <table>")
    parts.append("    <tr><th>Flag</th><th>Description</th><th>Example</th></tr>")
    for f in FILTERS:
        parts.append(f'    <tr><td><code>{h(f["flag"])}</code></td><td>{h(f["desc"])}</td><td><code>{h(f["example"])}</code></td></tr>')
    parts.append("  </table>")
    parts.append("</section>")

    # Output Formats section
    parts.append("")
    parts.append('<!-- ==================== OUTPUT FORMATS ==================== -->')
    parts.append('<section id="formats">')
    parts.append("  <h2>Output Formats</h2>")
    parts.append('  <p class="cmd-desc">Controlled via <code>-f, --format</code>. The <code>--fields</code> flag enables field projection (JSON only).</p>')
    parts.append("  <table>")
    parts.append("    <tr><th>Format</th><th>Flag</th><th>Description</th></tr>")
    for fmt in FORMATS:
        parts.append(f'    <tr><td>{h(fmt["name"])}</td><td><code>{h(fmt["flag"])}</code></td><td>{h(fmt["desc"])}</td></tr>')
    parts.append("  </table>")
    parts.append("</section>")

    return "\n".join(parts)


# ============================================================================
# Injection logic
# ============================================================================

START_MARKER = "<!-- COMMANDS:START -->"
END_MARKER = "<!-- COMMANDS:END -->"


def inject(filepath: Path, content: str) -> None:
    text = filepath.read_text(encoding="utf-8")
    try:
        start = text.index(START_MARKER)
        end = text.index(END_MARKER) + len(END_MARKER)
    except ValueError as e:
        raise SystemExit(f"ERROR: Markers not found in {filepath}: {e}")

    new_text = text[:start] + START_MARKER + "\n" + content + "\n" + END_MARKER + text[end:]
    filepath.write_text(new_text, encoding="utf-8")
    print(f"  Updated {filepath}")


# ============================================================================
# Main
# ============================================================================

def main():
    root = Path(__file__).resolve().parent.parent
    readme = root / "README.md"
    summary = root / "summary.html"

    print("Generating docs...")
    md_content = render_markdown()
    html_content = render_html()

    inject(readme, md_content)
    inject(summary, html_content)
    print("Done.")


if __name__ == "__main__":
    main()
