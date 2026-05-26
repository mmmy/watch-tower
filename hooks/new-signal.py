#!/usr/bin/env python3
"""Local Watch Tower hook template.

Watch Tower writes a JSON payload to stdin when new signals are detected.
This template appends the raw event and a short summary to files next to
this script. Replace `handle_alert` with your own local action.
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCRIPT_DIR = Path(__file__).resolve().parent
EVENT_LOG = SCRIPT_DIR / "new-signal-events.jsonl"
SUMMARY_LOG = SCRIPT_DIR / "new-signal-summary.log"


def main() -> int:
    raw = sys.stdin.read()
    if not raw.strip():
        return 0

    payload = json.loads(raw)
    append_jsonl(EVENT_LOG, payload)

    alerts = payload.get("alerts", [])
    if isinstance(alerts, list):
        for alert in alerts:
            if isinstance(alert, dict):
                handle_alert(alert, payload)

    return 0


def append_jsonl(path: Path, value: dict[str, Any]) -> None:
    with path.open("a", encoding="utf-8") as file:
        file.write(json.dumps(value, ensure_ascii=False, separators=(",", ":")))
        file.write("\n")


def handle_alert(alert: dict[str, Any], payload: dict[str, Any]) -> None:
    received_at = datetime.now(timezone.utc).isoformat()
    line = (
        f"{received_at} "
        f"event={payload.get('event', 'unknown')} "
        f"symbol={alert.get('symbol', '')} "
        f"period={alert.get('period', '')} "
        f"signalType={alert.get('signalType', '')} "
        f"side={alert.get('side', '')} "
        f"triggerTime={alert.get('triggerTime', '')}"
    )

    with SUMMARY_LOG.open("a", encoding="utf-8") as file:
        file.write(line)
        file.write("\n")


if __name__ == "__main__":
    raise SystemExit(main())
