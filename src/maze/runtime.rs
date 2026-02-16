use serde::{Deserialize, Serialize};
use spin_sdk::http::{Request, Response};
use std::time::{SystemTime, UNIX_EPOCH};

use super::content::{
    capitalize, generate_link_text, generate_paragraph, generate_title, DEPARTMENTS, NOUNS,
};
use super::renders::{
    generate_polymorphic_maze_page, AdvancedMazeLink, AdvancedMazeRenderOptions, MazeStyleTier,
};
use super::rng::{generate_path_segment, SeededRng};
use super::state::MazeStateStore;
use super::token::{self, MazeTokenError, MazeTraversalToken};
use super::types::MazeConfig;

const BUDGET_GLOBAL_ACTIVE_KEY: &str = "maze:budget:active:global";
const BUDGET_BUCKET_ACTIVE_PREFIX: &str = "maze:budget:active:bucket";
const TOKEN_REPLAY_PREFIX: &str = "maze:token:seen";
const TOKEN_ISSUE_PREFIX: &str = "maze:token:issue";
const TOKEN_CHAIN_PREFIX: &str = "maze:token:chain";
const CHECKPOINT_PREFIX: &str = "maze:checkpoint";
const RISK_PREFIX: &str = "maze:risk";
const VIOLATION_PREFIX: &str = "maze:violation";
const MAX_RISK_SCORE: u8 = 10;
const HIGH_CONFIDENCE_ESCALATION_CHALLENGE_COUNT: u32 = 2;
const HIGH_CONFIDENCE_ESCALATION_BLOCK_COUNT: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViolationState {
    count: u32,
    expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeFallbackReason {
    TokenInvalid,
    TokenExpired,
    TokenReplay,
    TokenBindingMismatch,
    TokenDepthExceeded,
    BudgetExceeded,
    CheckpointMissing,
    MicroPowFailed,
}

impl MazeFallbackReason {
    pub(crate) fn detection_label(self) -> &'static str {
        match self {
            MazeFallbackReason::TokenInvalid => "maze_token_invalid",
            MazeFallbackReason::TokenExpired => "maze_token_expired",
            MazeFallbackReason::TokenReplay => "maze_token_replay",
            MazeFallbackReason::TokenBindingMismatch => "maze_token_binding_mismatch",
            MazeFallbackReason::TokenDepthExceeded => "maze_depth_exceeded",
            MazeFallbackReason::BudgetExceeded => "maze_budget_exceeded",
            MazeFallbackReason::CheckpointMissing => "maze_checkpoint_missing",
            MazeFallbackReason::MicroPowFailed => "maze_micro_pow_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeFallbackAction {
    Challenge,
    Block,
}

impl MazeFallbackAction {
    pub(crate) fn label(self) -> &'static str {
        match self {
            MazeFallbackAction::Challenge => "challenge",
            MazeFallbackAction::Block => "block",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MazeFallbackDecision {
    pub reason: MazeFallbackReason,
    pub action: MazeFallbackAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeRolloutPhase {
    Instrument,
    Advisory,
    Enforce,
}

impl MazeRolloutPhase {
    fn from_config(cfg: &crate::config::Config) -> Self {
        match cfg.maze_rollout_phase.as_str() {
            "instrument" => MazeRolloutPhase::Instrument,
            "advisory" => MazeRolloutPhase::Advisory,
            _ => MazeRolloutPhase::Enforce,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckpointState {
    last_ts_ms: u64,
    last_depth: u16,
    expires_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct LinkIssueCandidate {
    path: String,
    text: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MazeRenderResult {
    pub html: String,
    pub depth: u16,
    pub flow_id: String,
    pub variant_id: String,
    pub seed_provider: String,
    pub seed_version: u64,
    pub seed_metadata_only: bool,
    pub seed_source_count: usize,
    pub response_cap_exceeded: bool,
    pub bytes: usize,
    pub render_ms: u64,
    pub token_validated: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum MazeServeDecision {
    Serve(MazeRenderResult),
    Fallback(MazeFallbackDecision),
}

struct BudgetLease<'a, S: MazeStateStore> {
    store: &'a S,
    global_key: String,
    bucket_key: String,
    active: bool,
}

impl<'a, S: MazeStateStore> BudgetLease<'a, S> {
    fn release(&mut self) {
        if !self.active {
            return;
        }
        decrement_counter(self.store, self.global_key.as_str());
        decrement_counter(self.store, self.bucket_key.as_str());
        self.active = false;
    }
}

impl<S: MazeStateStore> Drop for BudgetLease<'_, S> {
    fn drop(&mut self) {
        self.release();
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn read_counter(store: &(impl MazeStateStore + ?Sized), key: &str) -> u32 {
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(0)
}

fn write_counter(store: &(impl MazeStateStore + ?Sized), key: &str, value: u32) {
    if let Err(err) = store.set(key, value.to_string().as_bytes()) {
        eprintln!("[maze] failed to persist counter key={} err={:?}", key, err);
    }
}

fn increment_counter(store: &(impl MazeStateStore + ?Sized), key: &str) -> u32 {
    let next = read_counter(store, key).saturating_add(1);
    write_counter(store, key, next);
    next
}

fn decrement_counter(store: &(impl MazeStateStore + ?Sized), key: &str) -> u32 {
    let current = read_counter(store, key);
    let next = current.saturating_sub(1);
    write_counter(store, key, next);
    next
}

fn risk_key(ip_bucket: &str) -> String {
    format!("{}:{}", RISK_PREFIX, ip_bucket)
}

pub(crate) fn current_behavior_score(store: &(impl MazeStateStore + ?Sized), ip: &str) -> u8 {
    let key = risk_key(crate::signals::ip_identity::bucket_ip(ip).as_str());
    read_counter(store, key.as_str()).min(MAX_RISK_SCORE as u32) as u8
}

pub(crate) fn increment_behavior_score(
    store: &(impl MazeStateStore + ?Sized),
    ip: &str,
    amount: u8,
) {
    if amount == 0 {
        return;
    }
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let key = risk_key(ip_bucket.as_str());
    let next = read_counter(store, key.as_str())
        .saturating_add(amount as u32)
        .min(MAX_RISK_SCORE as u32);
    write_counter(store, key.as_str(), next);
}

fn budget_bucket_key(ip_bucket: &str) -> String {
    format!("{}:{}", BUDGET_BUCKET_ACTIVE_PREFIX, ip_bucket)
}

fn try_acquire_budget<'a, S: MazeStateStore>(
    store: &'a S,
    cfg: &crate::config::Config,
    ip_bucket: &str,
) -> Option<BudgetLease<'a, S>> {
    let global = read_counter(store, BUDGET_GLOBAL_ACTIVE_KEY);
    let bucket_key = budget_bucket_key(ip_bucket);
    let bucket = read_counter(store, bucket_key.as_str());
    if global >= cfg.maze_max_concurrent_global || bucket >= cfg.maze_max_concurrent_per_ip_bucket {
        return None;
    }

    increment_counter(store, BUDGET_GLOBAL_ACTIVE_KEY);
    increment_counter(store, bucket_key.as_str());
    Some(BudgetLease {
        store,
        global_key: BUDGET_GLOBAL_ACTIVE_KEY.to_string(),
        bucket_key,
        active: true,
    })
}

fn token_replay_key(flow_id: &str, operation_id: &str) -> String {
    format!("{}:{}:{}", TOKEN_REPLAY_PREFIX, flow_id, operation_id)
}

fn token_issue_key(flow_id: &str, operation_id: &str) -> String {
    format!("{}:{}:{}", TOKEN_ISSUE_PREFIX, flow_id, operation_id)
}

fn token_chain_key(flow_id: &str, op_digest: &str) -> String {
    format!("{}:{}:{}", TOKEN_CHAIN_PREFIX, flow_id, op_digest)
}

fn checkpoint_key(flow_id: &str, ip_bucket: &str) -> String {
    format!("{}:{}:{}", CHECKPOINT_PREFIX, flow_id, ip_bucket)
}

fn violation_key(ip_bucket: &str) -> String {
    format!("{}:{}", VIOLATION_PREFIX, ip_bucket)
}

fn replay_seen(
    store: &(impl MazeStateStore + ?Sized),
    flow_id: &str,
    operation_id: &str,
    now: u64,
) -> bool {
    let key = token_replay_key(flow_id, operation_id);
    let seen_until = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok());
    matches!(seen_until, Some(until) if now <= until)
}

fn issue_seen(
    store: &(impl MazeStateStore + ?Sized),
    flow_id: &str,
    operation_id: &str,
    now: u64,
) -> bool {
    let key = token_issue_key(flow_id, operation_id);
    let seen_until = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok());
    matches!(seen_until, Some(until) if now <= until)
}

fn mark_replay_seen(
    store: &impl MazeStateStore,
    token: &MazeTraversalToken,
    replay_ttl: u64,
    now: u64,
) {
    let key = token_replay_key(token.flow_id.as_str(), token.operation_id.as_str());
    let seen_until = std::cmp::min(token.expires_at, now.saturating_add(replay_ttl));
    if let Err(err) = store.set(key.as_str(), seen_until.to_string().as_bytes()) {
        eprintln!(
            "[maze] failed to persist replay marker key={} err={:?}",
            key, err
        );
    }
    let op_digest = token::digest(format!("{}:{}", token.flow_id, token.operation_id).as_str());
    let chain_key = token_chain_key(token.flow_id.as_str(), op_digest.as_str());
    if let Err(err) = store.set(chain_key.as_str(), seen_until.to_string().as_bytes()) {
        eprintln!(
            "[maze] failed to persist chain marker key={} err={:?}",
            chain_key, err
        );
    }
}

fn mark_issue_seen(
    store: &impl MazeStateStore,
    token: &MazeTraversalToken,
    replay_ttl: u64,
    now: u64,
) {
    let key = token_issue_key(token.flow_id.as_str(), token.operation_id.as_str());
    let seen_until = std::cmp::min(token.expires_at, now.saturating_add(replay_ttl));
    if let Err(err) = store.set(key.as_str(), seen_until.to_string().as_bytes()) {
        eprintln!(
            "[maze] failed to persist issue marker key={} err={:?}",
            key, err
        );
    }
}

fn chain_marker_seen(
    store: &(impl MazeStateStore + ?Sized),
    flow_id: &str,
    prev_digest: &str,
    now: u64,
) -> bool {
    let key = token_chain_key(flow_id, prev_digest);
    let seen_until = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok());
    matches!(seen_until, Some(until) if now <= until)
}

fn is_high_confidence_violation(reason: MazeFallbackReason) -> bool {
    matches!(
        reason,
        MazeFallbackReason::TokenReplay
            | MazeFallbackReason::TokenBindingMismatch
            | MazeFallbackReason::CheckpointMissing
            | MazeFallbackReason::MicroPowFailed
    )
}

fn high_confidence_escalation_action(count: u32) -> Option<MazeFallbackAction> {
    if count >= HIGH_CONFIDENCE_ESCALATION_BLOCK_COUNT {
        Some(MazeFallbackAction::Block)
    } else if count >= HIGH_CONFIDENCE_ESCALATION_CHALLENGE_COUNT {
        Some(MazeFallbackAction::Challenge)
    } else {
        None
    }
}

fn fallback_action_for_reason(reason: MazeFallbackReason) -> MazeFallbackAction {
    match reason {
        MazeFallbackReason::BudgetExceeded
        | MazeFallbackReason::CheckpointMissing
        | MazeFallbackReason::MicroPowFailed
        | MazeFallbackReason::TokenExpired => MazeFallbackAction::Challenge,
        MazeFallbackReason::TokenInvalid
        | MazeFallbackReason::TokenReplay
        | MazeFallbackReason::TokenBindingMismatch
        | MazeFallbackReason::TokenDepthExceeded => MazeFallbackAction::Block,
    }
}

fn high_confidence_violation_count(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    ip_bucket: &str,
    reason: MazeFallbackReason,
    now_secs: u64,
) -> u32 {
    if !is_high_confidence_violation(reason) {
        return 0;
    }

    let key = violation_key(ip_bucket);
    let mut state = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<ViolationState>(raw.as_slice()).ok())
        .unwrap_or(ViolationState {
            count: 0,
            expires_at: 0,
        });
    if now_secs > state.expires_at {
        state.count = 0;
    }
    state.count = state.count.saturating_add(1).min(32);
    state.expires_at = now_secs.saturating_add(cfg.maze_replay_ttl_seconds.max(60));
    if let Ok(raw) = serde_json::to_vec(&state) {
        if let Err(err) = store.set(key.as_str(), raw.as_slice()) {
            eprintln!(
                "[maze] failed to persist violation key={} err={:?}",
                key, err
            );
        }
    }
    state.count
}

fn current_violation_count(
    store: &(impl MazeStateStore + ?Sized),
    ip_bucket: &str,
    now_secs: u64,
) -> u32 {
    let key = violation_key(ip_bucket);
    let state = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<ViolationState>(raw.as_slice()).ok());
    match state {
        Some(state) if now_secs <= state.expires_at => state.count,
        _ => 0,
    }
}

fn load_checkpoint_state(
    store: &impl MazeStateStore,
    flow_id: &str,
    ip_bucket: &str,
    now: u64,
) -> Option<CheckpointState> {
    let key = checkpoint_key(flow_id, ip_bucket);
    let raw = store.get(key.as_str()).ok().flatten()?;
    let state = serde_json::from_slice::<CheckpointState>(raw.as_slice()).ok()?;
    if now > state.expires_at {
        return None;
    }
    Some(state)
}

fn checkpoint_is_required(cfg: &crate::config::Config, depth: u16) -> bool {
    cfg.maze_client_expansion_enabled && depth as u64 > cfg.maze_checkpoint_every_nodes
}

fn checkpoint_missing(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    token: &MazeTraversalToken,
    ip_bucket: &str,
    now_ms: u64,
) -> bool {
    if !checkpoint_is_required(cfg, token.depth) {
        return false;
    }
    let Some(state) =
        load_checkpoint_state(store, token.flow_id.as_str(), ip_bucket, now_ms / 1000)
    else {
        return true;
    };
    let depth_delta = token.depth.saturating_sub(state.last_depth) as u64;
    let elapsed_ms = now_ms.saturating_sub(state.last_ts_ms);
    depth_delta > cfg.maze_step_ahead_max || elapsed_ms > cfg.maze_checkpoint_every_ms
}

fn should_enforce_violation(phase: MazeRolloutPhase, reason: MazeFallbackReason) -> bool {
    match phase {
        MazeRolloutPhase::Enforce => true,
        MazeRolloutPhase::Advisory => matches!(reason, MazeFallbackReason::BudgetExceeded),
        MazeRolloutPhase::Instrument => false,
    }
}

fn pow_difficulty_for_depth(cfg: &crate::config::Config, depth: u16) -> Option<u8> {
    if !cfg.maze_micro_pow_enabled || depth < cfg.maze_micro_pow_depth_start {
        return None;
    }
    let extra = depth.saturating_sub(cfg.maze_micro_pow_depth_start) as u8;
    Some(
        cfg.maze_micro_pow_base_difficulty
            .saturating_add(extra)
            .min(24),
    )
}

fn map_token_error(err: MazeTokenError) -> MazeFallbackReason {
    match err {
        MazeTokenError::Expired => MazeFallbackReason::TokenExpired,
        MazeTokenError::Missing | MazeTokenError::Malformed | MazeTokenError::InvalidVersion => {
            MazeFallbackReason::TokenInvalid
        }
        MazeTokenError::SignatureMismatch => MazeFallbackReason::TokenBindingMismatch,
    }
}

fn parse_existing_token(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    query: &str,
    path: &str,
    ip_bucket: &str,
    ua_bucket: &str,
    path_prefix: &str,
    now_secs: u64,
    now_ms_value: u64,
) -> Result<Option<(String, MazeTraversalToken)>, MazeFallbackReason> {
    let Some(raw_token) = crate::request_validation::query_param(query, "mt") else {
        return Ok(None);
    };
    let secret = token::secret_from_env();
    let parsed = token::verify(raw_token.as_str(), secret.as_str(), Some(now_secs))
        .map_err(map_token_error)?;
    if parsed.path_prefix != path_prefix
        || parsed.path_digest != token::digest(path)
        || parsed.ip_bucket != ip_bucket
        || parsed.ua_bucket != ua_bucket
    {
        return Err(MazeFallbackReason::TokenBindingMismatch);
    }
    if parsed.depth > cfg.maze_token_max_depth {
        return Err(MazeFallbackReason::TokenDepthExceeded);
    }
    if parsed.branch_budget == 0 || parsed.branch_budget > cfg.maze_token_branch_budget {
        return Err(MazeFallbackReason::TokenInvalid);
    }
    if parsed.depth > 1
        && !chain_marker_seen(
            store,
            parsed.flow_id.as_str(),
            parsed.prev_digest.as_str(),
            now_secs,
        )
    {
        return Err(MazeFallbackReason::TokenBindingMismatch);
    }
    if replay_seen(
        store,
        parsed.flow_id.as_str(),
        parsed.operation_id.as_str(),
        now_secs,
    ) {
        return Err(MazeFallbackReason::TokenReplay);
    }
    if checkpoint_missing(store, cfg, &parsed, ip_bucket, now_ms_value) {
        if parsed.depth > cfg.maze_no_js_fallback_max_depth {
            return Err(MazeFallbackReason::CheckpointMissing);
        }
    }
    if let Some(required_pow) = pow_difficulty_for_depth(cfg, parsed.depth) {
        let nonce = crate::request_validation::query_param(query, "mpn").unwrap_or_default();
        if !token::verify_micro_pow(raw_token.as_str(), nonce.as_str(), required_pow) {
            return Err(MazeFallbackReason::MicroPowFailed);
        }
    }

    mark_replay_seen(store, &parsed, cfg.maze_replay_ttl_seconds, now_secs);
    Ok(Some((raw_token, parsed)))
}

pub(crate) fn handle_checkpoint(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    req: &Request,
    ip: &str,
    user_agent: &str,
) -> Response {
    if *req.method() != spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let payload = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_POW_VERIFY_BYTES,
    ) {
        Ok(payload) => payload,
        Err(_) => return Response::new(400, "Invalid checkpoint payload"),
    };
    let raw_token = payload
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if raw_token.is_empty() {
        return Response::new(400, "Missing token");
    }

    let secret = token::secret_from_env();
    let now_secs = now_ms() / 1000;
    let parsed = match token::verify(raw_token.as_str(), secret.as_str(), Some(now_secs)) {
        Ok(parsed) => parsed,
        Err(_) => return Response::new(400, "Invalid checkpoint token"),
    };
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let ua_bucket = token::ua_bucket(user_agent);
    if parsed.ip_bucket != ip_bucket || parsed.ua_bucket != ua_bucket {
        return Response::new(403, "Checkpoint binding mismatch");
    }

    let now_ms_value = now_ms();
    let state = CheckpointState {
        last_ts_ms: now_ms_value,
        last_depth: parsed.depth,
        expires_at: now_secs.saturating_add(cfg.maze_replay_ttl_seconds),
    };
    let key = checkpoint_key(parsed.flow_id.as_str(), ip_bucket.as_str());
    if let Ok(raw) = serde_json::to_vec(&state) {
        if let Err(err) = store.set(key.as_str(), raw.as_slice()) {
            eprintln!(
                "[maze] failed to persist checkpoint key={} err={:?}",
                key, err
            );
        }
    }
    Response::new(204, "")
}

fn flow_entropy_nonce(
    existing: Option<&MazeTraversalToken>,
    path_prefix: &str,
    now_secs: u64,
    ip_bucket: &str,
    ua_bucket: &str,
) -> String {
    existing
        .map(|token| token.entropy_nonce.clone())
        .unwrap_or_else(|| token::flow_id_from(ip_bucket, ua_bucket, path_prefix, now_secs))
}

fn is_safe_maze_candidate_path(path: &str, path_prefix: &str) -> bool {
    path.starts_with(path_prefix)
        && path.len() <= 256
        && path
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.' | '~'))
}

fn sanitize_candidate_text(value: Option<String>, fallback: &str) -> String {
    let trimmed = value.unwrap_or_default().trim().to_string();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.chars().take(96).collect()
    }
}

fn worker_next_seed(seed: u32) -> u32 {
    let mut next = seed;
    next ^= next.wrapping_shl(13);
    next ^= next.wrapping_shr(7);
    next ^= next.wrapping_shl(17);
    next
}

fn worker_candidate_paths(
    mut seed: u32,
    hidden_count: usize,
    segment_len: usize,
    path_prefix: &str,
) -> Vec<String> {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let bounded_count = hidden_count.min(24);
    let bounded_segment_len = segment_len.clamp(8, 48);
    let mut paths = Vec::with_capacity(bounded_count);
    for _ in 0..bounded_count {
        let mut segment = String::with_capacity(bounded_segment_len);
        for _ in 0..bounded_segment_len {
            seed = worker_next_seed(seed);
            let idx = (seed as usize) % ALPHABET.len();
            segment.push(ALPHABET[idx] as char);
        }
        paths.push(format!("{path_prefix}{segment}"));
    }
    paths
}

fn render_budget_open_maze_page(path_prefix: &str, flow_id: &str) -> String {
    let next_path = format!("{}{}", path_prefix, token::digest(flow_id));
    let issue_path = super::issue_links_path();
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><meta name=\"robots\" content=\"noindex,nofollow,noarchive\"><title>Operational Index</title></head><body class=\"style-machine\"><div class=\"wrap style-machine\"><header><h1>Operational Index</h1><div class=\"crumb\">Portal &gt; Routing &gt; Index</div></header><div class=\"content\"><p class=\"description\">Routing channel active. Continue operational traversal.</p><div class=\"nav-grid\" id=\"maze-nav-grid\"><a href=\"{}\" class=\"nav-card\" data-link-kind=\"maze\"><h3>Continue stream</h3><p>Operational path transition.</p></a></div><script id=\"maze-bootstrap\" type=\"application/json\">{{\"flow_id\":\"{}\",\"depth\":0,\"checkpoint_token\":\"\",\"path_prefix\":\"{}\",\"entropy_nonce\":\"\",\"assets\":{{\"worker_url\":\"{}\"}},\"client_expansion\":{{\"enabled\":false,\"seed\":0,\"seed_sig\":\"\",\"hidden_count\":0,\"segment_len\":16,\"issue_path\":\"{}\"}}}}</script><script defer src=\"{}\"></script></div></div></body></html>",
        next_path,
        flow_id,
        path_prefix,
        super::assets::maze_worker_path(),
        issue_path,
        super::assets::maze_script_path(),
    )
}

pub(crate) fn handle_issue_links(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    req: &Request,
    ip: &str,
    user_agent: &str,
) -> Response {
    if *req.method() != spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let payload = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_POW_VERIFY_BYTES,
    ) {
        Ok(payload) => payload,
        Err(_) => return Response::new(400, "Invalid link issuance payload"),
    };
    let path_prefix = payload
        .get("path_prefix")
        .and_then(|v| v.as_str())
        .unwrap_or(super::path_prefix());
    if path_prefix != super::path_prefix() {
        return Response::new(400, "Invalid path prefix");
    }

    let candidates = payload
        .get("candidates")
        .and_then(|v| serde_json::from_value::<Vec<LinkIssueCandidate>>(v.clone()).ok())
        .unwrap_or_default();

    let now_secs = now_ms() / 1000;
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let ua_bucket = token::ua_bucket(user_agent);
    let parent_token_raw = payload
        .get("parent_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if parent_token_raw.is_empty() {
        return Response::new(400, "Missing parent token");
    }

    let secret = token::secret_from_env();
    let now_ms_value = now_ms();
    let parsed_parent =
        match token::verify(parent_token_raw.as_str(), secret.as_str(), Some(now_secs)) {
            Ok(parsed) => {
                if parsed.ip_bucket != ip_bucket
                    || parsed.ua_bucket != ua_bucket
                    || parsed.path_prefix != path_prefix
                {
                    return Response::new(403, "Link issuance binding mismatch");
                }
                if checkpoint_missing(store, cfg, &parsed, ip_bucket.as_str(), now_ms_value)
                    && parsed.depth > cfg.maze_no_js_fallback_max_depth
                {
                    return Response::new(403, "Checkpoint required before hidden issuance");
                }
                parsed
            }
            Err(_) => return Response::new(400, "Invalid parent token"),
        };

    if issue_seen(
        store,
        parsed_parent.flow_id.as_str(),
        parsed_parent.operation_id.as_str(),
        now_secs,
    ) {
        return Response::new(409, "Link issuance replay");
    }

    let payload_flow_id = payload
        .get("flow_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let payload_entropy_nonce = payload
        .get("entropy_nonce")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if payload_flow_id != parsed_parent.flow_id
        || payload_entropy_nonce != parsed_parent.entropy_nonce
    {
        return Response::new(403, "Link issuance flow mismatch");
    }

    let seed = payload.get("seed").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let hidden_count = payload
        .get("hidden_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let requested_hidden_count = payload
        .get("requested_hidden_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(hidden_count as u64) as usize;
    let segment_len = payload
        .get("segment_len")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let seed_signature = payload
        .get("seed_sig")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if hidden_count == 0 || segment_len == 0 {
        return Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body("{\"links\":[]}")
            .build();
    }
    if !token::verify_expansion_seed_signature(
        seed_signature,
        secret.as_str(),
        parsed_parent.flow_id.as_str(),
        path_prefix,
        parsed_parent.entropy_nonce.as_str(),
        parsed_parent.depth,
        seed as u64,
        hidden_count,
        segment_len,
    ) {
        return Response::new(403, "Invalid expansion seed signature");
    }

    let expected_paths = worker_candidate_paths(seed, hidden_count, segment_len, path_prefix);
    let hard_limit = cfg.maze_max_links.min(24).max(1) as usize;
    let branch_limit = parsed_parent.branch_budget.max(1) as usize;
    let requested_limit = requested_hidden_count.clamp(1, hidden_count);
    let limit = expected_paths
        .len()
        .min(requested_limit)
        .min(hard_limit)
        .min(branch_limit);

    let links = expected_paths
        .into_iter()
        .take(limit)
        .map(|path| {
            let from_client = candidates
                .iter()
                .find(|candidate| candidate.path == path)
                .filter(|candidate| is_safe_maze_candidate_path(candidate.path.as_str(), path_prefix));
            let child = token::issue_child_token(
                Some(&parsed_parent),
                path.as_str(),
                path_prefix,
                ip_bucket.as_str(),
                ua_bucket.as_str(),
                cfg.maze_token_ttl_seconds,
                cfg.maze_token_max_depth,
                parsed_parent.branch_budget.saturating_sub(1).max(1),
                parsed_parent.entropy_nonce.as_str(),
                parsed_parent.variant_id,
                now_secs,
            );
            let signed = token::sign(&child, secret.as_str());
            serde_json::json!({
                "href": format!("{}?mt={}", path, signed),
                "text": sanitize_candidate_text(from_client.and_then(|candidate| candidate.text.clone()), "Continue"),
                "description": sanitize_candidate_text(from_client.and_then(|candidate| candidate.description.clone()), "Operational stream."),
                "pow_difficulty": pow_difficulty_for_depth(cfg, child.depth)
            })
        })
        .collect::<Vec<_>>();

    mark_issue_seen(store, &parsed_parent, cfg.maze_replay_ttl_seconds, now_secs);

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "links": links
            })
            .to_string(),
        )
        .build()
}

