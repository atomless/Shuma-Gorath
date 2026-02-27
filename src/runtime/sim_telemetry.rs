use spin_sdk::http::Request;
use std::cell::RefCell;

const SIM_RUN_ID_HEADER: &str = "x-shuma-sim-run-id";
const SIM_PROFILE_HEADER: &str = "x-shuma-sim-profile";
const SIM_LANE_HEADER: &str = "x-shuma-sim-lane";
const SIM_VALUE_MAX_CHARS: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimulationRequestMetadata {
    pub sim_run_id: String,
    pub sim_profile: String,
    pub sim_lane: String,
}

thread_local! {
    static CURRENT_SIM_METADATA: RefCell<Option<SimulationRequestMetadata>> = const { RefCell::new(None) };
}

fn sanitize_sim_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > SIM_VALUE_MAX_CHARS {
        return None;
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '.'))
    {
        return None;
    }
    Some(trimmed.to_string())
}

fn header_value(req: &Request, name: &str) -> Option<String> {
    req.header(name)
        .and_then(|value| value.as_str())
        .and_then(sanitize_sim_value)
}

pub(crate) fn metadata_from_request(
    req: &Request,
    runtime_environment: crate::config::RuntimeEnvironment,
    env_available: bool,
) -> Option<SimulationRequestMetadata> {
    if !runtime_environment.is_dev() || !env_available {
        return None;
    }

    Some(SimulationRequestMetadata {
        sim_run_id: header_value(req, SIM_RUN_ID_HEADER)?,
        sim_profile: header_value(req, SIM_PROFILE_HEADER)?,
        sim_lane: header_value(req, SIM_LANE_HEADER)?,
    })
}

pub(crate) struct SimulationContextGuard {
    previous: Option<SimulationRequestMetadata>,
}

pub(crate) fn enter(metadata: Option<SimulationRequestMetadata>) -> SimulationContextGuard {
    let previous = CURRENT_SIM_METADATA.with(|cell| {
        let mut slot = cell.borrow_mut();
        let previous = slot.clone();
        *slot = metadata;
        previous
    });
    SimulationContextGuard { previous }
}

impl Drop for SimulationContextGuard {
    fn drop(&mut self) {
        CURRENT_SIM_METADATA.with(|cell| {
            *cell.borrow_mut() = self.previous.clone();
        });
    }
}

pub(crate) fn current_metadata() -> Option<SimulationRequestMetadata> {
    CURRENT_SIM_METADATA.with(|cell| cell.borrow().clone())
}

pub(crate) fn is_simulation_context_active() -> bool {
    current_metadata().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use spin_sdk::http::Method;

    fn make_request_with_headers(headers: &[(&str, &str)]) -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/");
        for (key, value) in headers {
            builder.header(*key, *value);
        }
        builder.build()
    }

    #[test]
    fn metadata_requires_dev_env_and_all_headers() {
        let req = make_request_with_headers(&[
            ("X-Shuma-Sim-Run-Id", "run_123"),
            ("X-Shuma-Sim-Profile", "fast_smoke"),
            ("X-Shuma-Sim-Lane", "deterministic_black_box"),
        ]);

        let prod = metadata_from_request(
            &req,
            crate::config::RuntimeEnvironment::RuntimeProd,
            true,
        );
        assert!(prod.is_none());

        let dev_unavailable = metadata_from_request(
            &req,
            crate::config::RuntimeEnvironment::RuntimeDev,
            false,
        );
        assert!(dev_unavailable.is_none());

        let dev_enabled = metadata_from_request(
            &req,
            crate::config::RuntimeEnvironment::RuntimeDev,
            true,
        )
        .expect("dev metadata");
        assert_eq!(dev_enabled.sim_run_id, "run_123");
        assert_eq!(dev_enabled.sim_profile, "fast_smoke");
        assert_eq!(dev_enabled.sim_lane, "deterministic_black_box");
    }

    #[test]
    fn context_guard_restores_previous_metadata() {
        let first = SimulationRequestMetadata {
            sim_run_id: "run_a".to_string(),
            sim_profile: "fast_smoke".to_string(),
            sim_lane: "deterministic_black_box".to_string(),
        };
        let second = SimulationRequestMetadata {
            sim_run_id: "run_b".to_string(),
            sim_profile: "abuse_regression".to_string(),
            sim_lane: "deterministic_black_box".to_string(),
        };

        let _guard_a = enter(Some(first.clone()));
        assert_eq!(current_metadata(), Some(first.clone()));
        {
            let _guard_b = enter(Some(second.clone()));
            assert_eq!(current_metadata(), Some(second));
        }
        assert_eq!(current_metadata(), Some(first));
    }
}
