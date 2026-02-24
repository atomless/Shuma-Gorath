use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use spin_sdk::http::Response;
use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::deception::primitives::{
    marker_seen, progression_chain_key, progression_replay_key, try_acquire_shared_budget,
    BudgetLease, SharedBudgetGovernor,
};
use crate::tarpit::proof::{adaptive_difficulty, verify_hashcash, DifficultyPolicy};
use crate::tarpit::types::{
    ProgressRejectReason, TarpitProgressToken, PATH_CLASS_TARPIT_PROGRESS, TOKEN_VERSION_V1,
    WORK_ALG_HASHCASH_SHA256_V1,
};

type HmacSha256 = Hmac<Sha256>;

const TOKEN_PREFIX: &str = "trp1";
const MAX_PROGRESS_TOKEN_BYTES: usize = 4096;
const TARPIT_BUDGET_GLOBAL_ACTIVE_PREFIX: &str = "tarpit:budget:active:global";
const TARPIT_BUDGET_BUCKET_ACTIVE_PREFIX: &str = "tarpit:budget:active:bucket";
const TARPIT_PERSISTENCE_PREFIX: &str = "tarpit:persistence";
const TARPIT_PROGRESS_REPLAY_PREFIX: &str = "tarpit:progress:seen";
const TARPIT_PROGRESS_CHAIN_PREFIX: &str = "tarpit:progress:chain";
const TARPIT_PROGRESS_STEP_PREFIX: &str = "tarpit:progress:step";
const TARPIT_BUDGET_FLOW_BYTES_PREFIX: &str = "tarpit:budget:egress:flow";
const TARPIT_BUDGET_EGRESS_GLOBAL_PREFIX: &str = "tarpit:budget:egress:global";
const TARPIT_BUDGET_EGRESS_BUCKET_PREFIX: &str = "tarpit:budget:egress:bucket";
const TARPIT_ESCALATION_SHORT_BAN_THRESHOLD: u32 = 5;
const TARPIT_ESCALATION_BLOCK_THRESHOLD: u32 = 10;
const SHARD_ROTATION: [&str; 6] = [
    "archive index fragment catalog vector mesh",
    "service reference matrix compliance surface",
    "routing substrate checkpoint ledger ingress",
    "analysis corpus table signal topology stack",
    "history shard channel report segment profile",
    "namespace map registry spool transport cache",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TarpitPersistenceState {
    count: u32,
    expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PersistenceEscalation {
    None,
    ShortBan,
    Block,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EgressPolicy {
    pub window_seconds: u64,
    pub global_bytes_per_window: u64,
    pub bucket_bytes_per_window: u64,
    pub flow_max_bytes: u64,
    pub flow_max_duration_seconds: u64,
    pub chunk_base_bytes: u32,
    pub chunk_max_bytes: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct ProgressAdvanceSuccess {
    pub flow_id: String,
    pub step: u16,
    pub chunk: String,
    pub chunk_bytes: usize,
    pub flow_bytes_emitted: u64,
    pub next_token: String,
    pub next_difficulty: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgressAdvanceOutcome {
    Reject(ProgressRejectReason),
    Advanced,
}

#[derive(Debug, Clone)]
pub(crate) struct ProgressAdvanceResult {
    pub outcome: ProgressAdvanceOutcome,
    pub success: Option<ProgressAdvanceSuccess>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}

fn now_secs() -> u64 {
    now_ms() / 1000
}

fn op_id(flow_id: &str, step: u16, now: u64) -> String {
    crate::maze::token::digest(format!("{}:{}:{}", flow_id, step, now).as_str())
}

fn hmac_sign(secret: &str, payload: &[u8]) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any secret length");
    mac.update(payload);
    mac.finalize().into_bytes().to_vec()
}

fn sign_progress_token(token: &TarpitProgressToken, secret: &str) -> String {
    let payload = serde_json::to_vec(token).expect("tarpit token should serialize");
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.as_slice());
    let sig_b64 = URL_SAFE_NO_PAD.encode(hmac_sign(secret, payload.as_slice()));
    format!("{TOKEN_PREFIX}.{payload_b64}.{sig_b64}")
}

fn verify_progress_token(
    raw: &str,
    secret: &str,
    now: u64,
) -> Result<TarpitProgressToken, ProgressRejectReason> {
    if raw.trim().is_empty() || raw.len() > MAX_PROGRESS_TOKEN_BYTES {
        return Err(ProgressRejectReason::Malformed);
    }
    let mut parts = raw.split('.');
    let prefix = parts.next().ok_or(ProgressRejectReason::Malformed)?;
    let payload = parts.next().ok_or(ProgressRejectReason::Malformed)?;
    let sig = parts.next().ok_or(ProgressRejectReason::Malformed)?;
    if parts.next().is_some() || prefix != TOKEN_PREFIX {
        return Err(ProgressRejectReason::Malformed);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| ProgressRejectReason::Malformed)?;
    let expected_sig = hmac_sign(secret, payload_bytes.as_slice());
    let supplied_sig = URL_SAFE_NO_PAD
        .decode(sig)
        .map_err(|_| ProgressRejectReason::Malformed)?;
    if expected_sig != supplied_sig {
        return Err(ProgressRejectReason::SignatureMismatch);
    }
    let token = serde_json::from_slice::<TarpitProgressToken>(payload_bytes.as_slice())
        .map_err(|_| ProgressRejectReason::Malformed)?;
    if token.version != TOKEN_VERSION_V1 {
        return Err(ProgressRejectReason::InvalidVersion);
    }
    if token.path_class != PATH_CLASS_TARPIT_PROGRESS {
        return Err(ProgressRejectReason::PathMismatch);
    }
    if token.work_alg != WORK_ALG_HASHCASH_SHA256_V1 {
        return Err(ProgressRejectReason::Malformed);
    }
    if token.issued_at > token.expires_at {
        return Err(ProgressRejectReason::InvalidWindow);
    }
    if now > token.expires_at {
        return Err(ProgressRejectReason::Expired);
    }
    Ok(token)
}

fn read_u64(store: &Store, key: &str) -> u64 {
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0)
}

fn write_u64(store: &Store, key: &str, value: u64) {
    if let Err(err) = store.set(key, value.to_string().as_bytes()) {
        eprintln!("[tarpit] failed writing key={} err={:?}", key, err);
    }
}

fn add_u64(store: &Store, key: &str, amount: u64) -> u64 {
    let next = read_u64(store, key).saturating_add(amount);
    write_u64(store, key, next);
    next
}

fn persistence_key(site_id: &str, ip_bucket: &str) -> String {
    format!("{}:{}:{}", TARPIT_PERSISTENCE_PREFIX, site_id, ip_bucket)
}

fn progress_step_key(flow_id: &str) -> String {
    format!("{}:{}", TARPIT_PROGRESS_STEP_PREFIX, flow_id)
}

fn progress_flow_bytes_key(flow_id: &str) -> String {
    format!("{}:{}", TARPIT_BUDGET_FLOW_BYTES_PREFIX, flow_id)
}

fn budget_window_id(now: u64, window_seconds: u64) -> u64 {
    now / window_seconds.max(1)
}

fn egress_global_key(site_id: &str, window_id: u64) -> String {
    format!("{}:{}:{}", TARPIT_BUDGET_EGRESS_GLOBAL_PREFIX, site_id, window_id)
}

fn egress_bucket_key(site_id: &str, ip_bucket: &str, window_id: u64) -> String {
    format!(
        "{}:{}:{}:{}",
        TARPIT_BUDGET_EGRESS_BUCKET_PREFIX, site_id, ip_bucket, window_id
    )
}

fn increment_step(store: &Store, flow_id: &str, next_step: u16) {
    let key = progress_step_key(flow_id);
    write_u64(store, key.as_str(), u64::from(next_step));
}

fn expected_step(store: &Store, flow_id: &str) -> u16 {
    let key = progress_step_key(flow_id);
    read_u64(store, key.as_str()).min(u64::from(u16::MAX)) as u16
}

fn difficulty_policy_from_config(cfg: &crate::config::Config) -> DifficultyPolicy {
    let base = cfg.tarpit_hashcash_base_difficulty;
    DifficultyPolicy {
        min: cfg.tarpit_hashcash_min_difficulty,
        max: cfg.tarpit_hashcash_max_difficulty,
        base,
        adaptive: cfg.tarpit_hashcash_adaptive,
    }
}

pub(crate) fn egress_policy_from_config(cfg: &crate::config::Config) -> EgressPolicy {
    EgressPolicy {
        window_seconds: cfg.tarpit_egress_window_seconds,
        global_bytes_per_window: cfg.tarpit_egress_global_bytes_per_window,
        bucket_bytes_per_window: cfg.tarpit_egress_per_ip_bucket_bytes_per_window,
        flow_max_bytes: cfg.tarpit_egress_per_flow_max_bytes,
        flow_max_duration_seconds: cfg.tarpit_egress_per_flow_max_duration_seconds,
        chunk_base_bytes: cfg.tarpit_step_chunk_base_bytes,
        chunk_max_bytes: cfg.tarpit_step_chunk_max_bytes,
    }
}

fn jittered_chunk_size(flow_id: &str, step: u16, policy: EgressPolicy, jitter_percent: u8) -> usize {
    let digest = crate::maze::token::digest(format!("{}:{}", flow_id, step).as_str());
    let seed = u32::from_str_radix(digest.get(0..4).unwrap_or("0000"), 16).unwrap_or(0);
    let jitter = u32::from(jitter_percent);
    let spread = jitter.saturating_mul(2).saturating_add(1);
    let offset = (seed % spread) as i32 - jitter as i32;
    let mut bytes = (u64::from(policy.chunk_base_bytes) * (100 + offset) as u64) / 100;
    if bytes == 0 {
        bytes = 256;
    }
    bytes = bytes.min(u64::from(policy.chunk_max_bytes));
    bytes as usize
}

fn next_chunk(
    flow_id: &str,
    step: u16,
    policy: EgressPolicy,
    jitter_percent: u8,
    shard_rotation_enabled: bool,
) -> String {
    let target = jittered_chunk_size(flow_id, step, policy, jitter_percent).max(256);
    let digest = crate::maze::token::digest(format!("{}:{}:{}", flow_id, step, target).as_str());
    let shard_seed = u32::from_str_radix(digest.get(4..8).unwrap_or("0000"), 16).unwrap_or(0);
    let shard = if shard_rotation_enabled {
        SHARD_ROTATION[(shard_seed as usize) % SHARD_ROTATION.len()]
    } else {
        SHARD_ROTATION[0]
    };
    let mut out = String::with_capacity(target);
    while out.len() < target {
        out.push_str(shard);
        out.push('\n');
    }
    out.truncate(target);
    out
}

fn bootstrap_html(
    progress_token: &str,
    progress_path: &str,
    initial_difficulty: u8,
    source_path: &str,
) -> Vec<u8> {
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta name=\"robots\" content=\"noindex,nofollow,noarchive\"><title>Verification</title></head><body><main><h1>Verification pending</h1><p>Automated traffic is required to complete progressive proof steps before continuing.</p><p data-tarpit-source=\"{source_path}\">Progress endpoint: <code>{progress_path}</code></p></main><script>window.__shumaTarpit={{token:\"{progress_token}\",endpoint:\"{progress_path}\",difficulty:{initial_difficulty}}};</script></body></html>"
    )
    .into_bytes()
}

pub(crate) fn tarpit_budget_global_active_key(site_id: &str) -> String {
    format!("{}:{}", TARPIT_BUDGET_GLOBAL_ACTIVE_PREFIX, site_id)
}

pub(crate) fn tarpit_budget_bucket_active_prefix(site_id: &str) -> String {
    format!("{}:{}", TARPIT_BUDGET_BUCKET_ACTIVE_PREFIX, site_id)
}

pub(crate) fn now_duration_ms(started_at: u64) -> u64 {
    now_ms().saturating_sub(started_at)
}

pub(crate) fn now_millis() -> u64 {
    now_ms()
}

pub(crate) fn next_persistence_count(
    store: &Store,
    site_id: &str,
    ip_bucket: &str,
    ttl_seconds: u64,
) -> u32 {
    let key = persistence_key(site_id, ip_bucket);
    let now = now_secs();
    let mut state = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<TarpitPersistenceState>(raw.as_slice()).ok())
        .unwrap_or(TarpitPersistenceState {
            count: 0,
            expires_at: 0,
        });

    if now > state.expires_at {
        state.count = 0;
    }
    state.count = state.count.saturating_add(1).min(128);
    state.expires_at = now.saturating_add(ttl_seconds.max(300));
    if let Ok(raw) = serde_json::to_vec(&state) {
        if let Err(err) = store.set(key.as_str(), raw.as_slice()) {
            eprintln!(
                "[tarpit] failed persisting persistence state key={} err={:?}",
                key, err
            );
        }
    }
    state.count
}

