"""Standalone CLI helper scripts for the GDELT monitoring agent.

These are called by Claude via Bash tool, not as MCP servers.
Each function has a CLI entrypoint via the __main__ block.

Usage:
    python tools.py send_email --to user@example.com --subject "Test" --html "<p>Hello</p>"
    python tools.py check_seen --ids "url1" "url2" "url3"
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path

AGENT_DIR = Path(__file__).parent
SEEN_FILE = AGENT_DIR / "seen.json"


def load_config() -> dict:
    with open(AGENT_DIR / "config.json") as f:
        return json.load(f)


# ---------------------------------------------------------------------------
# send_email — send an email via SendGrid
# ---------------------------------------------------------------------------

def cmd_send_email(args: argparse.Namespace) -> None:
    api_key = os.environ.get("SENDGRID_API_KEY")
    if not api_key:
        print("ERROR: SENDGRID_API_KEY not set", file=sys.stderr)
        sys.exit(1)

    config = load_config()
    from_env_key = config["email"]["from_env"]
    from_email = os.environ.get(from_env_key)
    if not from_email:
        print(f"ERROR: {from_env_key} not set", file=sys.stderr)
        sys.exit(1)

    to_env_key = config["email"].get("to_env", "NEWS_RECIPIENT")
    to_email = os.environ.get(to_env_key) or args.to
    if not to_email:
        print(f"ERROR: {to_env_key} not set and --to not provided", file=sys.stderr)
        sys.exit(1)

    from sendgrid import SendGridAPIClient
    from sendgrid.helpers.mail import Mail

    message = Mail(
        from_email=from_email,
        to_emails=to_email,
        subject=args.subject,
        html_content=args.html,
    )

    sg = SendGridAPIClient(api_key)
    response = sg.send(message)
    print(f"Email sent to {args.to} — status {response.status_code}")


# ---------------------------------------------------------------------------
# check_seen — dedup articles by document_identifier
# ---------------------------------------------------------------------------

def _load_seen() -> dict[str, float]:
    if SEEN_FILE.exists():
        with open(SEEN_FILE) as f:
            return json.load(f)
    return {}


def _save_seen(seen: dict[str, float]) -> None:
    with open(SEEN_FILE, "w") as f:
        json.dump(seen, f, indent=2)


def cmd_check_seen(args: argparse.Namespace) -> None:
    seen = _load_seen()
    now = time.time()
    cutoff = now - 86400  # 24 hours

    # Prune old entries
    seen = {k: v for k, v in seen.items() if v > cutoff}

    # Find new IDs
    doc_ids = args.ids
    new_ids = [did for did in doc_ids if did not in seen]

    # Mark all as seen
    for did in doc_ids:
        seen[did] = now

    _save_seen(seen)

    print(json.dumps({
        "new_ids": new_ids,
        "total_checked": len(doc_ids),
        "new_count": len(new_ids),
    }))


# ---------------------------------------------------------------------------
# CLI entrypoint
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="GDELT monitor helper tools")
    sub = parser.add_subparsers(dest="command", required=True)

    # send_email
    p_email = sub.add_parser("send_email", help="Send email via SendGrid")
    p_email.add_argument("--to", required=False, default=None, help="Recipient (falls back to NEWS_RECIPIENT env var)")
    p_email.add_argument("--subject", required=True)
    p_email.add_argument("--html", required=True, help="HTML body content")

    # check_seen
    p_seen = sub.add_parser("check_seen", help="Dedup check for document IDs")
    p_seen.add_argument("--ids", nargs="+", required=True, help="Document IDs to check")

    args = parser.parse_args()

    if args.command == "send_email":
        cmd_send_email(args)
    elif args.command == "check_seen":
        cmd_check_seen(args)


if __name__ == "__main__":
    main()
