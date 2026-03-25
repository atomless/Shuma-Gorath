use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

use super::contracts::{
    BanStoreProvider, ChallengeEngineProvider, FingerprintSignalProvider, MazeTarpitProvider,
    RateLimitDecision, RateLimiterProvider, VerifiedIdentityProvider,
};

pub(crate) struct InternalRateLimiterProvider;
pub(crate) struct InternalBanStoreProvider;
pub(crate) struct InternalChallengeEngineProvider;
pub(crate) struct InternalMazeTarpitProvider;
pub(crate) struct InternalFingerprintSignalProvider;
pub(crate) struct InternalVerifiedIdentityProvider;

pub(crate) const RATE_LIMITER: InternalRateLimiterProvider = InternalRateLimiterProvider;
pub(crate) const BAN_STORE: InternalBanStoreProvider = InternalBanStoreProvider;
pub(crate) const CHALLENGE_ENGINE: InternalChallengeEngineProvider =
    InternalChallengeEngineProvider;
pub(crate) const MAZE_TARPIT: InternalMazeTarpitProvider = InternalMazeTarpitProvider;
pub(crate) const FINGERPRINT_SIGNAL: InternalFingerprintSignalProvider =
    InternalFingerprintSignalProvider;
pub(crate) const VERIFIED_IDENTITY: InternalVerifiedIdentityProvider =
    InternalVerifiedIdentityProvider;

pub(crate) fn tarpit_budget_global_active_key(site_id: &str) -> String {
    crate::tarpit::runtime::tarpit_budget_global_active_key(site_id)
}

pub(crate) fn tarpit_budget_bucket_active_prefix(site_id: &str) -> String {
    crate::tarpit::runtime::tarpit_budget_bucket_active_prefix(site_id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TarpitPersistenceCounts {
    bucket_count: u32,
    principal_count: u32,
}

fn tarpit_persistence_counts(
    store: &(impl crate::maze::state::MazeStateStore + ?Sized),
    site_id: &str,
    ip: &str,
    ttl_seconds: u64,
) -> TarpitPersistenceCounts {
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    TarpitPersistenceCounts {
        bucket_count: crate::tarpit::runtime::next_persistence_count(
            store,
            site_id,
            ip_bucket.as_str(),
            ttl_seconds,
        ),
        principal_count: crate::tarpit::runtime::next_persistence_principal_count(
            store,
            site_id,
            ip,
            ttl_seconds,
        ),
    }
}

fn tarpit_entry_budget_exhaustion_reason(
    store: &(impl crate::deception::primitives::DeceptionStateStore + ?Sized),
    cfg: &crate::config::Config,
    site_id: &str,
    ip_bucket: &str,
) -> crate::tarpit::runtime::BudgetExhaustionReason {
    let global_used = crate::deception::primitives::read_counter(
        store,
        crate::tarpit::runtime::tarpit_budget_global_active_key(site_id).as_str(),
    );
    let bucket_key = crate::deception::primitives::budget_bucket_key(
        crate::tarpit::runtime::tarpit_budget_bucket_active_prefix(site_id).as_str(),
        ip_bucket,
    );
    let bucket_used = crate::deception::primitives::read_counter(store, bucket_key.as_str());
    let global_exhausted = global_used >= cfg.tarpit_max_concurrent_global;
    let bucket_exhausted = bucket_used >= cfg.tarpit_max_concurrent_per_ip_bucket;
    match (global_exhausted, bucket_exhausted) {
        (true, true) => crate::tarpit::runtime::BudgetExhaustionReason::EntryGlobalAndBucketCap,
        (true, false) => crate::tarpit::runtime::BudgetExhaustionReason::EntryGlobalCap,
        (false, true) => crate::tarpit::runtime::BudgetExhaustionReason::EntryBucketCap,
        (false, false) => crate::tarpit::runtime::BudgetExhaustionReason::EntryGlobalCap,
    }
}

fn render_tarpit_budget_fallback(
    provider: &InternalMazeTarpitProvider,
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    ip: &str,
    user_agent: &str,
) -> Response {
    match cfg.tarpit_fallback_action {
        crate::config::TarpitFallbackAction::Maze => {
            crate::observability::metrics::record_tarpit_budget_outcome(store, "fallback_maze");
            provider.serve_maze_with_tracking(
                req,
                store,
                cfg,
                ip,
                user_agent,
                crate::maze::entry_path("tarpit-budget-fallback").as_str(),
                "tarpit_budget_fallback_maze",
                format!(
                    "source_path={} mode={} fallback=maze",
                    req.path(),
                    "progressive"
                )
                .as_str(),
                Some(10),
            )
        }
        crate::config::TarpitFallbackAction::Block => {
            crate::observability::metrics::record_tarpit_budget_outcome(store, "fallback_block");
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::BlocksTotal,
                None,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Block,
                    ip: Some(ip.to_string()),
                    reason: Some("tarpit_budget_fallback_block".to_string()),
                    outcome: Some(format!(
                        "source_path={} mode={} fallback=block",
                        req.path(),
                        "progressive"
                    )),
                    admin: None,
                },
            );
            Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::Honeypot,
                ),
            )
        }
    }
}