pub(crate) fn persistence_escalation(
    cfg: &crate::config::Config,
    persistence_count: u32,
) -> PersistenceEscalation {
    if cfg.test_mode || persistence_count < TARPIT_ESCALATION_SHORT_BAN_THRESHOLD {
        return PersistenceEscalation::None;
    }
    if persistence_count >= TARPIT_ESCALATION_BLOCK_THRESHOLD {
        PersistenceEscalation::Block
    } else {
        PersistenceEscalation::ShortBan
    }
}

pub(crate) fn try_acquire_entry_budget<'a>(
    store: &'a Store,
    cfg: &crate::config::Config,
    site_id: &str,
    ip_bucket: &str,
) -> Option<BudgetLease<'a, Store>> {
    let budget_global_key = tarpit_budget_global_active_key(site_id);
    let budget_bucket_prefix = tarpit_budget_bucket_active_prefix(site_id);
    try_acquire_shared_budget(
        store,
        SharedBudgetGovernor {
            global_active_key: budget_global_key.as_str(),
            bucket_active_prefix: budget_bucket_prefix.as_str(),
            max_concurrent_global: cfg.tarpit_max_concurrent_global,
            max_concurrent_per_ip_bucket: cfg.tarpit_max_concurrent_per_ip_bucket,
        },
        ip_bucket,
    )
}

