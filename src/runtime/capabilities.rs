#[derive(Debug, Clone)]
pub(crate) struct RuntimeCapabilities {
    metrics: MetricsCapability,
    monitoring: MonitoringCapability,
    event_log: EventLogCapability,
    ban_write: BanWriteCapability,
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

impl RuntimeCapabilities {
    /// Request-path capabilities are minted at the trust boundary and passed
    /// explicitly to effect executors.
    pub(crate) fn for_request_path() -> Self {
        Self {
            metrics: MetricsCapability { _sealed: () },
            monitoring: MonitoringCapability { _sealed: () },
            event_log: EventLogCapability { _sealed: () },
            ban_write: BanWriteCapability { _sealed: () },
        }
    }

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
}
