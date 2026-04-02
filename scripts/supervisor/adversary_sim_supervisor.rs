use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:3000";
const DEFAULT_INTERVAL_MS: u64 = 1_000;
const DEFAULT_OFF_INTERVAL_MS: u64 = 3_000;
const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 1_000;
const DEFAULT_IO_TIMEOUT_MS: u64 = 2_000;
const DEFAULT_MAX_FAILURES: u32 = 8;
const BEAT_PATH: &str = "/shuma/internal/adversary-sim/beat";
const WORKER_RESULT_PATH: &str = "/shuma/internal/adversary-sim/worker-result";
const DEFAULT_SCRAPLING_PYTHON_RELATIVE: &str = ".venv-scrapling/bin/python3";
const DEFAULT_SCRAPLING_CRAWLDIR_RELATIVE: &str = ".shuma/adversary-sim/scrapling-crawldir";
const DEFAULT_LLM_RUNTIME_PYTHON: &str = "python3";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExternalWorkerDispatch {
    Scrapling,
    LlmRuntime,
}

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
    repo_root: PathBuf,
    scrapling_python: PathBuf,
    llm_runtime_python: PathBuf,
    scrapling_scope_descriptor_path: Option<PathBuf>,
    scrapling_seed_inventory_path: Option<PathBuf>,
    scrapling_crawldir: PathBuf,
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    body: String,
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut cursor = 0usize;
    let mut decoded = Vec::new();

    loop {
        let remaining = &body[cursor..];
        let Some(size_line_end) = find_bytes(remaining, b"\r\n") else {
            return Err("invalid chunked response: missing size delimiter".to_string());
        };
        let size_line = std::str::from_utf8(&remaining[..size_line_end])
            .map_err(|_| "invalid chunked response: non-utf8 size line".to_string())?;
        let size_text = size_line.split(';').next().unwrap_or("").trim();
        let chunk_size = usize::from_str_radix(size_text, 16)
            .map_err(|_| "invalid chunked response: bad chunk size".to_string())?;
        cursor = cursor.saturating_add(size_line_end + 2);

        if chunk_size == 0 {
            return Ok(decoded);
        }

        if body.len() < cursor.saturating_add(chunk_size + 2) {
            return Err("invalid chunked response: truncated chunk".to_string());
        }
        decoded.extend_from_slice(&body[cursor..cursor + chunk_size]);
        cursor = cursor.saturating_add(chunk_size);
        if &body[cursor..cursor + 2] != b"\r\n" {
            return Err("invalid chunked response: missing chunk terminator".to_string());
        }
        cursor = cursor.saturating_add(2);
    }
}

fn parse_http_response(raw: &[u8]) -> Result<HttpResponse, String> {
    let Some(head_end) = find_bytes(raw, b"\r\n\r\n") else {
        return Err("invalid HTTP response: missing head/body delimiter".to_string());
    };
    let head = String::from_utf8_lossy(&raw[..head_end]).to_string();
    let body = &raw[head_end + 4..];
    let status = head
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|raw_code| raw_code.parse::<u16>().ok())
        .ok_or_else(|| "invalid HTTP response status".to_string())?;
    let chunked = head.lines().skip(1).any(|line| {
        let mut parts = line.splitn(2, ':');
        let header = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();
        header.eq_ignore_ascii_case("transfer-encoding")
            && value
                .split(',')
                .any(|token| token.trim().eq_ignore_ascii_case("chunked"))
    });
    let decoded_body = if chunked {
        decode_chunked_body(body)?
    } else {
        body.to_vec()
    };

    Ok(HttpResponse {
        status,
        body: String::from_utf8_lossy(decoded_body.as_slice()).to_string(),
    })
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

fn detect_repo_root() -> Result<PathBuf, String> {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(repo_root) = exe_path
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
        {
            return Ok(repo_root.to_path_buf());
        }
    }
    env::current_dir().map_err(|err| format!("cwd unavailable: {err}"))
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
    let repo_root = detect_repo_root()?;
    let scrapling_python = env::var("ADVERSARY_SIM_SCRAPLING_PYTHON")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(DEFAULT_SCRAPLING_PYTHON_RELATIVE));
    let llm_runtime_python = env::var("ADVERSARY_SIM_LLM_RUNTIME_PYTHON")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LLM_RUNTIME_PYTHON));
    let scrapling_scope_descriptor_path =
        env::var("ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH")
            .ok()
            .map(PathBuf::from);
    let scrapling_seed_inventory_path =
        env::var("ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH")
            .ok()
            .map(PathBuf::from);
    let scrapling_crawldir = env::var("ADVERSARY_SIM_SCRAPLING_CRAWLDIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(DEFAULT_SCRAPLING_CRAWLDIR_RELATIVE));

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
        repo_root,
        scrapling_python,
        llm_runtime_python,
        scrapling_scope_descriptor_path,
        scrapling_seed_inventory_path,
        scrapling_crawldir,
    })
}

