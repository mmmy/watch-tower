import http from "node:http";

const port = Number(process.env.MOCK_API_PORT || 8787);
const expectedApiKey = process.env.MOCK_API_KEY || "demo-key";
const logRequests = process.env.MOCK_API_LOG !== "0";

const DEFAULT_PERIODS = [
  "10D",
  "W",
  "4D",
  "3D",
  "2D",
  "D",
  "720",
  "480",
  "360",
  "240",
  "180",
  "120",
  "90",
  "60",
  "45",
  "30",
  "20",
  "15",
  "10",
  "8",
  "5",
  "4",
  "3",
  "2",
  "1",
] ;

const DEFAULT_SIGNAL_TYPES = ["vegas", "divMacd", "tdMd", "atrIndex"];

const signalStore = new Map();

function now() {
  return Date.now();
}

function splitCsv(value, fallback = []) {
  if (typeof value !== "string" || !value.trim()) {
    return [...fallback];
  }

  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function normalizePeriod(period) {
  const value = String(period).trim();
  const lower = value.toLowerCase();

  if (lower.endsWith("m")) {
    return lower.slice(0, -1);
  }

  if (lower.endsWith("h")) {
    return String(Number.parseInt(lower.slice(0, -1), 10) * 60);
  }

  if (lower === "1d") {
    return "D";
  }

  return value.toUpperCase() === "W" ? "W" : value.toUpperCase() === "D" ? "D" : value;
}

function periodToMs(period) {
  const normalized = normalizePeriod(period);

  if (normalized === "W") {
    return 7 * 24 * 60 * 60 * 1000;
  }

  if (normalized === "D") {
    return 24 * 60 * 60 * 1000;
  }

  if (normalized.endsWith("D")) {
    return Number.parseInt(normalized.slice(0, -1), 10) * 24 * 60 * 60 * 1000;
  }

  return Number.parseInt(normalized, 10) * 60 * 1000;
}

function keyFor(symbol, period, signalType) {
  return `${symbol}::${period}::${signalType}`;
}

function shouldCreateSignal(existingSignal) {
  if (!existingSignal) {
    return Math.random() < 0.42;
  }

  return Math.random() < 0.2;
}

function nextSignal(symbol, period, signalType) {
  const normalizedPeriod = normalizePeriod(period);
  const key = keyFor(symbol, normalizedPeriod, signalType);
  const existingSignal = signalStore.get(key);
  const periodMs = periodToMs(normalizedPeriod);

  if (!shouldCreateSignal(existingSignal)) {
    if (existingSignal && Math.random() < 0.12) {
      signalStore.delete(key);
    }
    return signalStore.get(key) ?? null;
  }

  const maxLookbackBars = 16;
  const lookbackBars = Math.floor(Math.random() * maxLookbackBars);
  const signalAt = now() - lookbackBars * periodMs - Math.floor(Math.random() * 20_000);
  const nextValue = {
    sd: Math.random() < 0.5 ? 1 : -1,
    t: signalAt,
    read: Math.random() < 0.7 ? false : existingSignal?.read ?? false,
  };

  signalStore.set(key, nextValue);
  return nextValue;
}

function buildSignalsResponse(body) {
  const symbols = splitCsv(body.symbols, ["BTCUSDT"]).map((symbol) => symbol.toUpperCase());
  const periods = splitCsv(body.periods, DEFAULT_PERIODS).map(normalizePeriod);
  const signalTypes = splitCsv(body.signalTypes, DEFAULT_SIGNAL_TYPES);
  const page = Number(body.page || 1);
  const pageSize = Math.min(Math.max(Number(body.pageSize || 100), 1), 100);

  const data = [];

  for (const symbol of symbols) {
    for (const period of periods) {
      const signals = {};
      for (const signalType of signalTypes) {
        const signal = nextSignal(symbol, period, signalType);
        if (signal) {
          signals[signalType] = signal;
        }
      }

      const timestamps = Object.values(signals).map((signal) => signal.t);
      data.push({
        symbol,
        period,
        t: timestamps.length > 0 ? Math.max(...timestamps) : now(),
        signals,
      });
    }
  }

  const offset = (page - 1) * pageSize;
  return {
    total: data.length,
    page,
    pageSize: pageSize,
    data: data.slice(offset, offset + pageSize),
  };
}

function updateReadStatus(body) {
  const symbol = String(body.symbol || "").toUpperCase();
  const period = normalizePeriod(body.period || "");
  const signalType = String(body.signalType || "");
  const read = Boolean(body.read);
  const key = keyFor(symbol, period, signalType);
  const existingSignal = signalStore.get(key);

  if (!existingSignal) {
    return false;
  }

  signalStore.set(key, {
    ...existingSignal,
    read,
  });
  return true;
}

function deleteSignal(body) {
  const symbol = String(body.symbol || "").toUpperCase();
  const period = normalizePeriod(body.period || "");
  const signalType = String(body.signalType || "");
  return signalStore.delete(keyFor(symbol, period, signalType));
}

function writeJson(response, statusCode, payload) {
  response.writeHead(statusCode, {
    "Content-Type": "application/json; charset=utf-8",
  });
  response.end(JSON.stringify(payload));
}

function unauthorized(response) {
  writeJson(response, 401, {
    error: "UNAUTHORIZED",
    message: "x-api-key is missing or invalid.",
  });
}

async function readJson(request) {
  let raw = "";

  for await (const chunk of request) {
    raw += chunk;
  }

  if (!raw.trim()) {
    return {};
  }

  return JSON.parse(raw);
}

const server = http.createServer(async (request, response) => {
  const url = new URL(request.url || "/", `http://${request.headers.host || "localhost"}`);

  if (request.method === "GET" && url.pathname === "/health") {
    return writeJson(response, 200, {
      ok: true,
      service: "watch-tower-mock-alert-server",
      time: now(),
    });
  }

  if (request.method !== "POST") {
    return writeJson(response, 404, {
      error: "NOT_FOUND",
      message: "Only the documented mock endpoints are available.",
    });
  }

  if (request.headers["x-api-key"] !== expectedApiKey) {
    return unauthorized(response);
  }

  let body;
  try {
    body = await readJson(request);
  } catch (error) {
    return writeJson(response, 400, {
      error: "INVALID_JSON",
      message: error instanceof Error ? error.message : "Request body is not valid JSON.",
    });
  }

  if (logRequests) {
    console.log(`[mock-alert-server] ${request.method} ${url.pathname}`, body);
  }

  if (url.pathname === "/api/open/watch-list/symbol-signals") {
    return writeJson(response, 200, buildSignalsResponse(body));
  }

  if (url.pathname === "/api/open/watch-list/symbol-alert/read-status") {
    return writeJson(response, 200, updateReadStatus(body));
  }

  if (url.pathname === "/api/open/watch-list/symbol-alert/delete-signal") {
    return writeJson(response, 200, deleteSignal(body));
  }

  return writeJson(response, 404, {
    error: "NOT_FOUND",
    message: "Unknown mock endpoint.",
  });
});

server.listen(port, () => {
  console.log(
    `[mock-alert-server] listening on http://127.0.0.1:${port} with x-api-key=${expectedApiKey}`,
  );
  console.log(
    "[mock-alert-server] endpoints: POST /api/open/watch-list/symbol-signals, /api/open/watch-list/symbol-alert/read-status, /api/open/watch-list/symbol-alert/delete-signal",
  );
});