pub(crate) fn build_progressive_entry_response(
    cfg: &crate::config::Config,
    ip_bucket: &str,
    ua_bucket: &str,
    source_path: &str,
    progress_path: &str,
) -> Response {
    let now = now_secs();
    let flow_id = crate::maze::token::flow_id_from(ip_bucket, ua_bucket, progress_path, now);
    let policy = egress_policy_from_config(cfg);
    let difficulty = difficulty_policy_from_config(cfg).base;
    let token = TarpitProgressToken {
        version: TOKEN_VERSION_V1,
        operation_id: op_id(flow_id.as_str(), 0, now),
        flow_id: flow_id.clone(),
        step: 0,
        parent_digest: crate::maze::token::digest("entry"),
        ip_bucket: ip_bucket.to_string(),
        ua_bucket: ua_bucket.to_string(),
        path_class: PATH_CLASS_TARPIT_PROGRESS.to_string(),
        issued_at: now,
        expires_at: now.saturating_add(cfg.tarpit_progress_token_ttl_seconds),
        difficulty,
        work_alg: WORK_ALG_HASHCASH_SHA256_V1.to_string(),
        max_chunk_bytes: policy.chunk_max_bytes,
        flow_bytes_emitted: 0,
        flow_started_at: now,
        hint: Some("entry".to_string()),
        policy_epoch: None,
    };
    let secret = crate::maze::token::secret_from_env();
    let signed = sign_progress_token(&token, secret.as_str());
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store, no-cache, must-revalidate")
        .header("X-Robots-Tag", "noindex, nofollow")
        .body(bootstrap_html(
            signed.as_str(),
            progress_path,
            difficulty,
            source_path,
        ))
        .build()
}

