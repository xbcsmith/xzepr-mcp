//! Metrics collection with Prometheus
//! Implementation is a stub for Phase 1 foundation.

/// Metrics collector
pub struct MetricsCollector {}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics server
pub struct MetricsServer {}

impl MetricsServer {
    /// Create new metrics server
    pub fn new(_port: u16) -> Self {
        Self {}
    }
}