fn request_post(config: &Config, path: &str, body: &str) -> Result<HttpResponse, String> {
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
        "POST {path} HTTP/1.1\r\nHost: {}:{}\r\nAuthorization: Bearer {}\r\nX-Forwarded-For: 127.0.0.1\r\nX-Forwarded-Proto: https\r\nX-Shuma-Internal-Supervisor: adversary-sim\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        config.host,
        config.port,
        config.api_key,
        body.as_bytes().len()
    );
    if let Some(secret) = config.forwarded_secret.as_ref() {
        request.push_str(format!("X-Shuma-Forwarded-Secret: {secret}\r\n").as_str());
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write failed: {err}"))?;
    stream.flush().map_err(|err| format!("flush failed: {err}"))?;

    let mut buffer = Vec::new();
    stream
        .read_to_end(&mut buffer)
        .map_err(|err| format!("read failed: {err}"))?;
    parse_http_response(buffer.as_slice())
}

fn request_beat(config: &Config) -> Result<HttpResponse, String> {
    request_post(config, BEAT_PATH, "")
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

fn json_string(body: &str, key: &str) -> Option<String> {
    let compact: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
    let needle = format!("\"{key}\":\"");
    let start = compact.find(needle.as_str())?;
    let mut escaped = false;
    let mut value = String::new();
    for ch in compact[start + needle.len()..].chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            return Some(value);
        }
        value.push(ch);
    }
    None
}

fn json_string_array(body: &str, key: &str) -> Vec<String> {
    let compact: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
    let needle = format!("\"{key}\":[");
    let Some(start) = compact.find(needle.as_str()) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut escaped = false;
    let mut in_string = false;
    let mut current = String::new();
    for ch in compact[start + needle.len()..].chars() {
        if !in_string {
            match ch {
                '"' => {
                    in_string = true;
                    current.clear();
                }
                ']' => break,
                _ => continue,
            }
            continue;
        }
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => {
                escaped = true;
            }
            '"' => {
                in_string = false;
                if !current.is_empty() {
                    values.push(current.clone());
                }
            }
            _ => current.push(ch),
        }
    }
    values
}

fn dispatch_mode(body: &str) -> Option<String> {
    json_string(body, "dispatch_mode")
}

fn worker_dispatches(body: &str) -> Vec<ExternalWorkerDispatch> {
    let compact: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
    let mut dispatches = Vec::new();
    if compact.contains("\"worker_plan\":{") {
        dispatches.push(ExternalWorkerDispatch::Scrapling);
    }
    if compact.contains("\"llm_fulfillment_plan\":{") {
        dispatches.push(ExternalWorkerDispatch::LlmRuntime);
    }
    dispatches
}

fn json_escape(raw: &str) -> String {
    raw.chars()
        .flat_map(|ch| match ch {
            '\\' => ['\\', '\\'].into_iter().collect::<Vec<_>>(),
            '"' => ['\\', '"'].into_iter().collect::<Vec<_>>(),
            '\n' => ['\\', 'n'].into_iter().collect::<Vec<_>>(),
            '\r' => ['\\', 'r'].into_iter().collect::<Vec<_>>(),
            '\t' => ['\\', 't'].into_iter().collect::<Vec<_>>(),
            other => [other].into_iter().collect::<Vec<_>>(),
        })
        .collect()
}

fn json_render_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value.as_str())))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn temp_file_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    env::temp_dir().join(format!("{prefix}-{nanos}-{}.json", std::process::id()))
}

fn scrapling_worker_script_path(config: &Config) -> PathBuf {
    config
        .repo_root
        .join("scripts")
        .join("supervisor")
        .join("scrapling_worker.py")
}

fn llm_runtime_worker_script_path(config: &Config) -> PathBuf {
    config
        .repo_root
        .join("scripts")
        .join("supervisor")
        .join("llm_runtime_worker.py")
}

