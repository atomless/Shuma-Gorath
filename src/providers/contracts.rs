use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RateLimitDecision {
    Allowed,
    Limited,
}

impl RateLimitDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            RateLimitDecision::Allowed => "allowed",
            RateLimitDecision::Limited => "limited",
        }
    }
}

pub(crate) trait RateLimiterProvider {
    fn current_rate_usage(&self, store: &Store, site_id: &str, ip: &str) -> u32;
    fn check_rate_limit(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        limit: u32,
    ) -> RateLimitDecision;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BanSyncResult {
    Synced,
    Deferred,
    Failed,
}

impl BanSyncResult {
    pub fn as_str(self) -> &'static str {
        match self {
            BanSyncResult::Synced => "synced",
            BanSyncResult::Deferred => "deferred",
            BanSyncResult::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BanLookupResult {
    Banned,
    NotBanned,
    Unavailable,
}

impl BanLookupResult {
    pub fn as_str(self) -> &'static str {
        match self {
            BanLookupResult::Banned => "banned",
            BanLookupResult::NotBanned => "not_banned",
            BanLookupResult::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum BanListResult {
    Available(Vec<(String, crate::enforcement::ban::BanEntry)>),
    Unavailable,
}

pub(crate) trait BanStoreProvider {
    fn is_banned(&self, store: &Store, site_id: &str, ip: &str) -> BanLookupResult;
    fn list_active_bans(&self, store: &Store, site_id: &str) -> BanListResult;
    fn ban_ip_with_fingerprint(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    ) -> BanSyncResult;
    fn unban_ip(&self, store: &Store, site_id: &str, ip: &str) -> BanSyncResult;
}

pub(crate) trait ChallengeEngineProvider {
    fn puzzle_path(&self) -> &'static str;
    fn not_a_bot_path(&self) -> &'static str;
    fn render_challenge(
        &self,
        req: &Request,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response;
    fn render_not_a_bot(&self, req: &Request, cfg: &crate::config::Config) -> Response;
    fn serve_challenge_page(
        &self,
        req: &Request,
        shadow_mode: bool,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response;
    fn serve_not_a_bot_page(
        &self,
        req: &Request,
        shadow_mode: bool,
        cfg: &crate::config::Config,
    ) -> Response;
    fn handle_challenge_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
        challenge_puzzle_attempt_window_seconds: u64,
        challenge_puzzle_attempt_limit_per_window: u32,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome);
    fn handle_not_a_bot_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::challenge::NotABotSubmitResult;
    fn handle_pow_challenge(
        &self,
        ip: &str,
        user_agent: &str,
        enabled: bool,
        difficulty: u8,
        ttl_seconds: u64,
    ) -> Response;
    fn handle_pow_verify(&self, req: &Request, ip: &str, enabled: bool) -> Response;
}

pub(crate) trait MazeTarpitProvider {
    fn is_maze_path(&self, path: &str) -> bool;
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
    ) -> Response;

    fn maybe_handle_tarpit(
        &self,
        _req: &Request,
        _store: &Store,
        _cfg: &crate::config::Config,
        _site_id: &str,
        _ip: &str,
    ) -> Option<Response> {
        None
    }

    fn handle_tarpit_progress(
        &self,
        _req: &Request,
        _store: &Store,
        _cfg: &crate::config::Config,
        _site_id: &str,
        _ip: &str,
        _user_agent: &str,
    ) -> Response {
        Response::new(404, "Not Found")
    }
}

pub(crate) trait FingerprintSignalProvider {
    fn report_path(&self) -> &'static str;
    fn source_availability(
        &self,
        cfg: &crate::config::Config,
    ) -> crate::signals::botness::SignalAvailability;
    fn handle_report(&self, store: &Store, req: &Request) -> Response;
    fn detection_script(&self) -> &'static str;
    fn report_script(&self, report_endpoint: &str) -> String;
    fn inject_detection(&self, html: &str, report_endpoint: Option<&str>) -> String;
}

pub(crate) trait VerifiedIdentityProvider {
    fn verify_identity(
        &self,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::bot_identity::verification::IdentityVerificationResult;
}

#[cfg(test)]
mod tests {
    use super::{BanLookupResult, BanSyncResult, RateLimitDecision};

    #[test]
    fn rate_limit_decision_has_stable_labels() {
        assert_eq!(RateLimitDecision::Allowed.as_str(), "allowed");
        assert_eq!(RateLimitDecision::Limited.as_str(), "limited");
    }

    #[test]
    fn ban_sync_result_has_stable_labels() {
        assert_eq!(BanSyncResult::Synced.as_str(), "synced");
        assert_eq!(BanSyncResult::Deferred.as_str(), "deferred");
        assert_eq!(BanSyncResult::Failed.as_str(), "failed");
    }

    #[test]
    fn ban_lookup_result_has_stable_labels() {
        assert_eq!(BanLookupResult::Banned.as_str(), "banned");
        assert_eq!(BanLookupResult::NotBanned.as_str(), "not_banned");
        assert_eq!(BanLookupResult::Unavailable.as_str(), "unavailable");
    }
}