fn make_breadcrumb(rng: &mut SeededRng) -> String {
    let dept = rng.pick(DEPARTMENTS);
    let noun = capitalize(rng.pick(NOUNS));
    format!("Portal > {} > {} Index", dept, noun)
}

pub(crate) fn serve(
    store: &impl MazeStateStore,
    cfg: &crate::config::Config,
    req: &Request,
    ip: &str,
    user_agent: &str,
    path: &str,
    botness_hint: Option<u8>,
) -> MazeServeDecision {
    let now_ms_value = now_ms();
    let now_secs = now_ms_value / 1000;
    let phase = MazeRolloutPhase::from_config(cfg);
    let query = req.query();
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let ua_bucket = token::ua_bucket(user_agent);
    let path_prefix = super::path_prefix();

    let token_ctx = match parse_existing_token(
        store,
        cfg,
        query,
        path,
        ip_bucket.as_str(),
        ua_bucket.as_str(),
        path_prefix,
        now_secs,
        now_ms_value,
    ) {
        Ok(ctx) => ctx,
        Err(reason) => {
            increment_behavior_score(store, ip, 2);
            let high_conf_count =
                high_confidence_violation_count(store, cfg, ip_bucket.as_str(), reason, now_secs);
            if let Some(action) = high_confidence_escalation_action(high_conf_count) {
                return MazeServeDecision::Fallback(MazeFallbackDecision { reason, action });
            }
            if should_enforce_violation(phase, reason) {
                return MazeServeDecision::Fallback(MazeFallbackDecision {
                    reason,
                    action: fallback_action_for_reason(reason),
                });
            }
            None
        }
    };

    let mut budget_lease = match try_acquire_budget(store, cfg, ip_bucket.as_str()) {
        Some(lease) => lease,
        None => {
            increment_behavior_score(store, ip, 1);
            if should_enforce_violation(phase, MazeFallbackReason::BudgetExceeded) {
                return MazeServeDecision::Fallback(MazeFallbackDecision {
                    reason: MazeFallbackReason::BudgetExceeded,
                    action: fallback_action_for_reason(MazeFallbackReason::BudgetExceeded),
                });
            }
            let flow_id = token::flow_id_from(ip_bucket.as_str(), ua_bucket.as_str(), path, now_secs);
            return MazeServeDecision::Serve(MazeRenderResult {
                html: render_budget_open_maze_page(path_prefix, flow_id.as_str()),
                depth: 0,
                flow_id,
                variant_id: "budget-open-compact".to_string(),
                seed_provider: "internal".to_string(),
                seed_version: now_secs,
                seed_metadata_only: true,
                seed_source_count: 0,
                response_cap_exceeded: false,
                bytes: 0,
                render_ms: 0,
                token_validated: false,
            });
        }
    };

    let entropy_nonce = flow_entropy_nonce(
        token_ctx.as_ref().map(|(_, token)| token),
        path_prefix,
        now_secs,
        ip_bucket.as_str(),
        ua_bucket.as_str(),
    );
    let minute_bucket = now_secs / cfg.maze_entropy_window_seconds.max(1);
    let secret = token::secret_from_env();
    let seed = token::entropy_seed(
        secret.as_str(),
        "default",
        ip_bucket.as_str(),
        ua_bucket.as_str(),
        path,
        minute_bucket,
        entropy_nonce.as_str(),
    );
    let mut rng = SeededRng::new(seed);
    let seed_corpus = super::seeds::load_seed_corpus(store, cfg, now_secs);
    let variant_layout = (seed & 0xff) as u8 % 3;
    let variant_palette = ((seed >> 8) & 0xff) as u8 % 3;
    let variant_id = format!(
        "maze-v{}{}-{}-s{}",
        variant_layout, variant_palette, minute_bucket, seed_corpus.version
    );
    let seed_term = if seed_corpus.terms.is_empty() {
        None
    } else {
        let idx = (seed as usize) % seed_corpus.terms.len();
        Some(seed_corpus.terms[idx].clone())
    };

    let render_cfg = MazeConfig::default();
    let mut paragraph_count = rng
        .range(render_cfg.min_paragraphs, render_cfg.max_paragraphs)
        .min(cfg.maze_max_paragraphs as usize)
        .max(1);
    let current_token = token_ctx.as_ref().map(|(_, token)| token);
    let branch_budget = current_token
        .map(|token| token.branch_budget)
        .unwrap_or(cfg.maze_token_branch_budget)
        .clamp(1, cfg.maze_token_branch_budget);
    let link_count = rng
        .range(render_cfg.min_links, render_cfg.max_links)
        .min(cfg.maze_max_links as usize)
        .max(1);
    let mut visible_links = cfg.maze_server_visible_links.min(link_count as u32).max(1) as usize;
    let child_branch_budget = branch_budget.saturating_sub(1).max(1);

    let current_depth = token_ctx
        .as_ref()
        .map(|(_, token)| token.depth)
        .unwrap_or(0);
    let behavior_score = current_behavior_score(store, ip);
    let violation_count = current_violation_count(store, ip_bucket.as_str(), now_secs);
    let suspicion_score = botness_hint.unwrap_or(behavior_score);
    let mut style_tier = if current_depth >= cfg.maze_no_js_fallback_max_depth
        && violation_count >= 2
        && suspicion_score >= cfg.botness_maze_threshold
    {
        MazeStyleTier::Machine
    } else if suspicion_score >= cfg.botness_maze_threshold || current_depth >= 2 {
        MazeStyleTier::Lite
    } else {
        MazeStyleTier::Full
    };

    let estimate = 1900usize
        .saturating_add(paragraph_count.saturating_mul(220))
        .saturating_add(visible_links.saturating_mul(280));
    if estimate > cfg.maze_max_response_bytes as usize {
        if should_enforce_violation(phase, MazeFallbackReason::BudgetExceeded) {
            return MazeServeDecision::Fallback(MazeFallbackDecision {
                reason: MazeFallbackReason::BudgetExceeded,
                action: fallback_action_for_reason(MazeFallbackReason::BudgetExceeded),
            });
        }
        paragraph_count = 1;
        visible_links = 1;
        style_tier = MazeStyleTier::Machine;
    }

    let mut paragraphs = Vec::with_capacity(paragraph_count);
    for index in 0..paragraph_count {
        let mut paragraph = generate_paragraph(&mut rng);
        if index == 0 {
            if let Some(term) = seed_term.as_deref() {
                paragraph.push_str(format!(" Reference focus: {}.", term).as_str());
            }
        }
        paragraphs.push(paragraph);
    }

    let hidden_count = link_count
        .saturating_sub(visible_links)
        .min(branch_budget as usize);
    let mut visible_link_set = Vec::with_capacity(visible_links);
    for _ in 0..visible_links {
        let segment_len = if cfg.maze_path_entropy_segment_len < 8 {
            8
        } else {
            cfg.maze_path_entropy_segment_len as usize
        };
        let next_path = format!(
            "{}{}",
            path_prefix,
            generate_path_segment(&mut rng, segment_len)
        );
        let next_token = token::issue_child_token(
            current_token,
            next_path.as_str(),
            path_prefix,
            ip_bucket.as_str(),
            ua_bucket.as_str(),
            cfg.maze_token_ttl_seconds,
            cfg.maze_token_max_depth,
            child_branch_budget,
            entropy_nonce.as_str(),
            variant_layout as u16 * 10 + variant_palette as u16,
            now_secs,
        );
        let raw_child = token::sign(&next_token, secret.as_str());
        let href = format!("{}?mt={}", next_path, raw_child);
        let pow = pow_difficulty_for_depth(cfg, next_token.depth);
        let topical_suffix = if seed_corpus.terms.is_empty() {
            None
        } else {
            let idx = (rng.next() as usize) % seed_corpus.terms.len();
            Some(seed_corpus.terms[idx].clone())
        };
        let link_text = if let Some(term) = topical_suffix.as_deref() {
            format!("{} {}", generate_link_text(&mut rng), capitalize(term))
        } else {
            generate_link_text(&mut rng)
        };
        let link_description = if let Some(term) = topical_suffix.as_deref() {
            format!("{} Context stream: {}.", generate_paragraph(&mut rng), term)
        } else {
            generate_paragraph(&mut rng)
        };
        visible_link_set.push(AdvancedMazeLink {
            href,
            text: link_text,
            description: link_description,
            pow_difficulty: pow,
        });
    }

    let checkpoint_token = token_ctx
        .as_ref()
        .map(|(raw, _)| raw.clone())
        .unwrap_or_default();
    let flow_id = token_ctx
        .as_ref()
        .map(|(_, token)| token.flow_id.clone())
        .unwrap_or_else(|| {
            token::flow_id_from(
                ip_bucket.as_str(),
                ua_bucket.as_str(),
                path_prefix,
                now_secs,
            )
        });
    let expansion_seed = (seed & 0xffff_ffff) as u32;
    let expansion_segment_len = cfg.maze_path_entropy_segment_len.max(8) as usize;
    let expansion_seed_signature = token::sign_expansion_seed(
        secret.as_str(),
        flow_id.as_str(),
        path_prefix,
        entropy_nonce.as_str(),
        current_depth,
        expansion_seed as u64,
        hidden_count,
        expansion_segment_len,
    );
    let bootstrap_json = serde_json::json!({
        "flow_id": flow_id,
        "depth": current_depth,
        "checkpoint_token": checkpoint_token,
        "path_prefix": path_prefix,
        "entropy_nonce": entropy_nonce,
        "assets": {
            "worker_url": super::assets::maze_worker_path()
        },
        "client_expansion": {
            "enabled": cfg.maze_client_expansion_enabled,
            "seed": expansion_seed,
            "seed_sig": expansion_seed_signature,
            "hidden_count": hidden_count,
            "segment_len": expansion_segment_len,
            "issue_path": super::issue_links_path()
        }
    })
    .to_string();

    let title = generate_title(&mut rng);
    let render_options = AdvancedMazeRenderOptions {
        title,
        breadcrumb: make_breadcrumb(&mut rng),
        paragraphs,
        links: visible_link_set,
        bootstrap_json,
        variant_layout,
        variant_palette,
        style_tier,
        style_sheet_url: match style_tier {
            MazeStyleTier::Machine => None,
            _ => Some(super::assets::maze_style_path().to_string()),
        },
        script_url: super::assets::maze_script_path().to_string(),
    };
    let started_at = now_ms();
    let html = generate_polymorphic_maze_page(&render_options);
    let elapsed_ms = now_ms().saturating_sub(started_at);
    let bytes = html.as_bytes().len();
    budget_lease.release();

    let response_cap_exceeded = bytes > cfg.maze_max_response_bytes as usize
        || elapsed_ms > cfg.maze_max_response_duration_ms;
    if response_cap_exceeded {
        increment_behavior_score(store, ip, 1);
        if should_enforce_violation(phase, MazeFallbackReason::BudgetExceeded) {
            return MazeServeDecision::Fallback(MazeFallbackDecision {
                reason: MazeFallbackReason::BudgetExceeded,
                action: fallback_action_for_reason(MazeFallbackReason::BudgetExceeded),
            });
        }
    }

    if token_ctx.is_some() && current_depth > 0 && current_depth % 2 == 0 {
        // Reduce routine KV write pressure by sampling progression scoring every other depth.
        increment_behavior_score(store, ip, 1);
    }
    MazeServeDecision::Serve(MazeRenderResult {
        html,
        depth: current_depth,
        flow_id,
        variant_id,
        seed_provider: seed_corpus.provider,
        seed_version: seed_corpus.version,
        seed_metadata_only: seed_corpus.metadata_only,
        seed_source_count: seed_corpus.source_count,
        response_cap_exceeded,
        bytes,
        render_ms: elapsed_ms,
        token_validated: token_ctx.is_some(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        checkpoint_key, replay_seen, token_replay_key, MazeFallbackReason, MazeServeDecision,
    };
    use crate::maze::state::MazeStateStore;
    use serde_json::Value;
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemStore {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MazeStateStore for MemStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }
    }

    fn maze_path(suffix: &str) -> String {
        super::super::entry_path(suffix)
    }

    #[test]
    fn helper_keys_are_stable() {
        assert_eq!(token_replay_key("f", "o"), "maze:token:seen:f:o");
        assert_eq!(checkpoint_key("f", "ip"), "maze:checkpoint:f:ip");
    }

    #[test]
    fn replay_state_detects_seen() {
        let store = MemStore::default();
        let key = token_replay_key("flow", "op");
        MazeStateStore::set(&store, key.as_str(), b"9999999999").expect("set replay key");
        assert!(replay_seen(&store, "flow", "op", 1));
    }

    #[test]
    fn high_confidence_escalation_transitions_from_challenge_to_block() {
        assert_eq!(super::high_confidence_escalation_action(1), None);
        assert_eq!(
            super::high_confidence_escalation_action(2),
            Some(super::MazeFallbackAction::Challenge)
        );
        assert_eq!(
            super::high_confidence_escalation_action(3),
            Some(super::MazeFallbackAction::Block)
        );
    }

    #[test]
    fn budget_open_fallback_uses_compact_shell_not_legacy_inline_page() {
        let store = MemStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_rollout_phase = crate::config::MazeRolloutPhase::Instrument;
        cfg.maze_max_concurrent_global = 0;

        let req = Request::builder()
            .method(Method::Get)
            .uri(maze_path("budget-open"))
            .body(Vec::<u8>::new())
            .build();
        let request_path = maze_path("budget-open");
        let decision = super::serve(
            &store,
            &cfg,
            &req,
            "198.51.100.91",
            "BudgetOpenBot/1.0",
            request_path.as_str(),
            None,
        );
        let MazeServeDecision::Serve(rendered) = decision else {
            panic!("expected maze serve decision");
        };
        assert!(
            !rendered.html.contains("<style>"),
            "budget-open path should not use legacy inline style renderer"
        );
        assert!(rendered.html.contains(super::super::assets::maze_script_path()));
    }

    fn first_maze_link(html: &str) -> Option<String> {
        for fragment in html.split("<a ") {
            if !fragment.contains("data-link-kind=\"maze\"") {
                continue;
            }
            let href_idx = fragment.find("href=\"")?;
            let start = href_idx + 6;
            let rest = &fragment[start..];
            let end = rest.find('"')?;
            return Some(rest[..end].to_string());
        }
        None
    }

    fn extract_bootstrap_json(html: &str) -> Value {
        let marker = "<script id=\"maze-bootstrap\" type=\"application/json\">";
        let start = html.find(marker).expect("bootstrap script should exist") + marker.len();
        let end = html[start..]
            .find("</script>")
            .map(|offset| start + offset)
            .expect("bootstrap script should terminate");
        serde_json::from_str(&html[start..end]).expect("bootstrap json should parse")
    }

    fn mt_from_uri(uri: &str) -> Option<String> {
        let (_, query) = uri.split_once('?')?;
        for segment in query.split('&') {
            let (key, value) = segment.split_once('=').unwrap_or((segment, ""));
            if key == "mt" && !value.is_empty() {
                return Some(value.to_string());
            }
        }
        None
    }

    fn fallback_reason(decision: MazeServeDecision) -> MazeFallbackReason {
        match decision {
            MazeServeDecision::Fallback(fallback) => fallback.reason,
            MazeServeDecision::Serve(_) => panic!("expected fallback decision"),
        }
    }

    #[test]
    fn invalid_token_maps_to_fallback() {
        let store = MemStore::default();
        let cfg = crate::config::defaults().clone();
        let req = Request::builder()
            .method(Method::Get)
            .uri(format!("{}?mt=bad-token", maze_path("abc")))
            .body(Vec::<u8>::new())
            .build();
        let request_path = maze_path("abc");
        let decision = super::serve(
            &store,
            &cfg,
            &req,
            "198.51.100.9",
            "TestUA/1.0",
            request_path.as_str(),
            None,
        );
        let reason = fallback_reason(decision);
        assert!(
            matches!(
                reason,
                MazeFallbackReason::TokenInvalid | MazeFallbackReason::TokenBindingMismatch
            ),
            "unexpected fallback reason: {:?}",
            reason
        );
    }

    #[test]
    fn live_maze_html_does_not_include_giveaway_markers_by_default() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_DEBUG_HEADERS");

        let store = MemStore::default();
        let cfg = crate::config::defaults().clone();
        let request_path = maze_path("entry");
        let req = Request::builder()
            .method(Method::Get)
            .uri(request_path.as_str())
            .body(Vec::<u8>::new())
            .build();
        let decision = super::serve(
            &store,
            &cfg,
            &req,
            "198.51.100.9",
            "TestUA/1.0",
            request_path.as_str(),
            None,
        );
        match decision {
            MazeServeDecision::Serve(rendered) => {
                assert!(!rendered.html.contains("Variant maze-v"));
                assert!(!rendered
                    .html
                    .contains("Synthetic navigation surface. Not authoritative content."));
            }
            MazeServeDecision::Fallback(fallback) => {
                panic!("expected served maze page, got fallback: {:?}", fallback)
            }
        }
    }

    #[test]
    fn live_maze_html_does_not_include_giveaway_markers_when_debug_enabled() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_DEBUG_HEADERS", "true");

        let store = MemStore::default();
        let cfg = crate::config::defaults().clone();
        let request_path = maze_path("entry");
        let req = Request::builder()
            .method(Method::Get)
            .uri(request_path.as_str())
            .body(Vec::<u8>::new())
            .build();
        let decision = super::serve(
            &store,
            &cfg,
            &req,
            "198.51.100.9",
            "TestUA/1.0",
            request_path.as_str(),
            None,
        );
        match decision {
            MazeServeDecision::Serve(rendered) => {
                assert!(!rendered.html.contains("Variant maze-v"));
                assert!(!rendered
                    .html
                    .contains("Synthetic navigation surface. Not authoritative content."));
            }
            MazeServeDecision::Fallback(fallback) => {
                panic!("expected served maze page, got fallback: {:?}", fallback)
            }
        }
        std::env::remove_var("SHUMA_DEBUG_HEADERS");
    }

    #[test]
    fn branch_budget_caps_progressive_link_issuance() {
        let store = MemStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_token_branch_budget = 1;
        cfg.maze_client_expansion_enabled = true;

        let entry_req = Request::builder()
            .method(Method::Get)
            .uri(maze_path("branch-budget-entry"))
            .body(Vec::<u8>::new())
            .build();
        let entry_path = maze_path("branch-budget-entry");
        let entry = super::serve(
            &store,
            &cfg,
            &entry_req,
            "198.51.100.45",
            "BudgetBot/1.0",
            entry_path.as_str(),
            None,
        );
        let MazeServeDecision::Serve(entry_page) = entry else {
            panic!("expected entry maze page");
        };
        let first_link =
            first_maze_link(entry_page.html.as_str()).expect("first link should exist");

        let child_req = Request::builder()
            .method(Method::Get)
            .uri(first_link.as_str())
            .body(Vec::<u8>::new())
            .build();
        let child_path = first_link.split('?').next().expect("path should exist");
        let child = super::serve(
            &store,
            &cfg,
            &child_req,
            "198.51.100.45",
            "BudgetBot/1.0",
            child_path,
            None,
        );
        let MazeServeDecision::Serve(child_page) = child else {
            panic!("expected child maze page");
        };

        let bootstrap = extract_bootstrap_json(child_page.html.as_str());
        let expansion = bootstrap
            .get("client_expansion")
            .cloned()
            .expect("client expansion should exist");
        let issue_req = Request::builder()
            .method(Method::Post)
            .uri(super::super::issue_links_path())
            .header("Content-Type", "application/json")
            .body(
                serde_json::json!({
                    "parent_token": bootstrap.get("checkpoint_token").and_then(|value| value.as_str()).unwrap_or_default(),
                    "flow_id": bootstrap.get("flow_id").and_then(|value| value.as_str()).unwrap_or_default(),
                    "entropy_nonce": bootstrap.get("entropy_nonce").and_then(|value| value.as_str()).unwrap_or_default(),
                    "path_prefix": bootstrap.get("path_prefix").and_then(|value| value.as_str()).unwrap_or(super::super::path_prefix()),
                    "seed": expansion.get("seed").and_then(|value| value.as_u64()).unwrap_or(0),
                    "seed_sig": expansion.get("seed_sig").and_then(|value| value.as_str()).unwrap_or_default(),
                    "hidden_count": expansion.get("hidden_count").and_then(|value| value.as_u64()).unwrap_or(0),
                    "requested_hidden_count": 6,
                    "segment_len": expansion.get("segment_len").and_then(|value| value.as_u64()).unwrap_or(16),
                    "candidates": []
                })
                .to_string()
                .into_bytes(),
            )
            .build();
        let issue_response =
            super::handle_issue_links(&store, &cfg, &issue_req, "198.51.100.45", "BudgetBot/1.0");
        assert_eq!(*issue_response.status(), 200);
        let json: Value = serde_json::from_slice(issue_response.body()).expect("valid json");
        let issued = json
            .get("links")
            .and_then(|value| value.as_array())
            .map(|value| value.len())
            .unwrap_or(0);
        assert_eq!(issued, 1);
    }

    #[test]
    fn child_token_branch_budget_decrements_from_parent_budget() {
        let store = MemStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_token_branch_budget = 4;
        cfg.maze_server_visible_links = 1;

        let req = Request::builder()
            .method(Method::Get)
            .uri(maze_path("budget-decrement"))
            .body(Vec::<u8>::new())
            .build();
        let request_path = maze_path("budget-decrement");
        let decision = super::serve(
            &store,
            &cfg,
            &req,
            "198.51.100.77",
            "BudgetStepBot/1.0",
            request_path.as_str(),
            None,
        );
        let MazeServeDecision::Serve(rendered) = decision else {
            panic!("expected maze page");
        };
        let first_link = first_maze_link(rendered.html.as_str()).expect("expected maze link");
        let raw_token = mt_from_uri(first_link.as_str()).expect("expected mt token");
        let parsed = super::token::verify(
            raw_token.as_str(),
            super::token::secret_from_env().as_str(),
            None,
        )
        .expect("token should verify");
        assert_eq!(parsed.branch_budget, 3);
    }

    #[test]
    fn issue_links_rejects_tampered_seed_signature() {
        let store = MemStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_client_expansion_enabled = true;

        let entry_req = Request::builder()
            .method(Method::Get)
            .uri(maze_path("issue-links-entry"))
            .body(Vec::<u8>::new())
            .build();
        let entry_path = maze_path("issue-links-entry");
        let entry = super::serve(
            &store,
            &cfg,
            &entry_req,
            "198.51.100.54",
            "SeedSigBot/1.0",
            entry_path.as_str(),
            None,
        );
        let MazeServeDecision::Serve(entry_page) = entry else {
            panic!("entry should serve maze");
        };
        let first_link =
            first_maze_link(entry_page.html.as_str()).expect("first link should exist");

        let child_req = Request::builder()
            .method(Method::Get)
            .uri(first_link.as_str())
            .body(Vec::<u8>::new())
            .build();
        let child_path = first_link.split('?').next().expect("path should exist");
        let child = super::serve(
            &store,
            &cfg,
            &child_req,
            "198.51.100.54",
            "SeedSigBot/1.0",
            child_path,
            None,
        );
        let MazeServeDecision::Serve(child_page) = child else {
            panic!("child page should serve maze");
        };
        let bootstrap = extract_bootstrap_json(child_page.html.as_str());
        let checkpoint_token = bootstrap
            .get("checkpoint_token")
            .and_then(|value| value.as_str())
            .expect("checkpoint token should exist");
        let flow_id = bootstrap
            .get("flow_id")
            .and_then(|value| value.as_str())
            .expect("flow id should exist");
        let entropy_nonce = bootstrap
            .get("entropy_nonce")
            .and_then(|value| value.as_str())
            .expect("entropy nonce should exist");
        let path_prefix = bootstrap
            .get("path_prefix")
            .and_then(|value| value.as_str())
            .expect("path prefix should exist");
        let expansion = bootstrap
            .get("client_expansion")
            .cloned()
            .expect("client expansion should exist");
        let seed = expansion
            .get("seed")
            .and_then(|value| value.as_u64())
            .expect("seed should exist");
        let hidden_count = expansion
            .get("hidden_count")
            .and_then(|value| value.as_u64())
            .expect("hidden count should exist");
        let segment_len = expansion
            .get("segment_len")
            .and_then(|value| value.as_u64())
            .expect("segment len should exist");
        let seed_sig = expansion
            .get("seed_sig")
            .and_then(|value| value.as_str())
            .expect("seed signature should exist");

        let invalid_payload = serde_json::json!({
            "parent_token": checkpoint_token,
            "flow_id": flow_id,
            "entropy_nonce": entropy_nonce,
            "path_prefix": path_prefix,
            "seed": seed,
            "seed_sig": format!("{}x", seed_sig),
            "hidden_count": hidden_count,
            "requested_hidden_count": hidden_count.min(2),
            "segment_len": segment_len,
            "candidates": []
        });
        let invalid_req = Request::builder()
            .method(Method::Post)
            .uri(super::super::issue_links_path())
            .header("Content-Type", "application/json")
            .body(invalid_payload.to_string().into_bytes())
            .build();
        let invalid_response = super::handle_issue_links(
            &store,
            &cfg,
            &invalid_req,
            "198.51.100.54",
            "SeedSigBot/1.0",
        );
        assert_eq!(*invalid_response.status(), 403);

        let valid_payload = serde_json::json!({
            "parent_token": checkpoint_token,
            "flow_id": flow_id,
            "entropy_nonce": entropy_nonce,
            "path_prefix": path_prefix,
            "seed": seed,
            "seed_sig": seed_sig,
            "hidden_count": hidden_count,
            "requested_hidden_count": hidden_count.min(2),
            "segment_len": segment_len,
            "candidates": []
        });
        let valid_req = Request::builder()
            .method(Method::Post)
            .uri(super::super::issue_links_path())
            .header("Content-Type", "application/json")
            .body(valid_payload.to_string().into_bytes())
            .build();
        let valid_response =
            super::handle_issue_links(&store, &cfg, &valid_req, "198.51.100.54", "SeedSigBot/1.0");
        assert_eq!(*valid_response.status(), 200);
    }

    #[test]
    fn issue_links_rejects_parent_token_replay() {
        let store = MemStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_client_expansion_enabled = true;
        cfg.maze_server_visible_links = 1;
        cfg.maze_token_branch_budget = 3;

        let entry_req = Request::builder()
            .method(Method::Get)
            .uri(maze_path("issue-links-replay-entry"))
            .body(Vec::<u8>::new())
            .build();
        let entry_path = maze_path("issue-links-replay-entry");
        let entry = super::serve(
            &store,
            &cfg,
            &entry_req,
            "198.51.100.99",
            "IssueReplayBot/1.0",
            entry_path.as_str(),
            None,
        );
        let MazeServeDecision::Serve(entry_page) = entry else {
            panic!("entry should serve maze");
        };
        let first_link =
            first_maze_link(entry_page.html.as_str()).expect("first link should exist");

        let child_req = Request::builder()
            .method(Method::Get)
            .uri(first_link.as_str())
            .body(Vec::<u8>::new())
            .build();
        let child_path = first_link.split('?').next().expect("path should exist");
        let child = super::serve(
            &store,
            &cfg,
            &child_req,
            "198.51.100.99",
            "IssueReplayBot/1.0",
            child_path,
            None,
        );
        let MazeServeDecision::Serve(child_page) = child else {
            panic!("child should serve maze");
        };

        let bootstrap = extract_bootstrap_json(child_page.html.as_str());
        let expansion = bootstrap
            .get("client_expansion")
            .cloned()
            .expect("client expansion should exist");
        let payload = serde_json::json!({
            "parent_token": bootstrap.get("checkpoint_token").and_then(|value| value.as_str()).unwrap_or_default(),
            "flow_id": bootstrap.get("flow_id").and_then(|value| value.as_str()).unwrap_or_default(),
            "entropy_nonce": bootstrap.get("entropy_nonce").and_then(|value| value.as_str()).unwrap_or_default(),
            "path_prefix": bootstrap.get("path_prefix").and_then(|value| value.as_str()).unwrap_or(super::super::path_prefix()),
            "seed": expansion.get("seed").and_then(|value| value.as_u64()).unwrap_or(0),
            "seed_sig": expansion.get("seed_sig").and_then(|value| value.as_str()).unwrap_or_default(),
            "hidden_count": expansion.get("hidden_count").and_then(|value| value.as_u64()).unwrap_or(0),
            "requested_hidden_count": 1,
            "segment_len": expansion.get("segment_len").and_then(|value| value.as_u64()).unwrap_or(16),
            "candidates": []
        })
        .to_string()
        .into_bytes();
        let issue_req = Request::builder()
            .method(Method::Post)
            .uri(super::super::issue_links_path())
            .header("Content-Type", "application/json")
            .body(payload.clone())
            .build();
        let first_issue = super::handle_issue_links(
            &store,
            &cfg,
            &issue_req,
            "198.51.100.99",
            "IssueReplayBot/1.0",
        );
        assert_eq!(*first_issue.status(), 200);

        let replay_req = Request::builder()
            .method(Method::Post)
            .uri(super::super::issue_links_path())
            .header("Content-Type", "application/json")
            .body(payload)
            .build();
        let replay_issue = super::handle_issue_links(
            &store,
            &cfg,
            &replay_req,
            "198.51.100.99",
            "IssueReplayBot/1.0",
        );
        assert_eq!(*replay_issue.status(), 409);
    }
}