fn build_scrapling_worker_failure_result(
    beat_body: &str,
    failure_class: &str,
    error: &str,
) -> String {
    let run_id = json_string(beat_body, "run_id").unwrap_or_default();
    let tick_id = json_string(beat_body, "tick_id").unwrap_or_default();
    let lane = json_string(beat_body, "lane").unwrap_or_else(|| "scrapling_traffic".to_string());
    let fulfillment_mode = json_string(beat_body, "fulfillment_mode").unwrap_or_default();
    let tick_started_at = json_u64(beat_body, "tick_started_at").unwrap_or(0);
    let worker_id = format!("adversary-sim-supervisor-{}", std::process::id());
    let tick_completed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    let category_targets = json_string_array(beat_body, "category_targets");
    format!(
        "{{\"schema_version\":\"adversary-sim-scrapling-worker-result.v1\",\"run_id\":\"{}\",\"tick_id\":\"{}\",\"lane\":\"{}\",\"fulfillment_mode\":\"{}\",\"category_targets\":{},\"worker_id\":\"{}\",\"tick_started_at\":{},\"tick_completed_at\":{},\"generated_requests\":0,\"failed_requests\":0,\"last_response_status\":null,\"failure_class\":\"{}\",\"error\":\"{}\",\"crawl_stats\":{{\"requests_count\":0,\"offsite_requests_count\":0,\"blocked_requests_count\":0,\"response_status_count\":{{}},\"response_bytes\":0}},\"scope_rejections\":{{}}}}",
        json_escape(run_id.as_str()),
        json_escape(tick_id.as_str()),
        json_escape(lane.as_str()),
        json_escape(fulfillment_mode.as_str()),
        json_render_string_array(category_targets.as_slice()),
        json_escape(worker_id.as_str()),
        tick_started_at,
        tick_completed_at,
        json_escape(failure_class),
        json_escape(error)
    )
}

fn scrapling_worker_timeout_ms(beat_body: &str) -> u64 {
    const SCRAPLING_TIMEOUT_HEADROOM_MS: u64 = 2_500;
    const SCRAPLING_TIMEOUT_FLOOR_MS: u64 = 8_000;
    json_u64(beat_body, "max_ms")
        .unwrap_or(2_000)
        .saturating_add(SCRAPLING_TIMEOUT_HEADROOM_MS)
        .max(SCRAPLING_TIMEOUT_FLOOR_MS)
}

fn build_llm_runtime_worker_failure_result(
    beat_body: &str,
    failure_class: &str,
    error: &str,
) -> String {
    let run_id = json_string(beat_body, "run_id").unwrap_or_default();
    let tick_id = json_string(beat_body, "tick_id").unwrap_or_default();
    let lane = json_string(beat_body, "lane").unwrap_or_else(|| "bot_red_team".to_string());
    let fulfillment_mode = json_string(beat_body, "fulfillment_mode").unwrap_or_default();
    let tick_started_at = json_u64(beat_body, "tick_started_at").unwrap_or(0);
    let worker_id = format!("adversary-sim-supervisor-{}", std::process::id());
    let tick_completed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    let terminal_failure = match failure_class {
        "timeout" => "deadline_exceeded",
        "cancelled" => "cancelled",
        _ => "worker_execution_failed",
    };
    format!(
        "{{\"schema_version\":\"adversary-sim-llm-runtime-result.v1\",\"run_id\":\"{}\",\"tick_id\":\"{}\",\"lane\":\"{}\",\"fulfillment_mode\":\"{}\",\"worker_id\":\"{}\",\"tick_started_at\":{},\"tick_completed_at\":{},\"backend_kind\":\"{}\",\"backend_state\":\"{}\",\"generation_source\":\"runtime_failure\",\"provider\":\"\",\"model_id\":\"\",\"fallback_reason\":null,\"category_targets\":[],\"generated_action_count\":0,\"executed_action_count\":0,\"failed_action_count\":0,\"last_response_status\":null,\"passed\":false,\"failure_class\":\"{}\",\"error\":\"{}\",\"terminal_failure\":\"{}\",\"action_receipts\":[]}}",
        json_escape(run_id.as_str()),
        json_escape(tick_id.as_str()),
        json_escape(lane.as_str()),
        json_escape(fulfillment_mode.as_str()),
        json_escape(worker_id.as_str()),
        tick_started_at,
        tick_completed_at,
        json_escape(
            json_string(beat_body, "backend_kind")
                .unwrap_or_else(|| "frontier_reference".to_string())
                .as_str()
        ),
        json_escape(
            json_string(beat_body, "backend_state")
                .unwrap_or_else(|| "unknown".to_string())
                .as_str()
        ),
        json_escape(failure_class),
        json_escape(error),
        json_escape(terminal_failure)
    )
}

