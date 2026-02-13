use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

use super::contracts::{
    BanStoreProvider, BanSyncResult, ChallengeEngineProvider, FingerprintSignalProvider,
    MazeTarpitProvider, RateLimitDecision, RateLimiterProvider,
};
use super::internal;

pub(crate) struct UnsupportedExternalRateLimiterProvider;
pub(crate) struct UnsupportedExternalBanStoreProvider;
pub(crate) struct UnsupportedExternalChallengeEngineProvider;
pub(crate) struct UnsupportedExternalMazeTarpitProvider;
pub(crate) struct ExternalFingerprintSignalProvider;

pub(crate) const UNSUPPORTED_RATE_LIMITER: UnsupportedExternalRateLimiterProvider =
    UnsupportedExternalRateLimiterProvider;
pub(crate) const UNSUPPORTED_BAN_STORE: UnsupportedExternalBanStoreProvider =
    UnsupportedExternalBanStoreProvider;
pub(crate) const UNSUPPORTED_CHALLENGE_ENGINE: UnsupportedExternalChallengeEngineProvider =
    UnsupportedExternalChallengeEngineProvider;
pub(crate) const UNSUPPORTED_MAZE_TARPIT: UnsupportedExternalMazeTarpitProvider =
    UnsupportedExternalMazeTarpitProvider;
pub(crate) const FINGERPRINT_SIGNAL: ExternalFingerprintSignalProvider =
    ExternalFingerprintSignalProvider;

impl RateLimiterProvider for UnsupportedExternalRateLimiterProvider {
    fn current_rate_usage(&self, store: &Store, site_id: &str, ip: &str) -> u32 {
        internal::RATE_LIMITER.current_rate_usage(store, site_id, ip)
    }

    fn check_rate_limit(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        limit: u32,
    ) -> RateLimitDecision {
        internal::RATE_LIMITER.check_rate_limit(store, site_id, ip, limit)
    }
}

impl BanStoreProvider for UnsupportedExternalBanStoreProvider {
    fn is_banned(&self, store: &Store, site_id: &str, ip: &str) -> bool {
        internal::BAN_STORE.is_banned(store, site_id, ip)
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
        internal::BAN_STORE
            .ban_ip_with_fingerprint(store, site_id, ip, reason, duration_secs, fingerprint);
    }

    fn unban_ip(&self, store: &Store, site_id: &str, ip: &str) {
        internal::BAN_STORE.unban_ip(store, site_id, ip);
    }

    fn sync_ban(&self, _site_id: &str, _ip: &str) -> BanSyncResult {
        BanSyncResult::Failed
    }

    fn sync_unban(&self, _site_id: &str, _ip: &str) -> BanSyncResult {
        BanSyncResult::Failed
    }
}

impl ChallengeEngineProvider for UnsupportedExternalChallengeEngineProvider {
    fn puzzle_path(&self) -> &'static str {
        internal::CHALLENGE_ENGINE.puzzle_path()
    }

    fn render_challenge(&self, req: &Request, transform_count: usize) -> Response {
        internal::CHALLENGE_ENGINE.render_challenge(req, transform_count)
    }

    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
    ) -> Response {
        internal::CHALLENGE_ENGINE.serve_challenge_page(req, test_mode, transform_count)
    }

    fn handle_challenge_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
        internal::CHALLENGE_ENGINE.handle_challenge_submit_with_outcome(store, req)
    }

    fn handle_pow_challenge(
        &self,
        ip: &str,
        enabled: bool,
        difficulty: u8,
        ttl_seconds: u64,
    ) -> Response {
        internal::CHALLENGE_ENGINE.handle_pow_challenge(ip, enabled, difficulty, ttl_seconds)
    }

    fn handle_pow_verify(&self, req: &Request, ip: &str, enabled: bool) -> Response {
        internal::CHALLENGE_ENGINE.handle_pow_verify(req, ip, enabled)
    }
}

impl MazeTarpitProvider for UnsupportedExternalMazeTarpitProvider {
    fn is_maze_path(&self, path: &str) -> bool {
        internal::MAZE_TARPIT.is_maze_path(path)
    }

    fn handle_maze_request(&self, path: &str) -> Response {
        internal::MAZE_TARPIT.handle_maze_request(path)
    }

    fn serve_maze_with_tracking(
        &self,
        store: &Store,
        cfg: &crate::config::Config,
        ip: &str,
        path: &str,
        event_reason: &str,
        event_outcome: &str,
    ) -> Response {
        internal::MAZE_TARPIT
            .serve_maze_with_tracking(store, cfg, ip, path, event_reason, event_outcome)
    }
}

impl FingerprintSignalProvider for ExternalFingerprintSignalProvider {
    fn report_path(&self) -> &'static str {
        "/fingerprint-report"
    }

    fn source_availability(
        &self,
        cfg: &crate::config::Config,
    ) -> crate::signals::botness::SignalAvailability {
        if cfg.cdp_detection_enabled {
            crate::signals::botness::SignalAvailability::Unavailable
        } else {
            crate::signals::botness::SignalAvailability::Disabled
        }
    }

    fn handle_report(&self, _store: &Store, _req: &Request) -> Response {
        Response::new(
            501,
            "External fingerprint provider selected but not configured",
        )
    }

    fn detection_script(&self) -> &'static str {
        ""
    }

    fn report_script(&self, _report_endpoint: &str) -> String {
        String::new()
    }

    fn inject_detection(&self, html: &str, _report_endpoint: Option<&str>) -> String {
        html.to_string()
    }
}
