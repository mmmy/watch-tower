# Watch Tower Native

Watch Tower Native is a Windows desktop monitor for trading signal alerts. It polls a Signal Desk-compatible API for configured symbols, periods, and signal types, then shows the latest signal state in a compact native UI with unread badges, desktop notifications, and a draggable widget.

The active application lives in:

- `native-shell/`: the Windows-native Slint shell

The old Tauri + React fallback has been removed from the main repo path.

## Business Purpose

Watch Tower helps a trader or signal operator keep an always-visible watch list of strategy signals such as `vegas` and `divMacd` for symbols like `BTCUSDT` and `ETHUSDT`.

It is not a general server monitor. Its core job is to watch market signal slots, surface new unread alerts, and let the user mark alerts as read.

## Main Capabilities

- Polls `/api/open/watch-list/symbol-signals` for configured watch groups.
- Groups signals by symbol, signal type, and period.
- Displays long/short direction, trigger time, unread count, and recent signal position.
- Supports manual refresh, always-on-top mode, edge mode, notifications, and sound.
- Provides a small floating widget for unread alert status.
- Marks one or many alerts read through `/api/open/watch-list/symbol-alert/read-status/batch`.
- Includes a CLI command for JSON signal export: `watch-tower signals list`.

## Configuration

Copy `config.yaml.example` and provide the real API host, API key, poll settings, UI settings, and watch groups.

Each enabled group defines:

- `symbol`: market symbol, for example `BTCUSDT`.
- `periods`: watched timeframes, for example `60`, `15`, `5`, `1`, `D`, or `10D`.
- `signal_types`: strategy signal names, for example `vegas` or `divMacd`.
- `timeline_bars`: how many candles are used when showing recent signal position.

## Default Windows Run Path

Use the default Slint shell from Windows PowerShell:

```powershell
npm run dev
```

This default path does not use WSL.

## Default Build And Check

```powershell
npm run check
npm run build
npm run test
```

These commands target `native-shell/`.

## CLI Usage

List signals from a config file:

```powershell
cargo run --manifest-path native-shell/Cargo.toml --bin watch-tower -- signals list --config config.yaml
```

Or pass the query directly:

```powershell
cargo run --manifest-path native-shell/Cargo.toml --bin watch-tower -- signals list --symbol BTCUSDT --periods 15,60,D --signal-types divMacd,vegas --base-url https://your-api-host.com --api-key replace-with-real-api-key
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