fn chain_marker_key(flow_id: &str, parent_digest: &str) -> String {
    progression_chain_key(TARPIT_PROGRESS_CHAIN_PREFIX, flow_id, parent_digest)
}

fn replay_marker_key(flow_id: &str, operation_id: &str) -> String {
    progression_replay_key(TARPIT_PROGRESS_REPLAY_PREFIX, flow_id, operation_id)
}

fn bucket_pressure(used: u64, cap: u64) -> f32 {
    if cap == 0 {
        return 1.0;
    }
    (used as f32 / cap as f32).clamp(0.0, 1.5)
}

fn validate_budgets(
    store: &Store,
    site_id: &str,
    ip_bucket: &str,
    token: &TarpitProgressToken,
    chunk_bytes: usize,
    policy: EgressPolicy,
    now: u64,
) -> Result<(f32, f32), ProgressRejectReason> {
    if now > token.flow_started_at.saturating_add(policy.flow_max_duration_seconds) {
        return Err(ProgressRejectReason::BudgetExhausted);
    }
    let chunk = chunk_bytes as u64;
    if token.flow_bytes_emitted.saturating_add(chunk) > policy.flow_max_bytes {
        return Err(ProgressRejectReason::BudgetExhausted);
    }

    let flow_key = progress_flow_bytes_key(token.flow_id.as_str());
    let server_flow_bytes = read_u64(store, flow_key.as_str());
    if server_flow_bytes > 0 && token.flow_bytes_emitted != server_flow_bytes {
        return Err(ProgressRejectReason::InvalidWindow);
    }

    let window_id = budget_window_id(now, policy.window_seconds);
    let global_key = egress_global_key(site_id, window_id);
    let bucket_key = egress_bucket_key(site_id, ip_bucket, window_id);
    let global_used = read_u64(store, global_key.as_str());
    let bucket_used = read_u64(store, bucket_key.as_str());
    if global_used.saturating_add(chunk) > policy.global_bytes_per_window
        || bucket_used.saturating_add(chunk) > policy.bucket_bytes_per_window
    {
        return Err(ProgressRejectReason::BudgetExhausted);
    }

    Ok((
        bucket_pressure(global_used, policy.global_bytes_per_window),
        bucket_pressure(bucket_used, policy.bucket_bytes_per_window),
    ))
}

