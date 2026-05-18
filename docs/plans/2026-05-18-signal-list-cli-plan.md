# Signal List CLI Plan

## Goal

Build a small read-only CLI for AI agent skills to fetch watch-tower signal data from the server and enrich each signal with the number of candles elapsed since it triggered.

The CLI is not meant for human browsing. It should optimize for deterministic machine use:

- Non-interactive command-line flags.
- JSON-only stdout.
- Diagnostics and logs on stderr.
- Stable field names that an AI agent skill can parse reliably.

## Scope

Implement only the list/read side of the server API for now:

```http
POST /api/open/watch-list/symbol-signals
```

Do not include read/unread mutation commands in the first version. The current need is analysis, not state changes.

## Recommended Command

Default config-driven usage:

```powershell
watch-tower signals list --config config.yaml
```

Optional direct query usage:

```powershell
watch-tower signals list --symbol BTCUSDT --periods 15,60,D --signal-types divMacd,vegas
```

When `--config` is used, the CLI should load enabled groups from `config.yaml` and request every configured period and signal type. When explicit flags are used, the CLI should query exactly those values.

## Output Shape

Stdout should contain one JSON object. On success:

```json
{
  "ok": true,
  "meta": {
    "requestedAt": 1710000000000,
    "count": 1
  },
  "data": [
    {
      "symbol": "BTCUSDT",
      "period": "15",
      "signalType": "divMacd",
      "side": 1,
      "triggerTime": 1710000000000,
      "read": false,
      "periodMs": 900000,
      "elapsedMs": 1800000,
      "elapsedCandles": 2
    }
  ]
}
```

On failure:

```json
{
  "ok": false,
  "error": {
    "code": "request_failed",
    "message": "request failed: 401 unauthorized"
  }
}
```

## Enrichment Rules

The server response is raw signal data. The CLI should add derived timing fields for every returned signal.

Use the same period parsing rules as the client:

- `W` means 7 days.
- `D` means 1 day.
- Values ending in `D`, such as `10D`, mean that many days.
- Values ending in `S`, such as `45S`, mean that many seconds.
- Plain numeric values, such as `15` or `240`, mean minutes.

For each signal:

```text
periodMs = parsed period duration in milliseconds
elapsedMs = max(nowMs - triggerTime, 0)
elapsedCandles = floor(elapsedMs / periodMs)
```

If `triggerTime <= 0` or the period cannot be parsed:

```json
{
  "periodMs": null,
  "elapsedMs": null,
  "elapsedCandles": null
}
```

Do not filter by active window or timeline size. The agent should receive all configured levels, including signals that triggered 1 candle ago and signals that triggered 10000 candles ago.

## Implementation Notes

Add a second Rust binary to the existing crate:

```text
native-shell/src/bin/watch-tower.rs
```

Extract shared non-UI logic into modules that both the desktop runtime and CLI can use:

```text
native-shell/src/api_client.rs
native-shell/src/signal_time.rs
```

`api_client.rs` should own:

- Building authenticated requests with `x-api-key`.
- Calling `/api/open/watch-list/symbol-signals`.
- Deserializing the server response.

`signal_time.rs` should own:

- Period-to-milliseconds parsing.
- Elapsed milliseconds calculation.
- Elapsed candle calculation.

The existing UI currently computes the timeline position in `app_state.rs` using equivalent period parsing and elapsed-candle math. The new module should become the single source of truth to avoid future drift between the UI and CLI.

## Non-Goals

- No `--only-active` filter.
- No `--timeline-bars` filter.
- No table output.
- No interactive prompts.
- No read/unread mutation commands in the first version.

## Verification

Add tests for:

- Period parsing: `W`, `D`, `10D`, `45S`, and numeric minute values.
- `elapsedCandles` calculation.
- Null derived fields for invalid period or missing trigger time.
- CLI JSON output shape for a mocked list response.
