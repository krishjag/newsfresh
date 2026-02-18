"""GDELT News Monitor Agent — newsfresh CLI + LLM providers.

Monitors GDELT GKG data every 15 minutes, searches for articles matching
interest profiles, and emails notifications for new matches via SendGrid.

Supports multiple LLM providers (config.json "provider" field):
  - "claude"  — uses Claude Agent SDK with built-in Bash tool
  - "openai"  — uses OpenAI chat completions with function calling

Architecture: The LLM calls shell commands to run:
  - newsfresh.exe analyze   — NL search on GDELT data
  - python tools.py         — email sending and dedup

Usage:
    export ANTHROPIC_API_KEY="your-key"   # for claude provider
    export OPENAI_API_KEY="your-key"      # for openai provider
    export SENDGRID_API_KEY="your-key"
    python agent/main.py              # continuous monitoring loop
    python agent/main.py --once       # single cycle then exit
"""

import argparse
import asyncio
import json
import logging
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from claude_agent_sdk import (
    AssistantMessage,
    ClaudeAgentOptions,
    ResultMessage,
    TextBlock,
    query,
)

try:
    from openai import AsyncOpenAI
except ImportError:
    AsyncOpenAI = None

# Allow running inside a Claude Code session (e.g. during development)
for key in list(os.environ):
    if key.startswith("CLAUDE") or key == "CLAUDECODE":
        os.environ.pop(key, None)

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("gdelt-monitor")

AGENT_DIR = Path(__file__).parent

# ---------------------------------------------------------------------------
# OpenAI tool definition and cost tracking
# ---------------------------------------------------------------------------

OPENAI_TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "run_bash",
            "description": (
                "Execute a bash/shell command and return stdout and stderr. "
                "Use this to run newsfresh searches, check_seen dedup, "
                "and send_email commands."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute",
                    }
                },
                "required": ["command"],
            },
        },
    }
]

OPENAI_PRICING = {
    "gpt-4o": {"input": 2.50, "output": 10.00},
    "gpt-4o-mini": {"input": 0.15, "output": 0.60},
}


def run_bash(command: str, cwd: str | None = None) -> str:
    """Execute a bash command and return stdout+stderr. Timeout 120s."""
    try:
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            timeout=120,
            cwd=cwd,
        )
        output = ""
        if result.stdout:
            output += result.stdout
        if result.stderr:
            output += "\n[STDERR]\n" + result.stderr
        if result.returncode != 0:
            output += f"\n[Exit code: {result.returncode}]"
        output = output or "(no output)"
        # Truncate large outputs to avoid blowing up the context window
        if len(output) > 50_000:
            output = output[:50_000] + f"\n[TRUNCATED: {len(output)} chars total]"
        return output
    except subprocess.TimeoutExpired:
        return "[ERROR: Command timed out after 120 seconds]"
    except Exception as e:
        return f"[ERROR: {e}]"


def calculate_openai_cost(model: str, usage) -> float:
    """Calculate approximate cost in USD from OpenAI usage object."""
    pricing = OPENAI_PRICING.get(model, {"input": 2.50, "output": 10.00})
    input_cost = (usage.prompt_tokens / 1_000_000) * pricing["input"]
    output_cost = (usage.completion_tokens / 1_000_000) * pricing["output"]
    return input_cost + output_cost


# ---------------------------------------------------------------------------
# Config and prompt helpers
# ---------------------------------------------------------------------------

def load_config() -> dict:
    with open(AGENT_DIR / "config.json") as f:
        return json.load(f)


def resolve_llm_provider(config: dict) -> dict:
    """Look up the active LLM provider from the providers list.

    Returns a dict with "id", "provider", and "model" keys.
    Falls back to claude-haiku if not configured.
    """
    active_id = config.get("active_llm_provider")
    providers = config.get("llm_providers", [])

    for p in providers:
        if p["id"] == active_id:
            return p

    # Fallback if ID not found or not configured
    if providers:
        log.warning("active_llm_provider '%s' not found, using first provider", active_id)
        return providers[0]

    return {"id": "default", "provider": "claude", "model": "claude-haiku-4-5-20251001"}


