use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub(crate) const DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION: &str =
    "sim-deterministic-attack-corpus.v1";
pub(crate) const DETERMINISTIC_ATTACK_CORPUS_PATH: &str =
    "scripts/tests/adversarial/deterministic_attack_corpus.v1.json";

pub(crate) static DETERMINISTIC_ATTACK_CORPUS: Lazy<DeterministicAttackCorpus> =
    Lazy::new(load_deterministic_attack_corpus);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DeterministicAttackCorpus {
    pub(crate) schema_version: String,
    pub(crate) corpus_revision: String,
    pub(crate) taxonomy_version: String,
    pub(crate) runtime_profile: String,
    pub(crate) ci_profile: String,
    pub(crate) runtime_toggle: RuntimeDeterministicProfile,
    pub(crate) ci_oracle: CiOracleDeterministicProfile,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RuntimeDeterministicProfile {
    pub(crate) active_lane_count: u32,
    pub(crate) primary_request_count: u64,
    pub(crate) supplemental_request_count: u64,
    pub(crate) primary_public_paths: Vec<String>,
    pub(crate) honeypot_probe_moduli: Vec<u64>,
    pub(crate) rate_burst: RateBurstProfile,
    pub(crate) lane_ip_octets: LaneIpOctets,
    pub(crate) lane_ip_rotation_ticks: LaneIpRotationTicks,
    pub(crate) lane_ip_entropy_salts: LaneIpEntropySalts,
    pub(crate) metadata: RuntimeMetadataProfile,
    pub(crate) paths: RuntimePathProfile,
    pub(crate) taxonomy: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RateBurstProfile {
    pub(crate) low: u64,
    pub(crate) medium: u64,
    pub(crate) high: u64,
    pub(crate) high_modulus: u64,
    pub(crate) medium_modulus: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct LaneIpOctets {
    pub(crate) rate_burst: u8,
    pub(crate) fingerprint_probe: u8,
    pub(crate) challenge_abuse: u8,
    pub(crate) pow_abuse: u8,
    pub(crate) tarpit_abuse: u8,
    pub(crate) cdp_report: u8,
    pub(crate) not_a_bot_fail: u8,
    pub(crate) not_a_bot_escalate: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct LaneIpRotationTicks {
    pub(crate) rate_burst: u64,
    pub(crate) fingerprint_probe: u64,
    pub(crate) challenge_abuse: u64,
    pub(crate) pow_abuse: u64,
    pub(crate) tarpit_abuse: u64,
    pub(crate) cdp_report: u64,
    pub(crate) not_a_bot_fail: u64,
    pub(crate) not_a_bot_escalate: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct LaneIpEntropySalts {
    pub(crate) rate_burst: u64,
    pub(crate) fingerprint_probe: u64,
    pub(crate) challenge_abuse: u64,
    pub(crate) pow_abuse: u64,
    pub(crate) tarpit_abuse: u64,
    pub(crate) cdp_report: u64,
    pub(crate) not_a_bot_fail: u64,
    pub(crate) not_a_bot_escalate: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RuntimeMetadataProfile {
    pub(crate) sim_profile: String,
    pub(crate) sim_lane: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct RuntimePathProfile {
    pub(crate) public_search: String,
    pub(crate) pow: String,
    pub(crate) not_a_bot_checkbox: String,
    pub(crate) honeypot: String,
    pub(crate) challenge_submit: String,
    pub(crate) pow_verify: String,
    pub(crate) cdp_report: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CiOracleDeterministicProfile {
    pub(crate) drivers: BTreeMap<String, CiDriverDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CiDriverDefinition {
    pub(crate) driver_class: String,
    pub(crate) path_hint: String,
    pub(crate) taxonomy_category: String,
}

pub(crate) fn deterministic_runtime_profile() -> &'static RuntimeDeterministicProfile {
    &DETERMINISTIC_ATTACK_CORPUS.runtime_toggle
}

pub(crate) fn deterministic_corpus_metadata_payload() -> serde_json::Value {
    json!({
        "schema_version": DETERMINISTIC_ATTACK_CORPUS.schema_version.clone(),
        "corpus_revision": DETERMINISTIC_ATTACK_CORPUS.corpus_revision.clone(),
        "taxonomy_version": DETERMINISTIC_ATTACK_CORPUS.taxonomy_version.clone(),
        "contract_path": DETERMINISTIC_ATTACK_CORPUS_PATH,
        "runtime_profile": DETERMINISTIC_ATTACK_CORPUS.runtime_profile.clone(),
        "ci_profile": DETERMINISTIC_ATTACK_CORPUS.ci_profile.clone(),
        "ci_driver_count": DETERMINISTIC_ATTACK_CORPUS.ci_oracle.drivers.len()
    })
}

fn default_deterministic_attack_corpus() -> DeterministicAttackCorpus {
    let mut ci_drivers = BTreeMap::new();
    for (driver, driver_class, path_hint, taxonomy_category) in [
        (
            "allow_browser_allowlist",
            "browser_realistic",
            "/sim/public/",
            "allowlist",
        ),
        (
            "not_a_bot_pass",
            "browser_realistic",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "challenge_puzzle_fail_maze",
            "browser_realistic",
            "/challenge/puzzle",
            "challenge",
        ),
        ("pow_success", "cost_imposition", "/pow", "pow"),
        ("pow_invalid_proof", "cost_imposition", "/pow/verify", "pow"),
        (
            "rate_limit_enforce",
            "http_scraper",
            "/sim/public/",
            "rate",
        ),
        (
            "retry_storm_enforce",
            "http_scraper",
            "/sim/public/",
            "rate",
        ),
        (
            "geo_challenge",
            "browser_realistic",
            "/sim/public/about/",
            "geo",
        ),
        (
            "geo_maze",
            "browser_realistic",
            "/sim/public/research/",
            "geo",
        ),
        (
            "geo_block",
            "browser_realistic",
            "/sim/public/plans/",
            "geo",
        ),
        ("honeypot_deny_temp", "browser_realistic", "/instaban", "honeypot"),
        (
            "not_a_bot_replay_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "not_a_bot_stale_token_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "not_a_bot_ordering_cadence_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "not_a_bot_replay_tarpit_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "tarpit",
        ),
        (
            "fingerprint_inconsistent_payload",
            "http_scraper",
            "/fingerprint-report",
            "fingerprint",
        ),
        (
            "header_spoofing_probe",
            "browser_realistic",
            "/sim/public/",
            "headers",
        ),
        ("cdp_high_confidence_deny", "http_scraper", "/cdp-report", "cdp"),
        (
            "akamai_additive_report",
            "edge_fixture",
            "/fingerprint-report",
            "akamai",
        ),
        (
            "akamai_authoritative_deny",
            "edge_fixture",
            "/fingerprint-report",
            "akamai",
        ),
    ] {
        ci_drivers.insert(
            driver.to_string(),
            CiDriverDefinition {
                driver_class: driver_class.to_string(),
                path_hint: path_hint.to_string(),
                taxonomy_category: taxonomy_category.to_string(),
            },
        );
    }

    let taxonomy = BTreeMap::from([
        ("public_probe".to_string(), "crawl_probe".to_string()),
        ("challenge_submit".to_string(), "challenge_abuse".to_string()),
        ("not_a_bot_fail".to_string(), "not_a_bot_fail".to_string()),
        (
            "not_a_bot_escalate".to_string(),
            "not_a_bot_escalate".to_string(),
        ),
        ("pow_verify".to_string(), "pow_abuse".to_string()),
        ("tarpit_progress".to_string(), "tarpit_abuse".to_string()),
        ("fingerprint_probe".to_string(), "fingerprint_probe".to_string()),
        ("cdp_report".to_string(), "cdp_probe".to_string()),
        ("rate_burst".to_string(), "rate_burst".to_string()),
    ]);

    DeterministicAttackCorpus {
        schema_version: DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION.to_string(),
        corpus_revision: "default-fallback".to_string(),
        taxonomy_version: "sim-policy-taxonomy.v1".to_string(),
        runtime_profile: "runtime_toggle".to_string(),
        ci_profile: "ci_oracle".to_string(),
        runtime_toggle: RuntimeDeterministicProfile {
            active_lane_count: 2,
            primary_request_count: 9,
            supplemental_request_count: 7,
            primary_public_paths: vec![
                "/sim/public/".to_string(),
                "/sim/public/about/".to_string(),
                "/sim/public/research/".to_string(),
                "/sim/public/plans/".to_string(),
                "/sim/public/work/".to_string(),
                "/sim/public/atom.xml".to_string(),
            ],
            honeypot_probe_moduli: vec![5, 7],
            rate_burst: RateBurstProfile {
                low: 8,
                medium: 16,
                high: 24,
                high_modulus: 9,
                medium_modulus: 3,
            },
            lane_ip_octets: LaneIpOctets {
                rate_burst: 248,
                fingerprint_probe: 249,
                challenge_abuse: 250,
                pow_abuse: 251,
                tarpit_abuse: 252,
                cdp_report: 253,
                not_a_bot_fail: 246,
                not_a_bot_escalate: 247,
            },
            lane_ip_rotation_ticks: LaneIpRotationTicks {
                rate_burst: 24,
                fingerprint_probe: 2,
                challenge_abuse: 1,
                pow_abuse: 1,
                tarpit_abuse: 1,
                cdp_report: 2,
                not_a_bot_fail: 2,
                not_a_bot_escalate: 2,
            },
            lane_ip_entropy_salts: LaneIpEntropySalts {
                rate_burst: 79,
                fingerprint_probe: 53,
                challenge_abuse: 17,
                pow_abuse: 29,
                tarpit_abuse: 41,
                cdp_report: 67,
                not_a_bot_fail: 97,
                not_a_bot_escalate: 113,
            },
            metadata: RuntimeMetadataProfile {
                sim_profile: "runtime_toggle".to_string(),
                sim_lane: "deterministic_black_box".to_string(),
            },
            paths: RuntimePathProfile {
                public_search: "/sim/public/".to_string(),
                pow: "/pow".to_string(),
                not_a_bot_checkbox: "/challenge/not-a-bot-checkbox".to_string(),
                honeypot: "/instaban".to_string(),
                challenge_submit: "/challenge/puzzle".to_string(),
                pow_verify: "/pow/verify".to_string(),
                cdp_report: "/cdp-report".to_string(),
            },
            taxonomy,
        },
        ci_oracle: CiOracleDeterministicProfile { drivers: ci_drivers },
    }
}

fn load_deterministic_attack_corpus() -> DeterministicAttackCorpus {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/scripts/tests/adversarial/deterministic_attack_corpus.v1.json"
    ));
    let parsed = serde_json::from_str::<DeterministicAttackCorpus>(raw)
        .ok()
        .filter(|corpus| corpus.schema_version == DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION)
        .filter(|corpus| !corpus.corpus_revision.trim().is_empty())
        .filter(|corpus| !corpus.taxonomy_version.trim().is_empty())
        .filter(|corpus| !corpus.runtime_toggle.primary_public_paths.is_empty())
        .filter(|corpus| {
            corpus.runtime_toggle.primary_request_count
                == corpus.runtime_toggle.primary_public_paths.len() as u64 + 3
        })
        .filter(|corpus| !corpus.runtime_toggle.honeypot_probe_moduli.is_empty())
        .filter(|corpus| !corpus.ci_oracle.drivers.is_empty());
    parsed.unwrap_or_else(default_deterministic_attack_corpus)
}

#[cfg(test)]
mod tests {
    use super::default_deterministic_attack_corpus;

    #[test]
    fn default_deterministic_attack_corpus_targets_generated_sim_public_routes() {
        let corpus = default_deterministic_attack_corpus();
        assert_eq!(
            corpus.runtime_toggle.primary_public_paths,
            vec![
                "/sim/public/".to_string(),
                "/sim/public/about/".to_string(),
                "/sim/public/research/".to_string(),
                "/sim/public/plans/".to_string(),
                "/sim/public/work/".to_string(),
                "/sim/public/atom.xml".to_string(),
            ]
        );
        assert_eq!(corpus.runtime_toggle.paths.public_search, "/sim/public/");
        assert_eq!(
            corpus
                .ci_oracle
                .drivers
                .get("allow_browser_allowlist")
                .expect("allow-browser driver")
                .path_hint,
            "/sim/public/"
        );
        assert_eq!(
            corpus
                .ci_oracle
                .drivers
                .get("geo_challenge")
                .expect("geo-challenge driver")
                .path_hint,
            "/sim/public/about/"
        );
        assert_eq!(
            corpus
                .ci_oracle
                .drivers
                .get("geo_maze")
                .expect("geo-maze driver")
                .path_hint,
            "/sim/public/research/"
        );
        assert_eq!(
            corpus
                .ci_oracle
                .drivers
                .get("geo_block")
                .expect("geo-block driver")
                .path_hint,
            "/sim/public/plans/"
        );
    }
}