fn run_python_worker<F>(
    python_binary: &PathBuf,
    worker_script: &PathBuf,
    beat_body: &str,
    beat_file_prefix: &str,
    result_file_prefix: &str,
    timeout_ms: u64,
    build_failure_result: fn(&str, &str, &str) -> String,
    configure_command: F,
) -> String
where
    F: FnOnce(&mut Command, &PathBuf, &PathBuf),
{
    let beat_file = temp_file_path(beat_file_prefix);
    let result_file = temp_file_path(result_file_prefix);
    if let Err(err) = fs::write(&beat_file, beat_body.as_bytes()) {
        return build_failure_result(
            beat_body,
            "transport",
            format!("write beat file failed: {err}").as_str(),
        );
    }
    if !worker_script.is_file() {
        let _ = fs::remove_file(&beat_file);
        return build_failure_result(
            beat_body,
            "transport",
            format!("missing worker script: {}", worker_script.display()).as_str(),
        );
    }

    let mut command = Command::new(python_binary);
    command.stdout(Stdio::null()).stderr(Stdio::piped());
    configure_command(&mut command, &beat_file, &result_file);
    let spawn_result = command.spawn();
    let mut child = match spawn_result {
        Ok(child) => child,
        Err(err) => {
            let _ = fs::remove_file(&beat_file);
            let _ = fs::remove_file(&result_file);
            return build_failure_result(
                beat_body,
                "transport",
                format!("spawn worker failed: {err}").as_str(),
            );
        }
    };

    let start = SystemTime::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let rendered = match fs::read_to_string(&result_file) {
                    Ok(body) => body,
                    Err(read_err) => {
                    let stderr_text = child
                        .stderr
                        .as_mut()
                        .map(|stderr| {
                            let mut captured = String::new();
                            let _ = stderr.read_to_string(&mut captured);
                            captured
                        })
                        .unwrap_or_default();
                    let detail = if stderr_text.trim().is_empty() {
                        format!("worker exited with status {:?}", status.code())
                    } else {
                        format!(
                            "worker exited with status {:?}; stderr={}",
                            status.code(),
                            stderr_text.trim()
                        )
                    };
                        let combined_detail = if status.success() {
                            format!("read worker result failed: {read_err}")
                        } else {
                            detail
                        };
                        build_failure_result(
                        beat_body,
                        "transport",
                        combined_detail.as_str(),
                    )
                    }
                };
                let _ = fs::remove_file(&beat_file);
                let _ = fs::remove_file(&result_file);
                return rendered;
            }
            Ok(None) => {
                let elapsed_ms = SystemTime::now()
                    .duration_since(start)
                    .unwrap_or_else(|_| Duration::from_secs(0))
                    .as_millis() as u64;
                if elapsed_ms >= timeout_ms {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = fs::remove_file(&beat_file);
                    let _ = fs::remove_file(&result_file);
                    return build_failure_result(
                        beat_body,
                        "timeout",
                        format!("worker exceeded timeout_ms={timeout_ms}").as_str(),
                    );
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&beat_file);
                let _ = fs::remove_file(&result_file);
                return build_failure_result(
                    beat_body,
                    "transport",
                    format!("worker wait failed: {err}").as_str(),
                );
            }
        }
    }
}

fn run_scrapling_worker(config: &Config, beat_body: &str) -> String {
    let worker_script = scrapling_worker_script_path(config);
    let timeout_ms = scrapling_worker_timeout_ms(beat_body);
    run_python_worker(
        &config.scrapling_python,
        &worker_script,
        beat_body,
        "shuma-scrapling-beat",
        "shuma-scrapling-result",
        timeout_ms,
        build_scrapling_worker_failure_result,
        |command, beat_file, result_file| {
            command
                .arg(&worker_script)
                .arg("--beat-response-file")
                .arg(beat_file)
                .arg("--result-output-file")
                .arg(result_file)
                .arg("--crawldir")
                .arg(&config.scrapling_crawldir);
            if let Some(path) = config.scrapling_scope_descriptor_path.as_ref() {
                command.arg("--scope-descriptor").arg(path);
            }
            if let Some(path) = config.scrapling_seed_inventory_path.as_ref() {
                command.arg("--seed-inventory").arg(path);
            }
        },
    )
}

fn run_llm_runtime_worker(config: &Config, beat_body: &str) -> String {
    let worker_script = llm_runtime_worker_script_path(config);
    let timeout_ms = json_u64(beat_body, "max_time_budget_seconds")
        .unwrap_or(120)
        .saturating_add(15)
        .saturating_mul(1_000);
    run_python_worker(
        &config.llm_runtime_python,
        &worker_script,
        beat_body,
        "shuma-llm-runtime-beat",
        "shuma-llm-runtime-result",
        timeout_ms,
        build_llm_runtime_worker_failure_result,
        |command, beat_file, result_file| {
            command
                .arg(&worker_script)
                .arg("--beat-response-file")
                .arg(beat_file)
                .arg("--result-output-file")
                .arg(result_file)
                .arg("--base-url")
                .arg(&config.base_url);
        },
    )
}

