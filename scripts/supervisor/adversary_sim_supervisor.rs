use std::env;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:3000";
const DEFAULT_INTERVAL_MS: u64 = 1_000;
const DEFAULT_OFF_INTERVAL_MS: u64 = 3_000;
const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 1_000;
const DEFAULT_IO_TIMEOUT_MS: u64 = 2_000;
const DEFAULT_MAX_FAILURES: u32 = 8;
const BEAT_PATH: &str = "/internal/adversary-sim/beat";

#[derive(Clone, Debug)]
struct Config {
    host: String,
    port: u16,
    base_url: String,
    api_key: String,
    forwarded_secret: Option<String>,
    interval_ms: u64,
    off_interval_ms: u64,
    connect_timeout_ms: u64,
    io_timeout_ms: u64,
    max_failures: u32,
    exit_when_off: bool,
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    body: String,
}

fn parse_bool(raw: &str, default: bool) -> bool {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
}

fn parse_u64_env(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn parse_u32_env(name: &str, default: u32) -> u32 {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(default)
}

fn parse_base_url(base_url: &str) -> Result<(String, u16), String> {
    let without_scheme = base_url
        .trim()
        .strip_prefix("http://")
        .ok_or_else(|| "base URL must use http://".to_string())?;
    let host_port = without_scheme.split('/').next().unwrap_or("").trim();
    if host_port.is_empty() {
        return Err("base URL host is empty".to_string());
    }
    if let Some((host, port_raw)) = host_port.rsplit_once(':') {
        let port = port_raw
            .trim()
            .parse::<u16>()
            .map_err(|_| "invalid base URL port".to_string())?;
        let host = host.trim().to_string();
        if host.is_empty() {
            return Err("base URL host is empty".to_string());
        }
        return Ok((host, port));
    }
    Ok((host_port.to_string(), 80))
}

fn parse_args() -> Result<Config, String> {
    let mut base_url = env::var("SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let mut exit_when_off = env::var("SHUMA_ADVERSARY_SIM_SUPERVISOR_EXIT_WHEN_OFF")
        .ok()
        .map(|value| parse_bool(value.as_str(), false))
        .unwrap_or(false);

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--base-url" => {
                let Some(value) = args.next() else {
                    return Err("--base-url requires a value".to_string());
                };
                base_url = value;
            }
            "--exit-when-off" => {
                exit_when_off = true;
            }
            "--watch" => {
                exit_when_off = false;
            }
            "--help" | "-h" => {
                println!(
                    "adversary_sim_supervisor [--base-url http://127.0.0.1:3000] [--watch|--exit-when-off]"
                );
                std::process::exit(0);
            }
            unknown => {
                return Err(format!("unknown argument: {unknown}"));
            }
        }
    }

    let api_key = env::var("SHUMA_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "SHUMA_API_KEY is required".to_string())?;
    let forwarded_secret = env::var("SHUMA_FORWARDED_IP_SECRET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let interval_ms = parse_u64_env("SHUMA_ADVERSARY_SIM_SUPERVISOR_INTERVAL_MS", DEFAULT_INTERVAL_MS).max(100);
    let off_interval_ms =
        parse_u64_env("SHUMA_ADVERSARY_SIM_SUPERVISOR_OFF_INTERVAL_MS", DEFAULT_OFF_INTERVAL_MS).max(250);
    let connect_timeout_ms = parse_u64_env(
        "SHUMA_ADVERSARY_SIM_SUPERVISOR_CONNECT_TIMEOUT_MS",
        DEFAULT_CONNECT_TIMEOUT_MS,
    )
    .max(100);
    let io_timeout_ms =
        parse_u64_env("SHUMA_ADVERSARY_SIM_SUPERVISOR_IO_TIMEOUT_MS", DEFAULT_IO_TIMEOUT_MS).max(250);
    let max_failures =
        parse_u32_env("SHUMA_ADVERSARY_SIM_SUPERVISOR_MAX_FAILURES", DEFAULT_MAX_FAILURES).max(1);
    let (host, port) = parse_base_url(base_url.as_str())?;

    Ok(Config {
        host,
        port,
        base_url,
        api_key,
        forwarded_secret,
        interval_ms,
        off_interval_ms,
        connect_timeout_ms,
        io_timeout_ms,
        max_failures,
        exit_when_off,
    })
}

fn request_beat(config: &Config) -> Result<HttpResponse, String> {
    let address_text = format!("{}:{}", config.host, config.port);
    let address = address_text
        .to_socket_addrs()
        .map_err(|err| format!("resolve failed: {err}"))?
        .next()
        .ok_or_else(|| "resolve failed: no address".to_string())?;
    let mut stream = TcpStream::connect_timeout(
        &address,
        Duration::from_millis(config.connect_timeout_ms),
    )
    .map_err(|err| format!("connect failed: {err}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(config.io_timeout_ms)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(config.io_timeout_ms)));

    let mut request = format!(
        "POST {BEAT_PATH} HTTP/1.1\r\nHost: {}:{}\r\nAuthorization: Bearer {}\r\nX-Forwarded-For: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n",
        config.host, config.port, config.api_key
    );
    if let Some(secret) = config.forwarded_secret.as_ref() {
        request.push_str(format!("X-Shuma-Forwarded-Secret: {secret}\r\n").as_str());
    }
    request.push_str("\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write failed: {err}"))?;
    stream.flush().map_err(|err| format!("flush failed: {err}"))?;

    let mut buffer = Vec::new();
    stream
        .read_to_end(&mut buffer)
        .map_err(|err| format!("read failed: {err}"))?;
    let raw = String::from_utf8_lossy(buffer.as_slice()).to_string();
    let mut sections = raw.splitn(2, "\r\n\r\n");
    let head = sections.next().unwrap_or("");
    let body = sections.next().unwrap_or("").to_string();
    let status = head
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|raw_code| raw_code.parse::<u16>().ok())
        .ok_or_else(|| "invalid HTTP response status".to_string())?;

    Ok(HttpResponse { status, body })
}

fn json_bool(body: &str, key: &str) -> Option<bool> {
    let compact: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.contains(format!("\"{key}\":true").as_str()) {
        return Some(true);
    }
    if compact.contains(format!("\"{key}\":false").as_str()) {
        return Some(false);
    }
    None
}

fn json_u64(body: &str, key: &str) -> Option<u64> {
    let compact: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
    let needle = format!("\"{key}\":");
    let start = compact.find(needle.as_str())?;
    let mut digits = String::new();
    for ch in compact[start + needle.len()..].chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        break;
    }
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

fn main() {
    let config = match parse_args() {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("[adversary-sim-supervisor] invalid configuration: {err}");
            std::process::exit(2);
        }
    };

    eprintln!(
        "[adversary-sim-supervisor] starting base_url={} mode={} interval_ms={} off_interval_ms={}",
        config.base_url,
        if config.exit_when_off { "exit-when-off" } else { "watch" },
        config.interval_ms,
        config.off_interval_ms
    );

    let mut consecutive_failures = 0u32;
    loop {
        match request_beat(&config) {
            Ok(response) => {
                consecutive_failures = 0;
                if response.status == 200 {
                    let executed_ticks = json_u64(response.body.as_str(), "executed_ticks").unwrap_or(0);
                    let generated_requests =
                        json_u64(response.body.as_str(), "generated_requests").unwrap_or(0);
                    let failed_requests = json_u64(response.body.as_str(), "failed_requests").unwrap_or(0);
                    let generation_active =
                        json_bool(response.body.as_str(), "generation_active").unwrap_or(false);
                    let should_exit = json_bool(response.body.as_str(), "should_exit").unwrap_or(false);
                    if executed_ticks > 0 || generated_requests > 0 || failed_requests > 0 {
                        eprintln!(
                            "[adversary-sim-supervisor] executed_ticks={} generated_requests={} failed_requests={}",
                            executed_ticks, generated_requests, failed_requests
                        );
                    }
                    if should_exit && config.exit_when_off {
                        eprintln!(
                            "[adversary-sim-supervisor] exiting: generation is inactive and exit-when-off is set"
                        );
                        std::process::exit(0);
                    }
                    let sleep_ms = if generation_active {
                        config.interval_ms
                    } else {
                        config.off_interval_ms
                    };
                    thread::sleep(Duration::from_millis(sleep_ms));
                    continue;
                }
                if response.status == 404 {
                    eprintln!(
                        "[adversary-sim-supervisor] internal beat endpoint unavailable (status=404); exiting"
                    );
                    std::process::exit(0);
                }
                if response.status == 401 || response.status == 403 {
                    eprintln!(
                        "[adversary-sim-supervisor] authorization rejected status={} body={}",
                        response.status, response.body
                    );
                    std::process::exit(1);
                }
                consecutive_failures = consecutive_failures.saturating_add(1);
                eprintln!(
                    "[adversary-sim-supervisor] beat request failed status={} failures={}/{}",
                    response.status, consecutive_failures, config.max_failures
                );
            }
            Err(err) => {
                consecutive_failures = consecutive_failures.saturating_add(1);
                eprintln!(
                    "[adversary-sim-supervisor] beat transport error failures={}/{} err={}",
                    consecutive_failures, config.max_failures, err
                );
            }
        }

        if consecutive_failures >= config.max_failures {
            eprintln!(
                "[adversary-sim-supervisor] exiting after {} consecutive failures",
                consecutive_failures
            );
            std::process::exit(1);
        }
        thread::sleep(Duration::from_millis(config.interval_ms));
    }
}
