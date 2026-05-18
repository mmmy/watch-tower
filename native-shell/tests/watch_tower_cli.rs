use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use serde_json::Value;

static CONFIG_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn signals_list_outputs_enriched_json_for_configured_groups() {
    let server = TestServer::spawn();

    let config_path = write_temp_config(&server.base_url());
    let cli_path = std::env::var("CARGO_BIN_EXE_watch-tower").expect("watch-tower binary path");
    let output = Command::new(cli_path)
        .args(["signals", "list", "--config"])
        .arg(&config_path)
        .output()
        .expect("run watch-tower cli");

    server.join();
    fs::remove_file(&config_path).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout is json");
    assert_eq!(json["ok"], true);
    assert_eq!(json["meta"]["count"], 1);
    assert_eq!(json["data"][0]["symbol"], "BTCUSDT");
    assert_eq!(json["data"][0]["period"], "15");
    assert_eq!(json["data"][0]["signalType"], "divMacd");
    assert_eq!(json["data"][0]["side"], 1);
    assert_eq!(json["data"][0]["triggerTime"], 1_000_000);
    assert!(json["data"][0]["triggerTimeIso"].as_str().unwrap().contains('T'));
    assert!(json["data"][0]["elapsedLabel"].as_str().unwrap().ends_with("前"));
    assert!(json["data"][0]["elapsedCandles"].as_i64().unwrap() >= 0);
    assert!(json["data"][0].get("read").is_none());
    assert!(json["data"][0].get("periodMs").is_none());
    assert!(json["data"][0].get("elapsedMs").is_none());
}

#[test]
fn signals_list_raw_includes_debug_fields() {
    let server = TestServer::spawn();

    let config_path = write_temp_config(&server.base_url());
    let cli_path = std::env::var("CARGO_BIN_EXE_watch-tower").expect("watch-tower binary path");
    let output = Command::new(cli_path)
        .args(["signals", "list", "--raw", "--config"])
        .arg(&config_path)
        .output()
        .expect("run watch-tower cli");

    server.join();
    fs::remove_file(&config_path).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout is json");
    assert_eq!(json["data"][0]["read"], false);
    assert_eq!(json["data"][0]["periodMs"], 15 * 60 * 1000);
    assert!(json["data"][0]["elapsedMs"].as_i64().unwrap() >= 0);
}

struct TestServer {
    addr: SocketAddr,
    handle: thread::JoinHandle<()>,
}

impl TestServer {
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        listener
            .set_nonblocking(true)
            .expect("set listener nonblocking");
        let addr = listener.local_addr().expect("local addr");
        let handle = thread::spawn(move || {
            let started_at = SystemTime::now();
            let (mut stream, _) = loop {
                match listener.accept() {
                    Ok(connection) => break connection,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            started_at.elapsed().unwrap_or_default() < Duration::from_secs(5),
                            "timed out waiting for cli request"
                        );
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("accept request: {err}"),
                }
            };
            let mut request = [0_u8; 4096];
            let bytes = stream.read(&mut request).expect("read request");
            let request = String::from_utf8_lossy(&request[..bytes]);

            assert!(request.contains("POST /api/open/watch-list/symbol-signals HTTP/1.1"));
            assert!(request.contains("x-api-key: test-key"));
            assert!(request.contains("\"symbols\":\"BTCUSDT\""));
            assert!(request.contains("\"periods\":\"15\""));
            assert!(request.contains("\"signalTypes\":\"divMacd\""));

            let body = r#"{
                "data": [
                    {
                        "symbol": "BTCUSDT",
                        "period": "15",
                        "signals": {
                            "divMacd": {
                                "sd": 1,
                                "t": 1000000,
                                "read": false
                            }
                        }
                    }
                ]
            }"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        Self { addr, handle }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn join(self) {
        self.handle.join().expect("server thread");
    }
}

fn write_temp_config(base_url: &str) -> std::path::PathBuf {
    let unique = CONFIG_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("watch-tower-cli-{unique}.yaml"));
    let config = format!(
        r#"
api:
  base_url: "{base_url}"
  api_key: "test-key"
poll:
  page_size: 100
groups:
  - id: "group-1"
    name: "BTC Main"
    symbol: "BTCUSDT"
    periods: ["15"]
    signal_types: ["divMacd"]
    enabled: true
"#
    );
    fs::write(&path, config).expect("write temp config");
    path
}