fn maybe_escalate_persistent_tarpit_client(
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    ip: &str,
    user_agent: &str,
    principal_count: u32,
    bucket_count: u32,
    escalation: crate::tarpit::runtime::PersistenceEscalation,
) -> Option<Response> {
    if escalation == crate::tarpit::runtime::PersistenceEscalation::None {
        return None;
    }
    if escalation == crate::tarpit::runtime::PersistenceEscalation::Block {
        crate::observability::metrics::record_tarpit_escalation_outcome(store, "block");
        crate::observability::metrics::increment(
            store,
            crate::observability::metrics::MetricName::BlocksTotal,
            None,
        );
        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Block,
                ip: Some(ip.to_string()),
                reason: Some("tarpit_persistence_block".to_string()),
                outcome: Some(format!(
                    "principal_count={} bucket_count={} ua_present={}",
                    principal_count,
                    bucket_count,
                    !user_agent.trim().is_empty()
                )),
                admin: None,
            },
        );
        return Some(Response::new(
            403,
            crate::enforcement::block_page::render_block_page(
                crate::enforcement::block_page::BlockReason::Honeypot,
            ),
        ));
    }

    crate::observability::metrics::record_tarpit_escalation_outcome(store, "short_ban");
    crate::enforcement::ban::ban_ip_with_fingerprint(
        store,
        site_id,
        ip,
        "tarpit_persistence",
        cfg.get_ban_duration("tarpit_persistence"),
        Some(crate::enforcement::ban::BanFingerprint {
            score: None,
            signals: vec!["tarpit_persistence".to_string()],
            summary: Some(format!(
                "principal_count={} bucket_count={}",
                principal_count, bucket_count
            )),
        }),
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BansTotal,
        Some("tarpit_persistence"),
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::BlocksTotal,
        None,
    );
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::Ban,
            ip: Some(ip.to_string()),
            reason: Some("tarpit_persistence".to_string()),
            outcome: Some(format!(
                "short_ban_{}s principal_count={} bucket_count={}",
                cfg.get_ban_duration("tarpit_persistence"),
                principal_count,
                bucket_count
            )),
            admin: None,
        },
    );

    Some(Response::new(
        403,
        crate::enforcement::block_page::render_block_page(
            crate::enforcement::block_page::BlockReason::Honeypot,
        ),
    ))
}

impl RateLimiterProvider for InternalRateLimiterProvider {
    fn current_rate_usage(&self, store: &Store, site_id: &str, ip: &str) -> u32 {
        crate::signals::rate_pressure::current_rate_usage(store, site_id, ip)
    }

