use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

use super::contracts::{
    BanStoreProvider, ChallengeEngineProvider, FingerprintSignalProvider, MazeTarpitProvider,
    RateLimitDecision, RateLimiterProvider,
};

pub(crate) struct InternalRateLimiterProvider;
pub(crate) struct InternalBanStoreProvider;
pub(crate) struct InternalChallengeEngineProvider;
pub(crate) struct InternalMazeTarpitProvider;
pub(crate) struct InternalFingerprintSignalProvider;

pub(crate) const RATE_LIMITER: InternalRateLimiterProvider = InternalRateLimiterProvider;
pub(crate) const BAN_STORE: InternalBanStoreProvider = InternalBanStoreProvider;
pub(crate) const CHALLENGE_ENGINE: InternalChallengeEngineProvider =
    InternalChallengeEngineProvider;
pub(crate) const MAZE_TARPIT: InternalMazeTarpitProvider = InternalMazeTarpitProvider;
pub(crate) const FINGERPRINT_SIGNAL: InternalFingerprintSignalProvider =
    InternalFingerprintSignalProvider;

const TARPIT_ESCALATION_SHORT_BAN_SECONDS: u64 = 600;

pub(crate) fn tarpit_budget_global_active_key(site_id: &str) -> String {
    crate::tarpit::runtime::tarpit_budget_global_active_key(site_id)
}

pub(crate) fn tarpit_budget_bucket_active_prefix(site_id: &str) -> String {
    crate::tarpit::runtime::tarpit_budget_bucket_active_prefix(site_id)
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
    site_id: &str,
    ip: &str,
    user_agent: &str,
    persistence_count: u32,
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
                    "count={} ua_present={}",
                    persistence_count,
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
        TARPIT_ESCALATION_SHORT_BAN_SECONDS,
        Some(crate::enforcement::ban::BanFingerprint {
            score: None,
            signals: vec!["tarpit_persistence".to_string()],
            summary: Some(format!("count={}", persistence_count)),
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
                "short_ban_{}s count={}",
                TARPIT_ESCALATION_SHORT_BAN_SECONDS, persistence_count
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
    fn is_banned(&self, store: &Store, site_id: &str, ip: &str) -> bool {
        crate::enforcement::ban::is_banned(store, site_id, ip)
    }

    fn list_active_bans(
        &self,
        store: &Store,
        site_id: &str,
    ) -> Vec<(String, crate::enforcement::ban::BanEntry)> {
        crate::enforcement::ban::list_active_bans_with_scan(store, site_id)
    }

    fn ban_ip_with_fingerprint(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    ) {
        crate::enforcement::ban::ban_ip_with_fingerprint(
            store,
            site_id,
            ip,
            reason,
            duration_secs,
            fingerprint,
        );
    }

    fn unban_ip(&self, store: &Store, site_id: &str, ip: &str) {
        crate::enforcement::ban::unban_ip(store, site_id, ip);
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
        crate::boundaries::serve_challenge_page(
            req,
            shadow_mode,
            transform_count,
            seed_ttl_seconds,
        )
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
        let persistence_count = crate::tarpit::runtime::next_persistence_count(
            store,
            site_id,
            ip_bucket.as_str(),
            cfg.maze_replay_ttl_seconds,
        );
        let escalation = crate::tarpit::runtime::persistence_escalation(cfg, persistence_count);
        if let Some(response) = maybe_escalate_persistent_tarpit_client(
            store,
            site_id,
            ip,
            user_agent,
            persistence_count,
            escalation,
        ) {
            return Some(response);
        }
        crate::observability::metrics::record_tarpit_escalation_outcome(store, "none");

        let _budget_lease =
            match crate::tarpit::runtime::try_acquire_entry_budget(store, cfg, site_id, ip_bucket.as_str()) {
            Some(lease) => lease,
            None => {
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
        let handled = crate::tarpit::http::handle_progress(req, store, cfg, site_id, ip, user_agent);
        if let Some(reason) = handled.reject_reason {
            crate::observability::metrics::record_tarpit_progress_outcome(store, reason.as_str());
            if reason.is_budget() {
                return render_tarpit_budget_fallback(self, req, store, cfg, ip, user_agent);
            }
            return handled.response;
        }

        crate::observability::metrics::record_tarpit_progress_outcome(store, "advanced");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpit_duration_bucket_has_stable_ranges() {
        assert_eq!(crate::tarpit::runtime::tarpit_duration_bucket(0), "lt_1s");
        assert_eq!(crate::tarpit::runtime::tarpit_duration_bucket(999), "lt_1s");
        assert_eq!(crate::tarpit::runtime::tarpit_duration_bucket(1_000), "1_5s");
        assert_eq!(crate::tarpit::runtime::tarpit_duration_bucket(5_000), "5_20s");
        assert_eq!(crate::tarpit::runtime::tarpit_duration_bucket(20_000), "20s_plus");
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
}