fn commit_progress_state(
    store: &Store,
    site_id: &str,
    ip_bucket: &str,
    token: &TarpitProgressToken,
    replay_ttl: u64,
    chunk_bytes: usize,
    policy: EgressPolicy,
    now: u64,
) {
    let seen_until = token.expires_at.min(now.saturating_add(replay_ttl));
    let replay_key = replay_marker_key(token.flow_id.as_str(), token.operation_id.as_str());
    crate::deception::primitives::mark_marker(store, replay_key.as_str(), seen_until);
    let chain_key = chain_marker_key(token.flow_id.as_str(), token.operation_digest().as_str());
    crate::deception::primitives::mark_marker(store, chain_key.as_str(), seen_until);

    let flow_key = progress_flow_bytes_key(token.flow_id.as_str());
    let updated_flow_bytes = token.flow_bytes_emitted.saturating_add(chunk_bytes as u64);
    write_u64(store, flow_key.as_str(), updated_flow_bytes);
    increment_step(store, token.flow_id.as_str(), token.step.saturating_add(1));

    let window_id = budget_window_id(now, policy.window_seconds);
    let global_key = egress_global_key(site_id, window_id);
    let bucket_key = egress_bucket_key(site_id, ip_bucket, window_id);
    add_u64(store, global_key.as_str(), chunk_bytes as u64);
    add_u64(store, bucket_key.as_str(), chunk_bytes as u64);
}

