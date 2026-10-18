//! Āryabhaṭa *Kuṭṭaka* Probe Scheduler.
//!
//! Schedules periodic health-check probes with deterministic jitter offsets
//! so no two heavy probes fire simultaneously (prevents Thundering Herd).

use sutra_nyaya::kuttaka::jitter_offsets;
use tracing::info;

/// A registered periodic probe.
#[derive(Debug, Clone)]
pub struct PeriodicProbe {
    pub name: String,
    pub period_ms: u64,
    pub offset_ms: u64,
    pub agent_id: String,
}

/// The Kuṭṭaka-powered probe scheduler.
pub struct KuttakaScheduler {
    probes: Vec<PeriodicProbe>,
}

impl KuttakaScheduler {
    /// Create a new scheduler with no registered probes.
    pub fn new() -> Self {
        Self { probes: vec![] }
    }

    /// Register a new periodic probe and compute its collision-free offset.
    pub fn register(&mut self, name: impl Into<String>, period_ms: u64, agent_id: impl Into<String>) {
        let idx = self.probes.len();
        let periods: Vec<u64> = self.probes.iter().map(|p| p.period_ms).chain([period_ms]).collect();
        let offsets = jitter_offsets(&periods);
        let offset_ms = offsets[idx];

        let probe = PeriodicProbe {
            name: name.into(),
            period_ms,
            offset_ms,
            agent_id: agent_id.into(),
        };
        info!(
            probe = %probe.name,
            period_ms = period_ms,
            offset_ms = offset_ms,
            "Registered probe with Kuṭṭaka jitter offset"
        );
        self.probes.push(probe);
    }

    /// Return probes that are due to fire at the given tick (milliseconds since epoch).
    pub fn due_probes(&self, tick_ms: u64) -> Vec<&PeriodicProbe> {
        self.probes
            .iter()
            .filter(|p| {
                if p.period_ms == 0 {
                    return false;
                }
                let adjusted_tick = tick_ms.saturating_sub(p.offset_ms);
                adjusted_tick % p.period_ms == 0
            })
            .collect()
    }

    /// List all registered probes.
    pub fn probes(&self) -> &[PeriodicProbe] {
        &self.probes
    }
}

impl Default for KuttakaScheduler {
    fn default() -> Self {
        Self::new()
    }
}