fn post_worker_result(config: &Config, body: &str) -> Result<HttpResponse, String> {
    request_post(config, WORKER_RESULT_PATH, body)
}

fn run_external_worker_dispatch(
    config: &Config,
    beat_body: &str,
    worker_dispatch: ExternalWorkerDispatch,
) -> Result<HttpResponse, String> {
    let worker_result_body = match worker_dispatch {
        ExternalWorkerDispatch::Scrapling => run_scrapling_worker(config, beat_body),
        ExternalWorkerDispatch::LlmRuntime => run_llm_runtime_worker(config, beat_body),
    };
    post_worker_result(config, worker_result_body.as_str())
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
                    let dispatch_mode =
                        dispatch_mode(response.body.as_str()).unwrap_or_else(|| "internal".to_string());
                    if executed_ticks > 0 || generated_requests > 0 || failed_requests > 0 {
                        eprintln!(
                            "[adversary-sim-supervisor] executed_ticks={} generated_requests={} failed_requests={}",
                            executed_ticks, generated_requests, failed_requests
                        );
                    }
                    let worker_dispatches = worker_dispatches(response.body.as_str());
                    if !worker_dispatches.is_empty() {
                        let mut handles = Vec::new();
                        for worker_dispatch in worker_dispatches {
                            let dispatch_config = config.clone();
                            let beat_body = response.body.clone();
                            handles.push(thread::spawn(move || {
                                run_external_worker_dispatch(
                                    &dispatch_config,
                                    beat_body.as_str(),
                                    worker_dispatch,
                                )
                            }));
                        }
                        for handle in handles {
                            let worker_outcome = handle
                                .join()
                                .unwrap_or_else(|_| Err("worker dispatch thread panicked".to_string()));
                            match worker_outcome {
                                Ok(worker_response) if worker_response.status == 200 => {}
                                Ok(worker_response) if worker_response.status == 409 => {
                                    eprintln!(
                                        "[adversary-sim-supervisor] worker result rejected as stale status={} body={}",
                                        worker_response.status, worker_response.body
                                    );
                                }
                                Ok(worker_response) => {
                                    consecutive_failures = consecutive_failures.saturating_add(1);
                                    eprintln!(
                                        "[adversary-sim-supervisor] worker result post failed status={} failures={}/{} body={}",
                                        worker_response.status,
                                        consecutive_failures,
                                        config.max_failures,
                                        worker_response.body
                                    );
                                }
                                Err(err) => {
                                    consecutive_failures = consecutive_failures.saturating_add(1);
                                    eprintln!(
                                        "[adversary-sim-supervisor] worker result transport error failures={}/{} err={}",
                                        consecutive_failures, config.max_failures, err
                                    );
                                }
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::{
        parse_http_response, scrapling_worker_timeout_ms, worker_dispatches,
        ExternalWorkerDispatch,
    };

    #[test]
    fn parse_http_response_keeps_plain_json_body() {
        let response = parse_http_response(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"ok\":true}",
        )
        .expect("plain response");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"ok\":true}");
    }

    #[test]
    fn parse_http_response_decodes_chunked_json_body() {
        let response = parse_http_response(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: application/json\r\n\r\n1\r\n{\r\n5\r\n\"ok\":\r\n4\r\ntrue\r\n1\r\n}\r\n0\r\n\r\n",
        )
        .expect("chunked response");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"ok\":true}");
    }

    #[test]
    fn worker_dispatches_detects_parallel_payload_plans() {
        assert_eq!(
            worker_dispatches(r#"{"worker_plan":{},"llm_fulfillment_plan":{}}"#),
            vec![
                ExternalWorkerDispatch::Scrapling,
                ExternalWorkerDispatch::LlmRuntime
            ]
        );
        assert_eq!(
            worker_dispatches(r#"{"worker_plan":{}}"#),
            vec![ExternalWorkerDispatch::Scrapling]
        );
        assert_eq!(worker_dispatches(r#"{"dispatch_mode":"internal"}"#), Vec::new());
    }

    #[test]
    fn scrapling_worker_timeout_preserves_result_materialization_headroom() {
        assert_eq!(scrapling_worker_timeout_ms(r#"{"max_ms":2500}"#), 8_000);
        assert_eq!(scrapling_worker_timeout_ms("{}"), 8_000);
        assert_eq!(scrapling_worker_timeout_ms(r#"{"max_ms":7000}"#), 9_500);
    }
}
