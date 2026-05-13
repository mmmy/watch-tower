# API Notes

## Batch Signal Read Status

Marks multiple symbol alert read states in one request.

```http
POST /api/open/watch-list/symbol-alert/read-status/batch
```

Headers:

```http
x-api-key: <api key>
content-type: application/json
```

Request body:

```json
{
  "items": [
    {
      "symbol": "BTCUSDT",
      "period": "15",
      "signalType": "divMacd",
      "read": true
    }
  ]
}
```

Response body:

```json
{
  "success": 2,
  "failed": 1,
  "results": [
    {
      "symbol": "BTCUSDT",
      "period": "15",
      "signalType": "divMacd",
      "read": true,
      "success": false,
      "reason": "signal_not_found"
    }
  ]
}
```

Client behavior:

- The native shell calls this endpoint for all symbol alert read-status writes.
- Single-item read or unread toggles are sent as an `items` array with one entry.
- Group unread-count badge clicks are sent as an `items` array with all unread entries in that group.
- Only result entries with `"success": true` are applied to local runtime state.
- Failed result entries remain unread after the runtime snapshot is refreshed.