pub(crate) fn advance_progress(
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    request_ip_bucket: &str,
    request_ua_bucket: &str,
    raw_token: &str,
    nonce: &str,
) -> ProgressAdvanceResult {
    let now = now_secs();
    let secret = crate::maze::token::secret_from_env();
    let token = match verify_progress_token(raw_token, secret.as_str(), now) {
        Ok(token) => token,
        Err(reason) => {
            return ProgressAdvanceResult {
                outcome: ProgressAdvanceOutcome::Reject(reason),
                success: None,
            };
        }
    };

    if token.ip_bucket != request_ip_bucket {
        return ProgressAdvanceResult {
            outcome: ProgressAdvanceOutcome::Reject(ProgressRejectReason::BindingIpMismatch),
            success: None,
        };
    }
    if token.ua_bucket != request_ua_bucket {
        return ProgressAdvanceResult {
            outcome: ProgressAdvanceOutcome::Reject(ProgressRejectReason::BindingUaMismatch),
            success: None,
        };
    }
    if token.path_class != PATH_CLASS_TARPIT_PROGRESS {
        return ProgressAdvanceResult {
            outcome: ProgressAdvanceOutcome::Reject(ProgressRejectReason::PathMismatch),
            success: None,
        };
    }

    let expected = expected_step(store, token.flow_id.as_str());
    if token.step != expected {
        return ProgressAdvanceResult {
            outcome: ProgressAdvanceOutcome::Reject(ProgressRejectReason::StepOutOfOrder),
            success: None,
        };
    }
    if token.step > 0 {
        let chain_key = chain_marker_key(token.flow_id.as_str(), token.parent_digest.as_str());
        if !marker_seen(store, chain_key.as_str(), now) {
            return ProgressAdvanceResult {
                outcome: ProgressAdvanceOutcome::Reject(ProgressRejectReason::ParentChainMissing),
                success: None,
            };
        }
    }
    let replay_key = replay_marker_key(token.flow_id.as_str(), token.operation_id.as_str());
    if marker_seen(store, replay_key.as_str(), now) {
        return ProgressAdvanceResult {
            outcome: ProgressAdvanceOutcome::Reject(ProgressRejectReason::Replay),
            success: None,
        };
    }
    if !verify_hashcash(raw_token, nonce, token.difficulty) {
        return ProgressAdvanceResult {
            outcome: ProgressAdvanceOutcome::Reject(ProgressRejectReason::InvalidProof),
            success: None,
        };
    }

    let policy = egress_policy_from_config(cfg);
    let chunk = next_chunk(
        token.flow_id.as_str(),
        token.step,
        policy,
        cfg.tarpit_step_jitter_percent,
        cfg.tarpit_shard_rotation_enabled,
    );
    let (global_pressure, bucket_pressure) =
        match validate_budgets(store, site_id, request_ip_bucket, &token, chunk.len(), policy, now)
        {
            Ok(v) => v,
            Err(reason) => {
                return ProgressAdvanceResult {
                    outcome: ProgressAdvanceOutcome::Reject(reason),
                    success: None,
                };
            }
        };

    commit_progress_state(
        store,
        site_id,
        request_ip_bucket,
        &token,
        cfg.tarpit_progress_replay_ttl_seconds,
        chunk.len(),
        policy,
        now,
    );

    let difficulty = adaptive_difficulty(
        difficulty_policy_from_config(cfg),
        token.step.saturating_add(1),
        global_pressure,
        bucket_pressure,
    );
    let next_token = TarpitProgressToken {
        version: TOKEN_VERSION_V1,
        operation_id: op_id(token.flow_id.as_str(), token.step.saturating_add(1), now),
        flow_id: token.flow_id.clone(),
        step: token.step.saturating_add(1),
        parent_digest: token.operation_digest(),
        ip_bucket: token.ip_bucket.clone(),
        ua_bucket: token.ua_bucket.clone(),
        path_class: token.path_class.clone(),
        issued_at: now,
        expires_at: now.saturating_add(cfg.tarpit_progress_token_ttl_seconds),
        difficulty,
        work_alg: token.work_alg.clone(),
        max_chunk_bytes: token.max_chunk_bytes,
        flow_bytes_emitted: token.flow_bytes_emitted.saturating_add(chunk.len() as u64),
        flow_started_at: token.flow_started_at,
        hint: token.hint.clone(),
        policy_epoch: token.policy_epoch,
    };
    let signed_next = sign_progress_token(&next_token, secret.as_str());

    ProgressAdvanceResult {
        outcome: ProgressAdvanceOutcome::Advanced,
        success: Some(ProgressAdvanceSuccess {
            flow_id: token.flow_id,
            step: token.step,
            chunk_bytes: chunk.len(),
            flow_bytes_emitted: next_token.flow_bytes_emitted,
            chunk,
            next_token: signed_next,
            next_difficulty: difficulty,
        }),
    }
}

