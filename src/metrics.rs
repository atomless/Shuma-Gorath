// src/metrics.rs
// Prometheus-compatible metrics for WASM Bot Trap
// Stores counters in KV store and exports in Prometheus text format

use spin_sdk::key_value::Store;

const METRICS_PREFIX: &str = "metrics:";

/// Metric types we track
#[derive(Debug, Clone, Copy)]
pub enum MetricName {
    RequestsTotal,
    BansTotal,
    BlocksTotal,
    ChallengesTotal,
    WhitelistedTotal,
    TestModeActions,
    MazeHits,
}

impl MetricName {
    fn as_str(&self) -> &'static str {
        match self {
            MetricName::RequestsTotal => "requests_total",
            MetricName::BansTotal => "bans_total",
            MetricName::BlocksTotal => "blocks_total",
            MetricName::ChallengesTotal => "challenges_total",
            MetricName::WhitelistedTotal => "whitelisted_total",
            MetricName::TestModeActions => "test_mode_actions_total",
            MetricName::MazeHits => "maze_hits_total",
        }
    }
}

/// Increment a counter metric, optionally with a label
pub fn increment(store: &Store, metric: MetricName, label: Option<&str>) {
    let key = match label {
        Some(l) => format!("{}{}:{}", METRICS_PREFIX, metric.as_str(), l),
        None => format!("{}{}", METRICS_PREFIX, metric.as_str()),
    };
    
    let current: u64 = store.get(&key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    
    let _ = store.set(&key, (current + 1).to_string().as_bytes());
}

/// Get current value of a counter
fn get_counter(store: &Store, key: &str) -> u64 {
    store.get(key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Count active bans (gauge)
fn count_active_bans(store: &Store) -> u64 {
    let mut count = 0;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if let Ok(keys) = store.get_keys() {
        for key in keys {
            if key.starts_with("ban:default:") {
                if let Ok(Some(val)) = store.get(&key) {
                    if let Ok(entry) = serde_json::from_slice::<serde_json::Value>(&val) {
                        if let Some(expires) = entry.get("expires").and_then(|v| v.as_u64()) {
                            if expires > now {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

/// Generate Prometheus-format metrics output
pub fn render_metrics(store: &Store) -> String {
    let mut output = String::new();
    
    // Header
    output.push_str("# WASM Bot Trap Metrics\n");
    output.push_str("# TYPE bot_trap_requests_total counter\n");
    
    // Requests total
    let requests = get_counter(store, &format!("{}requests_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_requests_total {}\n", requests));
    
    // Bans by reason
    output.push_str("\n# TYPE bot_trap_bans_total counter\n");
    output.push_str("# HELP bot_trap_bans_total Total number of IP bans by reason\n");
    for reason in &["honeypot", "rate_limit", "browser", "admin"] {
        let key = format!("{}bans_total:{}", METRICS_PREFIX, reason);
        let count = get_counter(store, &key);
        output.push_str(&format!("bot_trap_bans_total{{reason=\"{}\"}} {}\n", reason, count));
    }
    
    // Blocks total
    output.push_str("\n# TYPE bot_trap_blocks_total counter\n");
    let blocks = get_counter(store, &format!("{}blocks_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_blocks_total {}\n", blocks));
    
    // Challenges total
    output.push_str("\n# TYPE bot_trap_challenges_total counter\n");
    let challenges = get_counter(store, &format!("{}challenges_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_challenges_total {}\n", challenges));
    
    // Whitelisted total
    output.push_str("\n# TYPE bot_trap_whitelisted_total counter\n");
    let whitelisted = get_counter(store, &format!("{}whitelisted_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_whitelisted_total {}\n", whitelisted));
    
    // Test mode actions
    output.push_str("\n# TYPE bot_trap_test_mode_actions_total counter\n");
    let test_mode = get_counter(store, &format!("{}test_mode_actions_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_test_mode_actions_total {}\n", test_mode));
    
    // Maze hits
    output.push_str("\n# TYPE bot_trap_maze_hits_total counter\n");
    output.push_str("# HELP bot_trap_maze_hits_total Total hits on link maze honeypot pages\n");
    let maze_hits = get_counter(store, &format!("{}maze_hits_total", METRICS_PREFIX));
    output.push_str(&format!("bot_trap_maze_hits_total {}\n", maze_hits));
    
    // Active bans (gauge)
    output.push_str("\n# TYPE bot_trap_active_bans gauge\n");
    output.push_str("# HELP bot_trap_active_bans Current number of active (non-expired) bans\n");
    let active_bans = count_active_bans(store);
    output.push_str(&format!("bot_trap_active_bans {}\n", active_bans));
    
    // Test mode enabled (gauge, 0 or 1)
    output.push_str("\n# TYPE bot_trap_test_mode_enabled gauge\n");
    let cfg = crate::config::Config::load(store, "default");
    let test_mode_enabled = if cfg.test_mode { 1 } else { 0 };
    output.push_str(&format!("bot_trap_test_mode_enabled {}\n", test_mode_enabled));
    
    output
}

/// Handle GET /metrics endpoint
pub fn handle_metrics(store: &Store) -> spin_sdk::http::Response {
    let body = render_metrics(store);
    spin_sdk::http::Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .body(body)
        .build()
}
