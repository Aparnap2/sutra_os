//! Telemetry event reactor — ingests interrupts and dispatches agent wakeups.

use serde::{Deserialize, Serialize};
use sutra_nyaya::CatuskotiState;

/// The kind of telemetry event that woke an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    /// Prometheus alert fired.
    PrometheusAlert {
        alert_name: String,
        labels: std::collections::HashMap<String, String>,
        value: f64,
    },
    /// `journald` log pattern matched.
    LogPattern {
        unit: String,
        matched_line: String,
    },
    /// Cron tick.
    CronTick { expression: String },
    /// Git webhook event.
    GitWebhook { ref_name: String, commit_sha: String },
    /// Manual trigger (from `sutra trigger` CLI).
    ManualTrigger { payload: serde_json::Value },
}

/// A telemetry event that arrives at the kernel event loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Unique event ID (ULID).
    pub id: String,
    /// Target node ID (e.g., `k8s-worker-1`).
    pub target_node_id: String,
    /// The event kind and payload.
    pub kind: EventKind,
    /// ISO-8601 UTC timestamp.
    pub timestamp: String,
    /// Optional raw telemetry SHA-256 blob hash (becomes Hetu evidence).
    pub evidence_sha: Option<String>,
}

impl TelemetryEvent {
    /// Serialize to JSON for storage as a Hetu evidence blob.
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }
}