    fn check_rate_limit(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        limit: u32,
    ) -> RateLimitDecision {
        if crate::enforcement::rate::check_rate_limit(store, site_id, ip, limit) {
            RateLimitDecision::Allowed
        } else {
            RateLimitDecision::Limited
        }
    }
}

impl BanStoreProvider for InternalBanStoreProvider {
    fn is_banned(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
    ) -> crate::providers::contracts::BanLookupResult {
        if crate::enforcement::ban::is_banned(store, site_id, ip) {
            crate::providers::contracts::BanLookupResult::Banned
        } else {
            crate::providers::contracts::BanLookupResult::NotBanned
        }
    }

    fn list_active_bans(
        &self,
        store: &Store,
        site_id: &str,
    ) -> crate::providers::contracts::BanListResult {
        crate::providers::contracts::BanListResult::Available(
            crate::enforcement::ban::list_active_bans_with_scan(store, site_id),
        )
    }

    fn ban_ip_with_fingerprint(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    ) -> crate::providers::contracts::BanSyncResult {
        crate::enforcement::ban::ban_ip_with_fingerprint(
            store,
            site_id,
            ip,
            reason,
            duration_secs,
            fingerprint,
        );
        crate::providers::contracts::BanSyncResult::Deferred
    }

    fn unban_ip(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
    ) -> crate::providers::contracts::BanSyncResult {
        crate::enforcement::ban::unban_ip(store, site_id, ip);
        crate::providers::contracts::BanSyncResult::Deferred
    }
}

impl ChallengeEngineProvider for InternalChallengeEngineProvider {
    fn puzzle_path(&self) -> &'static str {
        crate::boundaries::challenge_puzzle_path()
    }

    fn not_a_bot_path(&self) -> &'static str {
        crate::boundaries::challenge_not_a_bot_path()
    }

    fn render_challenge(
        &self,
        req: &Request,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response {
        crate::boundaries::render_challenge(req, transform_count, seed_ttl_seconds)
    }

    fn render_not_a_bot(&self, req: &Request, cfg: &crate::config::Config) -> Response {
        crate::boundaries::render_not_a_bot(req, cfg)
    }

    fn serve_challenge_page(
        &self,
        req: &Request,
        shadow_mode: bool,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response {
        crate::boundaries::serve_challenge_page(req, shadow_mode, transform_count, seed_ttl_seconds)
    }

    fn serve_not_a_bot_page(
        &self,
        req: &Request,
        shadow_mode: bool,
        cfg: &crate::config::Config,
    ) -> Response {
        crate::boundaries::serve_not_a_bot_page(req, shadow_mode, cfg)
    }

    fn handle_challenge_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
        challenge_puzzle_attempt_window_seconds: u64,
        challenge_puzzle_attempt_limit_per_window: u32,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
        crate::boundaries::handle_challenge_submit_with_outcome(
            store,
            req,
            challenge_puzzle_attempt_window_seconds,
            challenge_puzzle_attempt_limit_per_window,
        )
    }

    fn handle_not_a_bot_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::challenge::NotABotSubmitResult {
        crate::boundaries::handle_not_a_bot_submit_with_outcome(store, req, cfg)
    }

    fn handle_pow_challenge(
        &self,
        ip: &str,
        user_agent: &str,
        enabled: bool,
        difficulty: u8,
        ttl_seconds: u64,
    ) -> Response {
        crate::challenge::pow::handle_pow_challenge(
            ip,
            user_agent,
            enabled,
            difficulty,
            ttl_seconds,
        )
    }

    fn handle_pow_verify(&self, req: &Request, ip: &str, enabled: bool) -> Response {
        crate::challenge::pow::handle_pow_verify(req, ip, enabled)
    }
}

impl MazeTarpitProvider for InternalMazeTarpitProvider {
    fn is_maze_path(&self, path: &str) -> bool {
        crate::boundaries::is_maze_path(path)
    }

