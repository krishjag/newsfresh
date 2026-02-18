# newsfresh Agent — Personalized GDELT News Monitor

An LLM-powered agent that monitors [GDELT](https://www.gdeltproject.org/) every 15 minutes, searches for articles matching your interest profiles, and sends email notifications with AI-generated summaries.

## How It Works

```
config.json          LLM (Claude / OpenAI)        newsfresh CLI
  profiles ──────►  orchestrates the workflow  ──►  analyze --latest
  email config       via bash tool calls            --search "..."
  llm provider                                          │
                           │                            ▼
                     summarizes articles          BM25-ranked results
                     checks dedup (seen.json)     in TeaLeaf format
                     sends email (SendGrid)
```

The agent delegates the entire workflow to an LLM. On each cycle the LLM:

1. Runs `newsfresh analyze` to search GDELT data for each interest profile
2. Extracts article URLs and checks them against `seen.json` for deduplication
3. Summarizes new articles (who, what, where, sentiment)
4. Sends an HTML email via SendGrid with the summaries
5. Sleeps for 15 minutes and repeats

## Prerequisites

- **newsfresh CLI** — install via any method in the [main README](../README.md#installation):
  ```bash
  cargo install newsfresh            # from crates.io
  # or download from GitHub Releases
  # or build from source: cargo build --release
  ```
- **Python 3.10+**

## Quick Start

### 1. Install Python dependencies

```bash
pip install -r requirements.txt
```

### 2. Set environment variables

```bash
export ANTHROPIC_API_KEY="sk-ant-..."          # for Claude providers
export OPENAI_API_KEY="sk-..."                 # for OpenAI providers
export SENDGRID_API_KEY="SG...."               # email sending
export SENDGRID_VERIFIED_SENDER="you@mail.com" # verified sender address
export NEWS_RECIPIENT="you@mail.com"             # notification recipient
```

### 3. Configure interest profiles

Edit `config.json`:

```json
{
  "profiles": [
    {
      "name": "US Politics",
      "search": "Congress White House politics legislation",
      "filters": { "country": "US" },
      "limit": 5
    },
    {
      "name": "AI Policy",
      "search": "artificial intelligence regulation policy",
      "filters": { "country": "US" },
      "limit": 10
    }
  ]
}
```

### 4. Run

```bash
# Single cycle (test)
python main.py --once

# Continuous monitoring (24/7)
python main.py

# Override the polling interval (minutes)
python main.py --interval 5
```

## Configuration

All settings live in `config.json`. The agent reloads this file at the start of every cycle, so you can change profiles or switch LLM providers without restarting.

### LLM Providers

```json
{
  "active_llm_provider": "claude-haiku",
  "llm_providers": [
    { "id": "claude-haiku",  "provider": "claude", "model": "claude-haiku-4-5-20251001" },
    { "id": "claude-sonnet", "provider": "claude", "model": "claude-sonnet-4-5-20250929" },
    { "id": "openai-mini",   "provider": "openai", "model": "gpt-4o-mini" },
    { "id": "openai-4o",     "provider": "openai", "model": "gpt-4o" }
  ]
}
```

Switch providers by changing `active_llm_provider` to any `id` from the list.

| Provider | How it works |
|----------|-------------|
| `claude` | Uses Claude Agent SDK with built-in Bash tool support. Cost tracked via `ResultMessage.total_cost_usd`. |
| `openai` | Uses OpenAI Chat Completions with function calling (`run_bash` tool). Cost estimated from token usage. |

### Interest Profiles

Each profile defines a search topic:

```json
{
  "name": "Display name (used in email subject)",
  "search": "Natural language search query",
  "filters": {
    "country": "US",
    "person": "Trump",
    "org": "Congress",
    "theme": "CLIMATE_CHANGE"
  },
  "limit": 10
}
```

All filters are optional and match the `newsfresh` CLI flags. See the main [README](../README.md) for the full filter list.

### Other Settings

| Key | Description | Default |
|-----|-------------|---------|
| `interval_minutes` | Minutes between monitoring cycles | `15` |
| `relevance_threshold` | Minimum BM25 relevance score | `1.0` |
| `newsfresh_path` | Path to newsfresh binary (relative to agent/ or absolute) | `../target/release/newsfresh.exe` |
| `email.to_env` | Env var name for recipient address | `NEWS_RECIPIENT` |
| `email.from_env` | Env var name for sender address | `SENDGRID_VERIFIED_SENDER` |
| `email.subject_prefix` | Email subject prefix | `[GDELT Monitor]` |

## Deduplication

The agent tracks seen articles in `seen.json` — a map of article URLs to timestamps. Articles older than 24 hours are pruned automatically. This prevents duplicate notifications when the same article appears in consecutive GDELT snapshots.

## Architecture

```
agent/
  main.py          # Agent loop, LLM dispatch (Claude + OpenAI)
  tools.py         # CLI helpers: check_seen (dedup), send_email (SendGrid)
  config.json      # Profiles, LLM config, email settings
  seen.json        # Dedup state (auto-generated)
  requirements.txt # Python deps
```

### main.py

- `load_config()` / `resolve_llm_provider()` — reads config, resolves active LLM
- `build_system_prompt()` — constructs the LLM system prompt with tool paths and instructions
- `build_prompt()` — constructs the user prompt with current profiles and settings
- `run_cycle_claude()` — runs one cycle via Claude Agent SDK (async streaming)
- `run_cycle_openai()` — runs one cycle via OpenAI function calling (multi-turn loop)
- `main()` — continuous loop: run cycle, sleep, reload config, repeat

### tools.py

Called by the LLM via bash commands, not imported directly:

- `check_seen --ids <url1> <url2> ...` — returns JSON with `new_ids`, `total_checked`, `new_count`
- `send_email --to <email> --subject <subject> --html <body>` — sends HTML email via SendGrid

## Dependencies

```
claude-agent-sdk>=0.1.36   # Claude Agent SDK with Bash tool
sendgrid>=6.11              # Email via SendGrid API
openai>=1.0                 # OpenAI API (optional, for OpenAI provider)
```
