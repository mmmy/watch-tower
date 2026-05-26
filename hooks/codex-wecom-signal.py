#!/usr/bin/env python3
"""Run Codex signal analysis and send the result to WeCom.

Runtime requirements:
- Put `.env` in the Watch Tower process working directory.
- Add `WECOM_BOT_KEY=...` to that file.
- Configure Watch Tower to run this script as the local hook.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
LOG_PATH = SCRIPT_DIR / "codex-wecom-signal.log"
WECOM_WEBHOOK_BASE = "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key="
CODEX_TIMEOUT_SECS = 900
WECOM_TIMEOUT_SECS = 20
MAX_MARKDOWN_CHARS = 3900
DEFAULT_CODEX_CD = r"F:\workspace\trade-agent"


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read() or "{}")
        env = load_env(Path.cwd())
        log(
            "hook started "
            f"cwd={Path.cwd()} "
            f"signal_types={env.get('SIGNAL_TYPES', '') or '<all>'} "
            f"min_signal_period={env.get('MIN_SIGNAL_PERIOD', '') or '<none>'} "
            f"codex_cd={env.get('CODEX_CD', '') or DEFAULT_CODEX_CD}"
        )
        bot_key = env.get("WECOM_BOT_KEY") or os.environ.get("WECOM_BOT_KEY", "")
        if not bot_key:
            log("missing WECOM_BOT_KEY")
            return 0

        alerts = payload.get("alerts", [])
        if not isinstance(alerts, list):
            log("payload alerts is not a list")
            return 0

        log(f"received alerts count={len(alerts)}")
        for alert in alerts:
            if not isinstance(alert, dict):
                log("skipped non-object alert")
                continue
            if not should_handle_alert(alert, env):
                log(f"skipped by filters: {format_alert_label(alert)}")
                continue
            prompt = build_codex_prompt(alert)
            result = run_codex(prompt, env, format_alert_label(alert))
            send_wecom_markdown(bot_key, build_wecom_markdown(alert, result))
    except Exception as error:  # noqa: BLE001 - hook failures must not affect Watch Tower.
        log(f"hook failed: {error}")

    return 0


def load_env(runtime_dir: Path) -> dict[str, str]:
    env_path = runtime_dir / ".env"
    values: dict[str, str] = {}
    if not env_path.exists():
        return values

    for line in env_path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or "=" not in stripped:
            continue
        key, value = stripped.split("=", 1)
        values[key.strip()] = value.strip().strip('"').strip("'")

    return values


def build_codex_prompt(alert: dict[str, Any]) -> str:
    signal_json = json.dumps(alert, ensure_ascii=False, separators=(",", ":"))
    # return f"/sma-trend 新信号触发:{signal_json}, 请分析是否可以在本级别(作为小级别超买超卖)等待附近机会入场, 需要大级别布林带趋势支持, 如果分析不适合入场, 那么给出做多做空级别推荐, 并给出预计等待时间"
    # return f"/sma-trend 做多做空级别推荐, 并给出预计等待时间,新信号触发:{signal_json}, 本级别适合附近等待入场吗"
    return "/sma-trend btcusdt做多做空级别推荐, 并给出预计等待时间"


def should_handle_alert(alert: dict[str, Any], env: dict[str, str]) -> bool:
    allowed = parse_csv(env.get("SIGNAL_TYPES", ""))
    if allowed:
        signal_type = str(alert.get("signalType", "")).strip().lower()
        if signal_type not in {value.lower() for value in allowed}:
            return False

    min_period = period_to_ms(env.get("MIN_SIGNAL_PERIOD", ""))
    if min_period is None:
        return True

    period = period_to_ms(str(alert.get("period", "")))
    return period is not None and period >= min_period


def parse_csv(value: str) -> list[str]:
    return [item.strip() for item in value.split(",") if item.strip()]


def period_to_ms(period: str) -> int | None:
    normalized = period.strip().upper()
    if not normalized:
        return None

    if normalized == "W":
        return 7 * 24 * 60 * 60 * 1000

    if normalized.endswith("D"):
        amount = normalized[:-1] or "1"
        return parse_positive_int(amount, 24 * 60 * 60 * 1000)

    if normalized.endswith("S"):
        return parse_positive_int(normalized[:-1], 1000)

    return parse_positive_int(normalized, 60 * 1000)


def parse_positive_int(value: str, multiplier: int) -> int | None:
    try:
        amount = int(value)
    except ValueError:
        return None

    if amount <= 0:
        return None
    return amount * multiplier


def run_codex(prompt: str, env: dict[str, str], alert_label: str = "") -> str:
    command = resolve_codex_command()
    log(f"codex start: command={command} alert={alert_label}")
    try:
        completed = subprocess.run(
            [command, *build_codex_args(prompt, env)],
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            timeout=CODEX_TIMEOUT_SECS,
            check=False,
        )
    except subprocess.TimeoutExpired:
        log(f"codex timeout after {CODEX_TIMEOUT_SECS}s: alert={alert_label}")
        return f"codex exec timeout after {CODEX_TIMEOUT_SECS}s"

    output = completed.stdout.strip()
    error = completed.stderr.strip()
    log(
        "codex done: "
        f"alert={alert_label} "
        f"exit={completed.returncode} "
        f"stdout_chars={len(output)} "
        f"stderr_chars={len(error)}"
    )
    if completed.returncode == 0:
        return output or "(codex returned no output)"

    parts = [f"codex exec failed with exit code {completed.returncode}"]
    if output:
        parts.append(output)
    if error:
        parts.append(error)
    return "\n\n".join(parts)


def build_codex_args(prompt: str, env: dict[str, str]) -> list[str]:
    codex_cd = env.get("CODEX_CD", "").strip() or DEFAULT_CODEX_CD
    return ["exec", "--cd", codex_cd, prompt]


def resolve_codex_command() -> str:
    env_command = os.environ.get("CODEX_BIN", "").strip()
    if env_command:
        return env_command

    if os.name == "nt":
        return shutil.which("codex.cmd") or "codex.cmd"

    return shutil.which("codex") or "codex"


def build_wecom_markdown(alert: dict[str, Any], analysis: str) -> str:
    side = "超买" if alert.get("side", 1) >= 0 else "超卖"
    title = (
        f"**Watch Tower 新信号分析**\n"
        f"> 标的: {alert.get('symbol', '')}\n"
        f"> 周期: {alert.get('period', '')}\n"
        f"> 信号: {alert.get('signalType', '')}\n"
        f"> 方向: {side}\n"
        f"> 触发时间: {alert.get('triggerTime', '')}\n\n"
    )
    content = title + analysis.strip()
    if len(content) <= MAX_MARKDOWN_CHARS:
        return content
    return content[: MAX_MARKDOWN_CHARS - 20] + "\n\n...(已截断)"


def send_wecom_markdown(bot_key: str, markdown: str) -> None:
    body = json.dumps(
        {
            "msgtype": "markdown",
            "markdown": {
                "content": markdown,
            },
        },
        ensure_ascii=False,
    ).encode("utf-8")
    request = urllib.request.Request(
        WECOM_WEBHOOK_BASE + bot_key,
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        with urllib.request.urlopen(request, timeout=WECOM_TIMEOUT_SECS) as response:
            response_body = response.read().decode("utf-8", errors="replace")
            try:
                result = json.loads(response_body)
            except json.JSONDecodeError:
                log(f"wecom returned non-json response: {response_body}")
                return

            errcode = result.get("errcode")
            if errcode == 0:
                log("wecom send ok")
            else:
                log(f"wecom send failed: {response_body}")
    except urllib.error.URLError as error:
        log(f"wecom send failed: {error}")


def format_alert_label(alert: dict[str, Any]) -> str:
    return (
        f"symbol={alert.get('symbol', '')} "
        f"period={alert.get('period', '')} "
        f"signalType={alert.get('signalType', '')} "
        f"side={alert.get('side', '')} "
        f"triggerTime={alert.get('triggerTime', '')}"
    )


def log(message: str) -> None:
    timestamp = datetime.now(timezone.utc).isoformat()
    with LOG_PATH.open("a", encoding="utf-8") as file:
        file.write(f"{timestamp} {message}\n")


if __name__ == "__main__":
    raise SystemExit(main())