    fn tarpit_progress_path(&self) -> &'static str {
        crate::tarpit::progress_path()
    }

    fn serve_maze_with_tracking(
        &self,
        req: &Request,
        store: &Store,
        cfg: &crate::config::Config,
        ip: &str,
        user_agent: &str,
        path: &str,
        event_reason: &str,
        event_outcome: &str,
        botness_hint: Option<u8>,
    ) -> Response {
        crate::serve_maze_with_tracking(
            req,
            store,
            cfg,
            ip,
            user_agent,
            path,
            event_reason,
            event_outcome,
            botness_hint,
        )
    }

    fn maybe_handle_tarpit(
        &self,
        req: &Request,
        store: &Store,
        cfg: &crate::config::Config,
        site_id: &str,
        ip: &str,
    ) -> Option<Response> {
        if !cfg.maze_enabled || !cfg.tarpit_enabled {
            return None;
        }

        let started_at = crate::tarpit::runtime::now_millis();
        let user_agent = req
            .header("user-agent")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if crate::tarpit::runtime::crawler_safety_bypass(req.path(), user_agent) {
            return Some(self.serve_maze_with_tracking(
                req,
                store,
                cfg,
                ip,
                user_agent,
                crate::maze::entry_path("tarpit-safety-bypass").as_str(),
                "tarpit_crawler_safety_bypass",
                format!("source_path={} crawler_safety=true", req.path()).as_str(),
                Some(6),
            ));
        }

        let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
        let ua_bucket = crate::maze::token::ua_bucket(user_agent);
        let persistence_counts = tarpit_persistence_counts(
            store,
            site_id,
            ip,
            cfg.maze_replay_ttl_seconds,
        );
        let escalation =
            crate::tarpit::runtime::persistence_escalation(cfg, persistence_counts.principal_count);
        if let Some(response) = maybe_escalate_persistent_tarpit_client(
            store,
            cfg,
            site_id,
            ip,
            user_agent,
            persistence_counts.principal_count,
            persistence_counts.bucket_count,
            escalation,
        ) {
            return Some(response);
        }
        crate::observability::metrics::record_tarpit_escalation_outcome(store, "none");

        let _budget_lease = match crate::tarpit::runtime::try_acquire_entry_budget(
            store,
            cfg,
            site_id,
            ip_bucket.as_str(),
        ) {
            Some(lease) => lease,
            None => {
                let reason =
                    tarpit_entry_budget_exhaustion_reason(store, cfg, site_id, ip_bucket.as_str());
                crate::observability::metrics::record_tarpit_budget_exhaustion_reason(
                    store,
                    reason.as_str(),
                );
                crate::observability::metrics::record_tarpit_budget_outcome(store, "saturated");
                return Some(render_tarpit_budget_fallback(
                    self, req, store, cfg, ip, user_agent,
                ));
            }
        };

        crate::observability::metrics::record_tarpit_budget_outcome(store, "acquired");
        let response = crate::tarpit::runtime::build_progressive_entry_response(
            cfg,
            ip_bucket.as_str(),
            ua_bucket.as_str(),
            req.path(),
            self.tarpit_progress_path(),
        );

        let response_bytes = response.body().len();
        crate::observability::metrics::record_tarpit_activation(store, "progressive");
        crate::observability::metrics::record_tarpit_bytes_bucket(
            store,
            crate::tarpit::runtime::tarpit_bytes_bucket(response_bytes),
        );
        crate::observability::metrics::record_tarpit_duration_bucket(
            store,
            crate::tarpit::runtime::tarpit_duration_bucket(
                crate::tarpit::runtime::now_duration_ms(started_at),
            ),
        );

        Some(response)
    }

    fn handle_tarpit_progress(
        &self,
        req: &Request,
        store: &Store,
        cfg: &crate::config::Config,
        site_id: &str,
        ip: &str,
        user_agent: &str,
    ) -> Response {
        if !cfg.maze_enabled || !cfg.tarpit_enabled {
            return Response::new(404, "Not Found");
        }

        let started_at = crate::tarpit::runtime::now_millis();
        crate::observability::metrics::record_tarpit_proof_outcome(store, "required");
        let handled =
            crate::tarpit::http::handle_progress(req, store, cfg, site_id, ip, user_agent);
        if let Some(reason) = handled.reject_reason {
            crate::observability::metrics::record_tarpit_progress_outcome(store, reason.as_str());
            if reason == crate::tarpit::types::ProgressRejectReason::InvalidProof {
                crate::observability::metrics::record_tarpit_proof_outcome(store, "failed");
            }
            if let Some(chain_reason) = reason.chain_violation_reason() {
                crate::observability::metrics::record_tarpit_chain_violation(store, chain_reason);
            }
            if let Some(budget_reason) = handled.budget_exhaustion_reason {
                crate::observability::metrics::record_tarpit_budget_exhaustion_reason(
                    store,
                    budget_reason.as_str(),
                );
            }
            if reason.is_budget() {
                return render_tarpit_budget_fallback(self, req, store, cfg, ip, user_agent);
            }
            return handled.response;
        }

        crate::observability::metrics::record_tarpit_progress_outcome(store, "advanced");
        crate::observability::metrics::record_tarpit_proof_outcome(store, "passed");

        if let Some(chunk_bytes) = handled.chunk_bytes {
            crate::observability::metrics::record_tarpit_bytes_bucket(
                store,
                crate::tarpit::runtime::tarpit_bytes_bucket(chunk_bytes),
            );
        }
        crate::observability::metrics::record_tarpit_duration_bucket(
            store,
            crate::tarpit::runtime::tarpit_duration_bucket(
                crate::tarpit::runtime::now_duration_ms(started_at),
            ),
        );
        handled.response
    }
}

