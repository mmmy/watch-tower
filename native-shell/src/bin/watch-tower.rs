use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Local, TimeZone};
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use signal_desk_native::api_client::{ApiClient, SignalListQuery};
use signal_desk_native::runtime::{AppConfig, WatchGroup};
use signal_desk_native::signal_time::{elapsed_candles, elapsed_label, ElapsedCandles};

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PAGE_SIZE: u32 = 100;

#[derive(Parser)]
#[command(name = "watch-tower")]
#[command(about = "Watch Tower command line utilities")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Signals(SignalsCommand),
}

#[derive(Parser)]
struct SignalsCommand {
    #[command(subcommand)]
    command: SignalsSubcommand,
}

#[derive(Subcommand)]
enum SignalsSubcommand {
    List(ListArgs),
}

#[derive(Args)]
struct ListArgs {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    symbol: Option<String>,
    #[arg(long)]
    periods: Option<String>,
    #[arg(long = "signal-types")]
    signal_types: Option<String>,
    #[arg(long)]
    base_url: Option<String>,
    #[arg(long)]
    api_key: Option<String>,
    #[arg(long)]
    raw: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SuccessOutput {
    ok: bool,
    meta: OutputMeta,
    data: Vec<SignalOutput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputMeta {
    requested_at: i64,
    count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SignalOutput {
    symbol: String,
    period: String,
    signal_type: String,
    side: i8,
    trigger_time: i64,
    trigger_time_iso: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    period_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    elapsed_ms: Option<i64>,
    elapsed_label: Option<String>,
    elapsed_candles: Option<i64>,
}

#[derive(Serialize)]
struct ErrorOutput {
    ok: bool,
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

struct ListContext {
    base_url: String,
    api_key: String,
    groups: Vec<QueryGroup>,
    page_size: u32,
}

struct QueryGroup {
    symbol: String,
    periods: Vec<String>,
    signal_types: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(output) => print_json(&output, 0),
        Err(error) => {
            let output = ErrorOutput {
                ok: false,
                error: ErrorBody {
                    code: error.code,
                    message: error.message,
                },
            };
            print_json(&output, 1);
        }
    }
}

fn run(cli: Cli) -> Result<SuccessOutput, CliError> {
    match cli.command {
        Command::Signals(command) => match command.command {
            SignalsSubcommand::List(args) => list_signals(args),
        },
    }
}

fn list_signals(args: ListArgs) -> Result<SuccessOutput, CliError> {
    let requested_at = now_ms();
    let raw = args.raw;
    let context = build_list_context(args)?;
    let client = ApiClient::new(&context.base_url, &context.api_key)
        .map_err(|message| CliError::new("client_init_failed", message))?;

    let mut signals = Vec::new();

    for group in context.groups {
        let mut group_signals = seed_outputs(&group, requested_at, raw);
        let mut index_by_key = HashMap::new();
        for (index, signal) in group_signals.iter().enumerate() {
            index_by_key.insert((signal.period.clone(), signal.signal_type.clone()), index);
        }

        let query = SignalListQuery {
            symbols: group.symbol.clone(),
            periods: group.periods.join(","),
            signal_types: group.signal_types.join(","),
            page: DEFAULT_PAGE,
            page_size: context.page_size,
        };
        let response = client
            .fetch_signal_list(&query)
            .map_err(|message| CliError::new("request_failed", message))?;

        for item in response.data {
            for (signal_type, detail) in item.signals {
                let key = (item.period.clone(), signal_type.clone());
                let output = match index_by_key.get(&key).copied() {
                    Some(index) => &mut group_signals[index],
                    None => {
                        group_signals.push(empty_output(
                            item.symbol.clone(),
                            item.period.clone(),
                            signal_type.clone(),
                            requested_at,
                            raw,
                        ));
                        let index = group_signals.len() - 1;
                        index_by_key.insert(key, index);
                        &mut group_signals[index]
                    }
                };

                output.symbol = item.symbol.clone();
                output.side = if detail.sd >= 0 { 1 } else { -1 };
                output.trigger_time = detail.t;
                output.read = raw.then_some(detail.read);
                apply_elapsed(output, requested_at, raw);
            }
        }

        signals.extend(group_signals);
    }

    Ok(SuccessOutput {
        ok: true,
        meta: OutputMeta {
            requested_at,
            count: signals.len(),
        },
        data: signals,
    })
}

fn build_list_context(args: ListArgs) -> Result<ListContext, CliError> {
    if let Some(config_path) = args.config {
        let content = fs::read_to_string(&config_path).map_err(|err| {
            CliError::new(
                "config_read_failed",
                format!("failed to read {}: {}", config_path.display(), err),
            )
        })?;
        let config: AppConfig = serde_yaml::from_str(&content)
            .map_err(|err| CliError::new("config_parse_failed", err.to_string()))?;
        return Ok(context_from_config(config));
    }

    let symbol = required_arg(args.symbol, "symbol")?;
    let periods = parse_csv(required_arg(args.periods, "periods")?);
    let signal_types = parse_csv(required_arg(args.signal_types, "signal-types")?);
    if periods.is_empty() {
        return Err(CliError::new("invalid_args", "--periods must not be empty"));
    }
    if signal_types.is_empty() {
        return Err(CliError::new(
            "invalid_args",
            "--signal-types must not be empty",
        ));
    }

    let base_url = args
        .base_url
        .or_else(|| std::env::var("WATCH_TOWER_BASE_URL").ok())
        .ok_or_else(|| CliError::new("invalid_args", "--base-url is required without --config"))?;
    let api_key = args
        .api_key
        .or_else(|| std::env::var("WATCH_TOWER_API_KEY").ok())
        .ok_or_else(|| CliError::new("invalid_args", "--api-key is required without --config"))?;

    Ok(ListContext {
        base_url,
        api_key,
        groups: vec![QueryGroup {
            symbol,
            periods,
            signal_types,
        }],
        page_size: DEFAULT_PAGE_SIZE,
    })
}

fn context_from_config(config: AppConfig) -> ListContext {
    ListContext {
        base_url: config.api.base_url,
        api_key: config.api.api_key,
        groups: config
            .groups
            .into_iter()
            .filter(|group| group.enabled)
            .map(query_group_from_watch_group)
            .collect(),
        page_size: config.poll.page_size.max(1),
    }
}

fn query_group_from_watch_group(group: WatchGroup) -> QueryGroup {
    QueryGroup {
        symbol: group.symbol,
        periods: group.periods,
        signal_types: group.signal_types,
    }
}

fn required_arg(value: Option<String>, name: &str) -> Result<String, CliError> {
    value.ok_or_else(|| CliError::new("invalid_args", format!("--{name} is required")))
}

fn parse_csv(value: String) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn seed_outputs(group: &QueryGroup, now_ms: i64, raw: bool) -> Vec<SignalOutput> {
    let mut outputs = Vec::new();
    for signal_type in &group.signal_types {
        for period in &group.periods {
            outputs.push(empty_output(
                group.symbol.clone(),
                period.clone(),
                signal_type.clone(),
                now_ms,
                raw,
            ));
        }
    }
    outputs
}

fn empty_output(
    symbol: String,
    period: String,
    signal_type: String,
    now_ms: i64,
    raw: bool,
) -> SignalOutput {
    let mut output = SignalOutput {
        symbol,
        period,
        signal_type,
        side: 1,
        trigger_time: 0,
        trigger_time_iso: None,
        read: raw.then_some(false),
        period_ms: None,
        elapsed_ms: None,
        elapsed_label: None,
        elapsed_candles: None,
    };
    apply_elapsed(&mut output, now_ms, raw);
    output
}

fn apply_elapsed(output: &mut SignalOutput, now_ms: i64, raw: bool) {
    let ElapsedCandles {
        period_ms,
        elapsed_ms,
        elapsed_candles,
    } = elapsed_candles(&output.period, output.trigger_time, now_ms);
    output.period_ms = raw.then_some(period_ms).flatten();
    output.elapsed_ms = raw.then_some(elapsed_ms).flatten();
    output.elapsed_label = elapsed_ms.and_then(elapsed_label);
    output.elapsed_candles = elapsed_candles;
    output.trigger_time_iso = trigger_time_iso(output.trigger_time);
}

fn trigger_time_iso(trigger_time: i64) -> Option<String> {
    if trigger_time <= 0 {
        return None;
    }

    Local
        .timestamp_millis_opt(trigger_time)
        .single()
        .map(|dt| dt.to_rfc3339())
}

fn print_json<T: Serialize>(output: &T, exit_code: i32) {
    match serde_json::to_string(output) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("failed to serialize output: {err}");
            std::process::exit(1);
        }
    }

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

struct CliError {
    code: String,
    message: String,
}

impl CliError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
