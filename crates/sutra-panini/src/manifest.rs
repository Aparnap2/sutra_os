//! `.sutra` TOML manifest parser.
//!
//! A `.sutra` file is the declarative Pāṇinian envelope for a DevOps agent.
//! It defines:
//! - `[nimitta]`  — Wakeup triggers (Prometheus alerts, log patterns, cron)
//! - `[adhikara]` — Git VFS path permissions and capability bitmask
//! - `[finops]`   — FinOps token / millicent budget per execution frame
//! - `[nigamana]` — Post-commit canary verification command

use crate::PaniniError;
use serde::{Deserialize, Serialize};

/// Wakeup trigger (*Nimitta* — cause/occasion) definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NimittaTrigger {
    /// Prometheus/Alertmanager alert by name.
    PrometheusAlert { alert_name: String },
    /// Regex pattern matched against `journald` log stream.
    LogPattern { regex: String, unit: Option<String> },
    /// Cron expression (standard 5-field).
    Cron { expression: String },
    /// Git webhook on a specific ref pattern.
    GitWebhook { ref_pattern: String },
}

/// VFS path permission entry for *Adhikāra* (jurisdiction/authority).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhikaraPermission {
    /// Glob pattern (e.g., `/ontology/nodes/k8s/**`).
    pub path_glob: String,
    /// Whether the agent can read this path.
    pub read: bool,
    /// Whether the agent can stage writes on this path.
    pub write: bool,
}

/// FinOps (financial operations) budget for a single execution frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinOpsConfig {
    /// Maximum LLM input tokens per frame (0 = Wasm-only, no LLM calls).
    pub max_input_tokens: u32,
    /// Maximum LLM output tokens per frame.
    pub max_output_tokens: u32,
    /// Maximum cost ceiling in USD millicents (1/1000 of a cent).
    pub max_millicents: u64,
    /// Maximum Wasmtime fuel units (instruction budget).
    pub max_wasm_fuel: u64,
}

impl Default for FinOpsConfig {
    fn default() -> Self {
        Self {
            max_input_tokens: 4096,
            max_output_tokens: 1024,
            max_millicents: 100, // $0.001 per frame
            max_wasm_fuel: 1_000_000_000,
        }
    }
}

/// The complete parsed `.sutra` agent manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SutraManifest {
    /// Human-readable agent name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Wakeup trigger conditions.
    pub nimitta: Vec<NimittaTrigger>,
    /// Git VFS permissions.
    pub adhikara: Vec<AdhikaraPermission>,
    /// FinOps budget.
    pub finops: FinOpsConfig,
    /// Post-commit canary verification command.
    pub nigamana_cmd: String,
    /// Path to the compiled `.wasm` actor binary (relative to manifest).
    pub actor_wasm_path: String,
}

impl SutraManifest {
    /// Parse a `.sutra` TOML manifest from a string.
    ///
    /// # Examples
    /// ```no_run
    /// use sutra_panini::SutraManifest;
    /// let toml_str = std::fs::read_to_string("agent.sutra").unwrap();
    /// let manifest = SutraManifest::from_toml(&toml_str).unwrap();
    /// ```
    pub fn from_toml(src: &str) -> Result<Self, PaniniError> {
        toml::from_str(src).map_err(|e| PaniniError::ParseError(e.to_string()))
    }

    /// Serialise the manifest back to TOML.
    pub fn to_toml(&self) -> Result<String, PaniniError> {
        toml::to_string_pretty(self).map_err(|e| PaniniError::ParseError(e.to_string()))
    }

    /// Check whether the given VFS path is within this agent's read scope.
    pub fn can_read(&self, path: &str) -> bool {
        self.adhikara.iter().any(|a| a.read && glob_matches(&a.path_glob, path))
    }

    /// Check whether the given VFS path is within this agent's write scope.
    pub fn can_write(&self, path: &str) -> bool {
        self.adhikara.iter().any(|a| a.write && glob_matches(&a.path_glob, path))
    }
}

/// Simple glob match helper (supports `*` and `**`).
fn glob_matches(pattern: &str, path: &str) -> bool {
    use globset::{Glob, GlobBuilder};
    GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .ok()
        .map(|g| g.compile_matcher().is_match(path))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
name = "k8s-oom-healer"
version = "0.1.0"
actor_wasm_path = "actor.wasm"
nigamana_cmd = "kubectl get pod auth-service --no-headers | awk '{print $3}' | grep -q Running"

[[nimitta]]
kind = "prometheus_alert"
alert_name = "KubePodOOMKilling"

[[adhikara]]
path_glob = "/ontology/nodes/k8s/**"
read = true
write = false

[[adhikara]]
path_glob = "/incidents/**"
read = true
write = true

[finops]
max_input_tokens = 8192
max_output_tokens = 2048
max_millicents = 500
max_wasm_fuel = 2000000000
"#;

    #[test]
    fn parse_sample_manifest() {
        let m = SutraManifest::from_toml(SAMPLE_TOML).unwrap();
        assert_eq!(m.name, "k8s-oom-healer");
        assert_eq!(m.adhikara.len(), 2);
    }

    #[test]
    fn path_permission_checks() {
        let m = SutraManifest::from_toml(SAMPLE_TOML).unwrap();
        assert!(m.can_read("/ontology/nodes/k8s/worker-1.json"));
        assert!(!m.can_write("/ontology/nodes/k8s/worker-1.json"));
        assert!(m.can_write("/incidents/inc-001.json"));
    }
}