def build_system_prompt(config: dict) -> str:
    """Build the system prompt with tool paths and instructions."""
    llm = resolve_llm_provider(config)
    provider = llm["provider"]
    newsfresh_path = str((AGENT_DIR / config["newsfresh_path"]).resolve()).replace("\\", "/")
    tools_path = str((AGENT_DIR / "tools.py").resolve()).replace("\\", "/")

    if provider == "openai":
        tool_intro = (
            "You have a tool called `run_bash` that executes shell commands. "
            "Use it to run these commands:"
        )
    else:
        tool_intro = "You have access to the Bash tool. Use it to run these commands:"

    return f"""\
You are a GDELT news monitoring agent. Your job is to check for new articles \
matching the user's interest profiles and send email notifications with \
insightful summaries for any new findings.

{tool_intro}

## 1. Search GDELT news (TeaLeaf format — compact, token-efficient)
```bash
"{newsfresh_path}" analyze --latest --persist-data-file --search "<query>" --limit <N> -f tealeaf [filters]
```
Optional filters: --country XX, --person "Name", --org "Org", --theme "Theme"
Returns results in TeaLeaf format — a compact schema-driven format that uses \
~47% fewer tokens than JSON. The output starts with @struct definitions \
(schema), followed by @table rows (data). Key fields in each record:
- relevance_score (first field)
- document_identifier (the article URL)
- v2_counts_xml_persons, v2_counts_xml_organizations (people, orgs)
- v2_enhanced_themes (topics), v2_enhanced_locations (places)
- extras_xml_source_urls (source), v1_5_tone (sentiment)

## 2. Get document IDs (JSON format — for dedup)
```bash
"{newsfresh_path}" analyze --latest --persist-data-file --search "<query>" --limit <N> -f json --fields document_identifier
```
Use JSON with --fields to extract just the document_identifier values for dedup.

## 3. Dedup check
```bash
python3 "{tools_path}" check_seen --ids "<url1>" "<url2>" ...
```
Returns JSON with new_ids (unseen), total_checked, new_count.

## 4. Send email
```bash
python3 "{tools_path}" send_email --to "<email>" --subject "<subject>" --html "<html_body>"
```
Sends an HTML email via SendGrid. Only call when there are new articles.

## Instructions

For each interest profile:

1. Run `newsfresh analyze` in **TeaLeaf format** (default) with the profile's \
search query, limit, and filters. Read and understand the results.
2. Also run `newsfresh analyze` with `-f json --fields document_identifier` \
to get just the article URLs for dedup.
3. Run `check_seen` with those document IDs to find new (unseen) articles.
4. If there are new articles, **summarize each article** based on what you \
read from the TeaLeaf output:
   - Who: key persons and organizations mentioned
   - What: main themes and topics
   - Where: locations involved
   - Sentiment: tone (positive/negative/neutral)
   - Source and link
5. Format a clean, informative HTML email with:
   - Profile name as heading
   - For each new article: your summary paragraph, relevance score, \
and a clickable link to the source
   - A brief overall analysis section at the end summarizing the key \
trends across all articles for this profile
6. Run `send_email` with the recipient, subject, and HTML body.
7. If no new articles for a profile, skip emailing.

After all profiles, output a brief summary of what was found and sent.

Be efficient — batch document IDs in a single check_seen call per profile.\
"""


def build_prompt(config: dict) -> str:
    """Build the user prompt with current profiles and settings."""
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    to_env_key = config["email"].get("to_env", "NEWS_RECIPIENT")
    email_to = os.environ.get(to_env_key, "")
    subject_prefix = config["email"]["subject_prefix"]
    threshold = config.get("relevance_threshold", 1.0)

    profiles_text = json.dumps(config["profiles"], indent=2)

    return f"""\
Current time: {now}

Send notifications to: {email_to}
Subject prefix: {subject_prefix}
Minimum relevance score: {threshold}

Interest profiles to monitor:
{profiles_text}

Please check each profile for new articles and send email notifications \
for any new findings."""


# ---------------------------------------------------------------------------
# Provider: Claude
# ---------------------------------------------------------------------------

async def run_cycle_claude(config: dict, llm: dict) -> None:
    """Run a single monitoring cycle using Claude Agent SDK."""
    model = llm.get("model", "claude-haiku-4-5-20251001")
    log.info("Using Claude provider [%s], model=%s", llm["id"], model)

    options = ClaudeAgentOptions(
        model=model,
        allowed_tools=["Bash"],
        permission_mode="bypassPermissions",
        system_prompt=build_system_prompt(config),
        max_turns=30,
        cwd=str(AGENT_DIR),
    )

    prompt = build_prompt(config)

    try:
        async for message in query(prompt=prompt, options=options):
            if isinstance(message, AssistantMessage):
                for block in message.content:
                    if isinstance(block, TextBlock):
                        log.info("Agent: %s", block.text[:200])
            elif isinstance(message, ResultMessage):
                cost = getattr(message, "total_cost_usd", None)
                if cost:
                    log.info("Cycle complete. Cost: $%.4f", cost)
                else:
                    log.info("Cycle complete.")
    except Exception as e:
        log.error("Cycle failed: %s", e, exc_info=True)


