#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapabilityClass {
    MetricsWrite,
    MonitoringWrite,
    EventLogWrite,
    BanWrite,
    ResponsePrivileged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapabilityPhase {
    RequestBootstrap,
    PolicyExecution,
    PostResponseFlush,
}

const REQUEST_BOOTSTRAP_CLASSES: [CapabilityClass; 1] = [CapabilityClass::MetricsWrite];
const POLICY_EXECUTION_CLASSES: [CapabilityClass; 5] = [
    CapabilityClass::MetricsWrite,
    CapabilityClass::MonitoringWrite,
    CapabilityClass::EventLogWrite,
    CapabilityClass::BanWrite,
    CapabilityClass::ResponsePrivileged,
];
const POST_RESPONSE_FLUSH_CLASSES: [CapabilityClass; 1] = [CapabilityClass::MonitoringWrite];

pub(crate) fn capability_lattice_for_phase(phase: CapabilityPhase) -> &'static [CapabilityClass] {
    match phase {
        CapabilityPhase::RequestBootstrap => &REQUEST_BOOTSTRAP_CLASSES,
        CapabilityPhase::PolicyExecution => &POLICY_EXECUTION_CLASSES,
        CapabilityPhase::PostResponseFlush => &POST_RESPONSE_FLUSH_CLASSES,
    }
}

fn phase_allows(phase: CapabilityPhase, class: CapabilityClass) -> bool {
    capability_lattice_for_phase(phase).contains(&class)
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeCapabilities;

#[derive(Debug, Clone)]
pub(crate) struct RequestBootstrapCapabilities {
    metrics: MetricsCapability,
}

#[derive(Debug, Clone)]
pub(crate) struct PolicyExecutionCapabilities {
    metrics: MetricsCapability,
    monitoring: MonitoringCapability,
    event_log: EventLogCapability,
    ban_write: BanWriteCapability,
    response_privileged: ResponsePrivilegedCapability,
}

#[derive(Debug, Clone)]
pub(crate) struct PostResponseFlushCapabilities {
    monitoring: MonitoringCapability,
}

#[derive(Debug, Clone)]
pub(crate) struct MetricsCapability {
    _sealed: (),
}

#[derive(Debug, Clone)]
pub(crate) struct MonitoringCapability {
    _sealed: (),
}

#[derive(Debug, Clone)]
pub(crate) struct EventLogCapability {
    _sealed: (),
}

#[derive(Debug, Clone)]
pub(crate) struct BanWriteCapability {
    _sealed: (),
}

#[derive(Debug, Clone)]
pub(crate) struct ResponsePrivilegedCapability {
    _sealed: (),
}

mod sealed {
    pub(crate) trait PolicyExecutionMintAuthority {}

    impl PolicyExecutionMintAuthority for crate::runtime::request_flow::RequestFlowCapabilityToken {}
    impl PolicyExecutionMintAuthority for crate::LibCapabilityToken {}
}

impl RuntimeCapabilities {
    pub(crate) fn for_request_bootstrap_phase(
        _token: crate::runtime::request_flow::RequestFlowCapabilityToken,
    ) -> RequestBootstrapCapabilities {
        debug_assert!(phase_allows(
            CapabilityPhase::RequestBootstrap,
            CapabilityClass::MetricsWrite
        ));
        RequestBootstrapCapabilities {
            metrics: MetricsCapability { _sealed: () },
        }
    }

    /// Policy execution capabilities are minted at the request trust boundary
    /// and passed explicitly through orchestration and effect executors.
    pub(crate) fn for_policy_execution_phase(
        _token: impl sealed::PolicyExecutionMintAuthority,
    ) -> PolicyExecutionCapabilities {
        debug_assert!(phase_allows(
            CapabilityPhase::PolicyExecution,
            CapabilityClass::MetricsWrite
        ));
        debug_assert!(phase_allows(
            CapabilityPhase::PolicyExecution,
            CapabilityClass::MonitoringWrite
        ));
        debug_assert!(phase_allows(
            CapabilityPhase::PolicyExecution,
            CapabilityClass::EventLogWrite
        ));
        debug_assert!(phase_allows(
            CapabilityPhase::PolicyExecution,
            CapabilityClass::BanWrite
        ));
        debug_assert!(phase_allows(
            CapabilityPhase::PolicyExecution,
            CapabilityClass::ResponsePrivileged
        ));
        PolicyExecutionCapabilities {
            metrics: MetricsCapability { _sealed: () },
            monitoring: MonitoringCapability { _sealed: () },
            event_log: EventLogCapability { _sealed: () },
            ban_write: BanWriteCapability { _sealed: () },
            response_privileged: ResponsePrivilegedCapability { _sealed: () },
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test_policy_execution_phase() -> PolicyExecutionCapabilities {
        PolicyExecutionCapabilities {
            metrics: MetricsCapability { _sealed: () },
            monitoring: MonitoringCapability { _sealed: () },
            event_log: EventLogCapability { _sealed: () },
            ban_write: BanWriteCapability { _sealed: () },
            response_privileged: ResponsePrivilegedCapability { _sealed: () },
        }
    }

    pub(crate) fn for_post_response_flush_phase(
        _token: crate::LibCapabilityToken,
    ) -> PostResponseFlushCapabilities {
        debug_assert!(phase_allows(
            CapabilityPhase::PostResponseFlush,
            CapabilityClass::MonitoringWrite
        ));
        PostResponseFlushCapabilities {
            monitoring: MonitoringCapability { _sealed: () },
        }
    }
}

impl RequestBootstrapCapabilities {
    pub(crate) fn metrics(&self) -> &MetricsCapability {
        &self.metrics
    }
}

impl PolicyExecutionCapabilities {
    pub(crate) fn metrics(&self) -> &MetricsCapability {
        &self.metrics
    }

    pub(crate) fn monitoring(&self) -> &MonitoringCapability {
        &self.monitoring
    }

    pub(crate) fn event_log(&self) -> &EventLogCapability {
        &self.event_log
    }

    pub(crate) fn ban_write(&self) -> &BanWriteCapability {
        &self.ban_write
    }

    pub(crate) fn response_privileged(&self) -> &ResponsePrivilegedCapability {
        &self.response_privileged
    }
}

impl PostResponseFlushCapabilities {
    pub(crate) fn monitoring(&self) -> &MonitoringCapability {
        &self.monitoring
    }
}

#[cfg(test)]
mod tests {
    use super::{capability_lattice_for_phase, CapabilityClass, CapabilityPhase};

    #[test]
    fn request_bootstrap_lattice_is_metrics_only() {
        assert_eq!(
            capability_lattice_for_phase(CapabilityPhase::RequestBootstrap),
            [CapabilityClass::MetricsWrite]
        );
    }

    #[test]
    fn policy_execution_lattice_grants_all_privileged_effect_classes() {
        assert_eq!(
            capability_lattice_for_phase(CapabilityPhase::PolicyExecution),
            [
                CapabilityClass::MetricsWrite,
                CapabilityClass::MonitoringWrite,
                CapabilityClass::EventLogWrite,
                CapabilityClass::BanWrite,
                CapabilityClass::ResponsePrivileged,
            ]
        );
    }

    #[test]
    fn post_response_flush_lattice_is_monitoring_only() {
        assert_eq!(
            capability_lattice_for_phase(CapabilityPhase::PostResponseFlush),
            [CapabilityClass::MonitoringWrite]
        );
    }

}