impl FingerprintSignalProvider for InternalFingerprintSignalProvider {
    fn report_path(&self) -> &'static str {
        "/cdp-report"
    }

    fn source_availability(
        &self,
        cfg: &crate::config::Config,
    ) -> crate::signals::botness::SignalAvailability {
        if cfg.cdp_detection_enabled {
            crate::signals::botness::SignalAvailability::Active
        } else {
            crate::signals::botness::SignalAvailability::Disabled
        }
    }

    fn handle_report(&self, store: &Store, req: &Request) -> Response {
        crate::signals::cdp::handle_cdp_report(store, req)
    }

    fn detection_script(&self) -> &'static str {
        crate::signals::cdp::get_cdp_detection_script()
    }

    fn report_script(&self, report_endpoint: &str) -> String {
        crate::signals::cdp::get_cdp_report_script(report_endpoint)
    }

    fn inject_detection(&self, html: &str, report_endpoint: Option<&str>) -> String {
        crate::signals::cdp::inject_cdp_detection(html, report_endpoint)
    }
}

impl VerifiedIdentityProvider for InternalVerifiedIdentityProvider {
    fn verify_identity(
        &self,
        store: &dyn crate::challenge::KeyValueStore,
        site_id: &str,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::bot_identity::verification::IdentityVerificationResult {
        if !cfg.verified_identity.enabled {
            return crate::bot_identity::verification::IdentityVerificationResult::disabled();
        }
        if !cfg.verified_identity.native_web_bot_auth_enabled {
            return crate::bot_identity::verification::IdentityVerificationResult::not_attempted();
        }

        crate::bot_identity::native_http_message_signatures::verify_request(
            store, site_id, req, cfg,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpit_duration_bucket_has_stable_ranges() {
        assert_eq!(crate::tarpit::runtime::tarpit_duration_bucket(0), "lt_1s");
        assert_eq!(crate::tarpit::runtime::tarpit_duration_bucket(999), "lt_1s");
        assert_eq!(
            crate::tarpit::runtime::tarpit_duration_bucket(1_000),
            "1_5s"
        );
        assert_eq!(
            crate::tarpit::runtime::tarpit_duration_bucket(5_000),
            "5_20s"
        );
        assert_eq!(
            crate::tarpit::runtime::tarpit_duration_bucket(20_000),
            "20s_plus"
        );
    }

    #[test]
    fn tarpit_bytes_bucket_has_stable_ranges() {
        assert_eq!(crate::tarpit::runtime::tarpit_bytes_bucket(0), "lt_8kb");
        assert_eq!(crate::tarpit::runtime::tarpit_bytes_bucket(8_191), "lt_8kb");
        assert_eq!(crate::tarpit::runtime::tarpit_bytes_bucket(8_192), "8_32kb");
        assert_eq!(
            crate::tarpit::runtime::tarpit_bytes_bucket(32_768),
            "32_128kb"
        );
        assert_eq!(
            crate::tarpit::runtime::tarpit_bytes_bucket(131_072),
            "128_512kb"
        );
        assert_eq!(
            crate::tarpit::runtime::tarpit_bytes_bucket(524_288),
            "512kb_plus"
        );
    }

    #[test]
    fn tarpit_budget_key_wrappers_match_runtime_helpers() {
        assert_eq!(
            tarpit_budget_global_active_key("default"),
            crate::tarpit::runtime::tarpit_budget_global_active_key("default")
        );
        assert_eq!(
            tarpit_budget_bucket_active_prefix("default"),
            crate::tarpit::runtime::tarpit_budget_bucket_active_prefix("default")
        );
    }

    #[test]
    fn tarpit_entry_budget_exhaustion_reason_distinguishes_cap_sources() {
        let store = crate::test_support::InMemoryStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.tarpit_max_concurrent_global = 4;
        cfg.tarpit_max_concurrent_per_ip_bucket = 2;
        let site_id = "default";
        let ip_bucket = "bucket-a";

        crate::deception::primitives::write_counter(
            &store,
            crate::tarpit::runtime::tarpit_budget_global_active_key(site_id).as_str(),
            4,
        );
        crate::deception::primitives::write_counter(
            &store,
            crate::deception::primitives::budget_bucket_key(
                crate::tarpit::runtime::tarpit_budget_bucket_active_prefix(site_id).as_str(),
                ip_bucket,
            )
            .as_str(),
            2,
        );

        assert_eq!(
            tarpit_entry_budget_exhaustion_reason(&store, &cfg, site_id, ip_bucket),
            crate::tarpit::runtime::BudgetExhaustionReason::EntryGlobalAndBucketCap
        );
    }

    #[test]
    fn verified_identity_internal_provider_is_a_noop_when_enabled() {
        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        let req = crate::test_support::request_with_headers("/", &[]);
        let store = crate::test_support::InMemoryStore::default();

        let result = VERIFIED_IDENTITY.verify_identity(&store, "default", &req, &cfg);

        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::NotAttempted
        );
        assert!(result.identity.is_none());
    }

    #[test]
    fn tarpit_persistence_escalation_does_not_cross_contaminate_same_bucket_ips() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = crate::config::defaults().clone();
        let ttl_seconds = cfg.maze_replay_ttl_seconds;

        let mut escalated = TarpitPersistenceCounts {
            bucket_count: 0,
            principal_count: 0,
        };
        for _ in 0..5 {
            escalated =
                tarpit_persistence_counts(&store, "default", "198.51.100.10", ttl_seconds);
        }

        assert_eq!(
            crate::tarpit::runtime::persistence_escalation(&cfg, escalated.principal_count),
            crate::tarpit::runtime::PersistenceEscalation::ShortBan
        );

        let fresh_same_bucket =
            tarpit_persistence_counts(&store, "default", "198.51.100.11", ttl_seconds);
        assert_eq!(fresh_same_bucket.bucket_count, 6);
        assert_eq!(fresh_same_bucket.principal_count, 1);
        assert_eq!(
            crate::tarpit::runtime::persistence_escalation(&cfg, fresh_same_bucket.principal_count),
            crate::tarpit::runtime::PersistenceEscalation::None
        );
    }
}