# ---------------------------------------------------------------------------
# Provider: OpenAI
# ---------------------------------------------------------------------------

async def run_cycle_openai(config: dict, llm: dict) -> None:
    """Run a single monitoring cycle using OpenAI."""
    if AsyncOpenAI is None:
        log.error("openai package not installed. Run: pip install openai")
        return

    model = llm.get("model", "gpt-4o-mini")
    log.info("Using OpenAI provider [%s], model=%s", llm["id"], model)

    client = AsyncOpenAI()

    system_prompt = build_system_prompt(config)
    user_prompt = build_prompt(config)
    cwd = str(AGENT_DIR)

    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": user_prompt},
    ]

    total_cost = 0.0
    max_turns = 30

    try:
        for turn in range(max_turns):
            log.info("OpenAI turn %d/%d", turn + 1, max_turns)

            response = await client.chat.completions.create(
                model=model,
                messages=messages,
                tools=OPENAI_TOOLS,
                tool_choice="auto",
            )

            if response.usage:
                total_cost += calculate_openai_cost(model, response.usage)

            choice = response.choices[0]
            assistant_msg = choice.message

            if assistant_msg.content:
                log.info("Agent: %s", assistant_msg.content[:200])

            # Append assistant message to conversation history
            messages.append(assistant_msg.model_dump(exclude_none=True))

            # If no tool calls, the model is done
            if not assistant_msg.tool_calls:
                log.info("Cycle complete. Approx cost: $%.6f", total_cost)
                break

            # Execute each tool call
            for tool_call in assistant_msg.tool_calls:
                fn_name = tool_call.function.name
                fn_args = json.loads(tool_call.function.arguments)

                if fn_name == "run_bash":
                    command = fn_args.get("command", "")
                    log.info("Bash: %s", command[:120])
                    result = run_bash(command, cwd=cwd)
                    log.info("Result: %s", result[:200])
                else:
                    result = f"Unknown tool: {fn_name}"

                messages.append({
                    "role": "tool",
                    "tool_call_id": tool_call.id,
                    "content": result,
                })
        else:
            log.warning("OpenAI agent hit max turns (%d)", max_turns)
            log.info("Total approx cost: $%.6f", total_cost)
    except Exception as e:
        log.error("Cycle failed: %s", e, exc_info=True)


# ---------------------------------------------------------------------------
# Dispatcher and main loop
# ---------------------------------------------------------------------------

async def run_cycle(config: dict) -> None:
    """Resolve the active LLM provider and dispatch."""
    llm = resolve_llm_provider(config)
    provider = llm["provider"]

    if provider == "openai":
        await run_cycle_openai(config, llm)
    elif provider == "claude":
        await run_cycle_claude(config, llm)
    else:
        log.error("Unknown provider type: %s (expected 'claude' or 'openai')", provider)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="GDELT News Monitor Agent")
    parser.add_argument("--once", action="store_true", help="Run a single cycle then exit")
    parser.add_argument(
        "--interval", type=int, default=None,
        help="Override interval_minutes from config.json",
    )
    return parser.parse_args()


def get_interval(config: dict, cli_override: int | None) -> int:
    """Return interval in minutes, CLI flag takes precedence over config."""
    if cli_override is not None:
        return cli_override
    return config.get("interval_minutes", 15)


async def main() -> None:
    args = parse_args()
    config = load_config()
    llm = resolve_llm_provider(config)
    interval_min = get_interval(config, args.interval)

    log.info(
        "GDELT Monitor started — llm=%s (%s/%s), %d profiles, %d-min interval%s",
        llm["id"],
        llm["provider"],
        llm["model"],
        len(config["profiles"]),
        interval_min,
        " (single run)" if args.once else "",
    )

    while True:
        await run_cycle(config)

        if args.once:
            break

        log.info("Sleeping %d minutes until next cycle...", interval_min)
        await asyncio.sleep(interval_min * 60)

        # Reload config each cycle to pick up changes
        config = load_config()
        interval_min = get_interval(config, args.interval)


if __name__ == "__main__":
    asyncio.run(main())
