use serde::Serialize;
use std::collections::BTreeMap;

use super::adversary_sim::RuntimeLane;
use super::adversary_sim_identity_pool::load_identity_pool_from_env;
use super::adversary_sim_trusted_ingress::trusted_ingress_proxy_config_from_env;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct LaneRepresentativenessReadiness {
    pub(crate) status: String,
    pub(crate) summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RepresentativenessPrerequisites {
    pub(crate) trusted_ingress_configured: bool,
    pub(crate) scrapling_request_proxy_pool_count: usize,
    pub(crate) scrapling_browser_proxy_pool_count: usize,
    pub(crate) agentic_request_proxy_pool_count: usize,
    pub(crate) scrapling_request_proxy_configured: bool,
    pub(crate) scrapling_browser_proxy_configured: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AdversarySimRepresentativenessReadiness {
    pub(crate) status: String,
    pub(crate) summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) blockers: Vec<String>,
    pub(crate) representative_hostile_lane_count: u64,
    pub(crate) partially_representative_hostile_lane_count: u64,
    pub(crate) hostile_lane_count: u64,
    pub(crate) prerequisites: RepresentativenessPrerequisites,
    pub(crate) lane_statuses: BTreeMap<String, LaneRepresentativenessReadiness>,
}

pub(crate) fn project_representativeness_readiness() -> AdversarySimRepresentativenessReadiness {
    let trusted_ingress_configured = trusted_ingress_proxy_config_from_env().is_some();
    let scrapling_request_pool_count =
        load_identity_pool_from_env("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON").len();
    let scrapling_browser_pool_count =
        load_identity_pool_from_env("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON").len();
    let agentic_request_pool_count =
        load_identity_pool_from_env("ADVERSARY_SIM_AGENTIC_REQUEST_PROXY_POOL_JSON").len();
    let scrapling_request_proxy_configured =
        has_non_empty_env("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
    let scrapling_browser_proxy_configured =
        has_non_empty_env("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");

    let prerequisites = RepresentativenessPrerequisites {
        trusted_ingress_configured,
        scrapling_request_proxy_pool_count: scrapling_request_pool_count,
        scrapling_browser_proxy_pool_count: scrapling_browser_pool_count,
        agentic_request_proxy_pool_count: agentic_request_pool_count,
        scrapling_request_proxy_configured,
        scrapling_browser_proxy_configured,
    };

    let synthetic = synthetic_lane_readiness();
    let scrapling = scrapling_lane_readiness(&prerequisites);
    let agentic = agentic_lane_readiness(&prerequisites);
    let mixed = mixed_lane_readiness(&scrapling, &agentic);

    let hostile_lanes = [&scrapling, &agentic];
    let representative_hostile_lane_count = hostile_lanes
        .iter()
        .filter(|entry| entry.status == "representative")
        .count() as u64;
    let partially_representative_hostile_lane_count = hostile_lanes
        .iter()
        .filter(|entry| entry.status == "partially_representative")
        .count() as u64;
    let hostile_lane_count = hostile_lanes.len() as u64;
    let overall_status = mixed.status.clone();
    let overall_summary = match overall_status.as_str() {
        "representative" => {
            "Trusted ingress and the required hostile-lane proxy pools are configured, so representative attacker claims are currently supported.".to_string()
        }
        "partially_representative" => {
            "Some realism prerequisites are configured, but hostile-lane claims remain only partially representative until the missing infrastructure is supplied.".to_string()
        }
        _ => {
            "Critical realism infrastructure is missing, so hostile-lane traffic must be treated as degraded rather than representative.".to_string()
        }
    };

    let mut lane_statuses = BTreeMap::new();
    lane_statuses.insert(RuntimeLane::SyntheticTraffic.as_str().to_string(), synthetic);
    lane_statuses.insert(RuntimeLane::ScraplingTraffic.as_str().to_string(), scrapling.clone());
    lane_statuses.insert(RuntimeLane::BotRedTeam.as_str().to_string(), agentic.clone());
    lane_statuses.insert(RuntimeLane::ParallelMixedTraffic.as_str().to_string(), mixed.clone());

    AdversarySimRepresentativenessReadiness {
        status: overall_status,
        summary: overall_summary,
        blockers: mixed.blockers,
        representative_hostile_lane_count,
        partially_representative_hostile_lane_count,
        hostile_lane_count,
        prerequisites,
        lane_statuses,
    }
}

fn has_non_empty_env(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn synthetic_lane_readiness() -> LaneRepresentativenessReadiness {
    LaneRepresentativenessReadiness {
        status: "degraded".to_string(),
        summary:
            "Synthetic Traffic is non-hostile by design and cannot support representative attacker claims."
                .to_string(),
        blockers: vec![
            "Synthetic Traffic is non-hostile by design and cannot stand in for wild adversary traffic."
                .to_string(),
        ],
    }
}

fn scrapling_lane_readiness(
    prerequisites: &RepresentativenessPrerequisites,
) -> LaneRepresentativenessReadiness {
    let request_pool_ready = prerequisites.scrapling_request_proxy_pool_count >= 2;
    let browser_pool_ready = prerequisites.scrapling_browser_proxy_pool_count >= 2;
    let any_request_backing =
        request_pool_ready
            || prerequisites.scrapling_request_proxy_pool_count > 0
            || prerequisites.scrapling_request_proxy_configured
            || prerequisites.trusted_ingress_configured;
    let any_browser_backing =
        browser_pool_ready
            || prerequisites.scrapling_browser_proxy_pool_count > 0
            || prerequisites.scrapling_browser_proxy_configured
            || prerequisites.trusted_ingress_configured
            || prerequisites.scrapling_request_proxy_configured;
    let mut blockers = Vec::new();
    if !request_pool_ready {
        blockers.push(
            "Scrapling request traffic is not backed by a multi-identity proxy pool."
                .to_string(),
        );
    }
    if !browser_pool_ready {
        blockers.push(
            "Scrapling browser traffic is not backed by a multi-identity browser proxy pool."
                .to_string(),
        );
    }
    let status = if request_pool_ready && browser_pool_ready {
        "representative"
    } else if any_request_backing || any_browser_backing {
        "partially_representative"
    } else {
        "degraded"
    };
    let summary = match status {
        "representative" => {
            "Scrapling Traffic has the required request and browser identity backing for representative hostile-lane claims."
                .to_string()
        }
        "partially_representative" => {
            "Scrapling Traffic has some realism backing, but missing proxy-pool coverage still limits representative claims."
                .to_string()
        }
        _ => {
            "Scrapling Traffic lacks the request and browser identity backing needed for representative hostile-lane claims."
                .to_string()
        }
    };
    LaneRepresentativenessReadiness {
        status: status.to_string(),
        summary,
        blockers,
    }
}

fn agentic_lane_readiness(
    prerequisites: &RepresentativenessPrerequisites,
) -> LaneRepresentativenessReadiness {
    let request_pool_ready = prerequisites.agentic_request_proxy_pool_count >= 2;
    let browser_backing_ready = prerequisites.trusted_ingress_configured;
    let any_request_backing =
        request_pool_ready || prerequisites.agentic_request_proxy_pool_count > 0 || browser_backing_ready;
    let mut blockers = Vec::new();
    if !request_pool_ready {
        blockers.push(
            "Agentic request traffic is not backed by a multi-identity proxy pool."
                .to_string(),
        );
    }
    if !browser_backing_ready {
        blockers.push(
            "Agentic browser traffic does not have trusted-ingress client-IP backing."
                .to_string(),
        );
    }
    let status = if request_pool_ready && browser_backing_ready {
        "representative"
    } else if any_request_backing {
        "partially_representative"
    } else {
        "degraded"
    };
    let summary = match status {
        "representative" => {
            "Agentic Traffic has both request-pool backing and browser trusted-ingress backing for representative hostile-lane claims."
                .to_string()
        }
        "partially_representative" => {
            "Agentic Traffic has some realism backing, but missing request-pool or browser trusted-ingress backing still limits representative claims."
                .to_string()
        }
        _ => {
            "Agentic Traffic lacks the request-pool and browser trusted-ingress backing needed for representative hostile-lane claims."
                .to_string()
        }
    };
    LaneRepresentativenessReadiness {
        status: status.to_string(),
        summary,
        blockers,
    }
}

fn mixed_lane_readiness(
    scrapling: &LaneRepresentativenessReadiness,
    agentic: &LaneRepresentativenessReadiness,
) -> LaneRepresentativenessReadiness {
    let mut blockers = Vec::new();
    blockers.extend(scrapling.blockers.iter().cloned());
    for blocker in &agentic.blockers {
        if !blockers.contains(blocker) {
            blockers.push(blocker.clone());
        }
    }
    let status = if scrapling.status == "representative" && agentic.status == "representative" {
        "representative"
    } else if scrapling.status != "degraded" || agentic.status != "degraded" {
        "partially_representative"
    } else {
        "degraded"
    };
    let summary = match status {
        "representative" => {
            "Scrapling + Agentic has the required mixed-lane infrastructure for representative hostile-lane claims."
                .to_string()
        }
        "partially_representative" => {
            "Scrapling + Agentic has some realism backing, but mixed-lane claims remain only partially representative until every hostile lane has its required infrastructure."
                .to_string()
        }
        _ => {
            "Scrapling + Agentic lacks the infrastructure required for representative mixed-lane hostile claims."
                .to_string()
        }
    };
    LaneRepresentativenessReadiness {
        status: status.to_string(),
        summary,
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::project_representativeness_readiness;

    fn sample_pool_json() -> &'static str {
        r#"[{"label":"res-gb-1","proxy_url":"http://127.0.0.1:8899","identity_class":"residential","country_code":"GB"},{"label":"mob-us-1","proxy_url":"http://127.0.0.1:8898","identity_class":"mobile","country_code":"US"}]"#
    }

    #[test]
    fn representativeness_readiness_is_degraded_without_realism_infrastructure() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_AGENTIC_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");

        let readiness = project_representativeness_readiness();

        assert_eq!(readiness.status, "degraded");
        assert_eq!(readiness.representative_hostile_lane_count, 0);
        assert_eq!(
            readiness
                .lane_statuses
                .get("scrapling_traffic")
                .expect("scrapling lane")
                .status,
            "degraded"
        );
        assert_eq!(
            readiness
                .lane_statuses
                .get("bot_red_team")
                .expect("agentic lane")
                .status,
            "degraded"
        );
    }

    #[test]
    fn representativeness_readiness_is_partial_with_trusted_ingress_only() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var(
            "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL",
            "http://127.0.0.1:3871",
        );
        std::env::set_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", "trusted-token");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_AGENTIC_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");

        let readiness = project_representativeness_readiness();

        assert_eq!(readiness.status, "partially_representative");
        assert_eq!(readiness.representative_hostile_lane_count, 0);
        assert_eq!(readiness.partially_representative_hostile_lane_count, 2);
        assert_eq!(
            readiness
                .lane_statuses
                .get("parallel_mixed_traffic")
                .expect("mixed lane")
                .status,
            "partially_representative"
        );
    }

    #[test]
    fn representativeness_readiness_is_representative_when_hostile_prerequisites_exist() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var(
            "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL",
            "http://127.0.0.1:3871",
        );
        std::env::set_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", "trusted-token");
        std::env::set_var(
            "ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON",
            sample_pool_json(),
        );
        std::env::set_var(
            "ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON",
            sample_pool_json(),
        );
        std::env::set_var(
            "ADVERSARY_SIM_AGENTIC_REQUEST_PROXY_POOL_JSON",
            sample_pool_json(),
        );
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");

        let readiness = project_representativeness_readiness();

        assert_eq!(readiness.status, "representative");
        assert_eq!(readiness.representative_hostile_lane_count, 2);
        assert!(readiness.blockers.is_empty());
        assert_eq!(
            readiness
                .lane_statuses
                .get("scrapling_traffic")
                .expect("scrapling lane")
                .status,
            "representative"
        );
        assert_eq!(
            readiness
                .lane_statuses
                .get("bot_red_team")
                .expect("agentic lane")
                .status,
            "representative"
        );
        assert_eq!(
            readiness
                .lane_statuses
                .get("parallel_mixed_traffic")
                .expect("mixed lane")
                .status,
            "representative"
        );
    }
}