pub(crate) fn tarpit_duration_bucket(duration_ms: u64) -> &'static str {
    match duration_ms {
        0..=999 => "lt_1s",
        1000..=4999 => "1_5s",
        5000..=19_999 => "5_20s",
        _ => "20s_plus",
    }
}

pub(crate) fn tarpit_bytes_bucket(bytes: usize) -> &'static str {
    match bytes {
        0..=8_191 => "lt_8kb",
        8_192..=32_767 => "8_32kb",
        32_768..=131_071 => "32_128kb",
        131_072..=524_287 => "128_512kb",
        _ => "512kb_plus",
    }
}

pub(crate) fn crawler_safety_bypass(path: &str, user_agent: &str) -> bool {
    let sensitive_path = matches!(path, "/robots.txt" | "/sitemap.xml" | "/health");
    if sensitive_path {
        return true;
    }

    let ua = user_agent.to_ascii_lowercase();
    let known_indexer = [
        "googlebot",
        "bingbot",
        "duckduckbot",
        "slurp",
        "baiduspider",
        "yandexbot",
        "applebot",
    ];
    known_indexer.iter().any(|token| ua.contains(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpit_duration_bucket_has_stable_ranges() {
        assert_eq!(tarpit_duration_bucket(0), "lt_1s");
        assert_eq!(tarpit_duration_bucket(999), "lt_1s");
        assert_eq!(tarpit_duration_bucket(1_000), "1_5s");
        assert_eq!(tarpit_duration_bucket(5_000), "5_20s");
        assert_eq!(tarpit_duration_bucket(20_000), "20s_plus");
    }

    #[test]
    fn tarpit_bytes_bucket_has_stable_ranges() {
        assert_eq!(tarpit_bytes_bucket(0), "lt_8kb");
        assert_eq!(tarpit_bytes_bucket(8_191), "lt_8kb");
        assert_eq!(tarpit_bytes_bucket(8_192), "8_32kb");
        assert_eq!(tarpit_bytes_bucket(32_768), "32_128kb");
        assert_eq!(tarpit_bytes_bucket(131_072), "128_512kb");
        assert_eq!(tarpit_bytes_bucket(524_288), "512kb_plus");
    }

    #[test]
    fn crawler_safety_bypass_detects_sensitive_paths_and_known_bots() {
        assert!(crawler_safety_bypass("/robots.txt", ""));
        assert!(crawler_safety_bypass("/", "Mozilla/5.0 (compatible; Googlebot/2.1)"));
        assert!(!crawler_safety_bypass("/", "Mozilla/5.0"));
    }
}
