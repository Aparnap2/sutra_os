//! Jain / Buddhist *Catuṣkoṭi* 4-valued epistemic state lattice.
//!
//! Replaces boolean UP/DOWN monitors with a precise 4-valued logic that forbids
//! destructive infrastructure mutations when a node is unobservable.

use serde::{Deserialize, Serialize};

/// The four epistemic states for any infrastructure resource.
///
/// Based on the Jain *Syādvāda* doctrine of conditional predication and the
/// Buddhist *Catuṣkoṭi* (four corners) logical framework.
///
/// # Examples
/// ```
/// use axiom_verify::HealthState;
/// let state = HealthState::Asti;
/// assert!(state.can_mutate_destructively());
/// let blind = HealthState::Avaktavya;
/// assert!(!blind.can_mutate_destructively());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    /// 1. *Asti* — Deterministically healthy. All invariants verified.
    Asti,
    /// 2. *Nāsti* — Deterministically faulted. Confirmed down or broken.
    Nasti,
    /// 3. *Ubhaya* — Both healthy & faulted simultaneously. Flapping, Byzantine,
    ///    or partial degradation. Treat with extreme caution.
    Ubhaya,
    /// 4. *Avaktavya* — Unobservable / indescribable. Telemetry partition or
    ///    sensor blindspot. **Destructive mutations are FORBIDDEN.**
    Avaktavya,
}

impl HealthState {
    /// Returns `true` only if the kernel is allowed to execute destructive
    /// remediation (restart, drain, patch) on a node in this state.
    ///
    /// Forbidden states: `Avaktavya` (unobservable) because we cannot know
    /// the actual fault, and `Ubhaya` (flapping) because the system is in a
    /// split-brain scenario.
    pub fn can_mutate_destructively(self) -> bool {
        matches!(self, HealthState::Asti | HealthState::Nasti)
    }

    /// Lattice join (⊔): returns the "less certain" of two states.
    ///
    /// Used when aggregating observations from multiple probes.
    /// `Avaktavya` dominates everything (epistemic blindspot propagates).
    pub fn join(self, other: Self) -> Self {
        use HealthState::*;
        match (self, other) {
            (Avaktavya, _) | (_, Avaktavya) => Avaktavya,
            (Ubhaya, _) | (_, Ubhaya) => Ubhaya,
            (Asti, Nasti) | (Nasti, Asti) => Ubhaya,
            (a, _) => a,
        }
    }

    /// Lattice meet (⊓): returns the "more certain" of two states.
    pub fn meet(self, other: Self) -> Self {
        use HealthState::*;
        match (self, other) {
            (Asti, Asti) => Asti,
            (Nasti, Nasti) => Nasti,
            (Asti, _) | (_, Asti) => Asti,
            (Nasti, _) | (_, Nasti) => Nasti,
            (Ubhaya, _) | (_, Ubhaya) => Ubhaya,
            _ => Avaktavya,
        }
    }

    /// Transition to `Ubhaya` if the node has been flapping between `Asti`
    /// and `Nasti` within a time window.
    ///
    /// `observations` is a slice of recent states ordered oldest → newest.
    pub fn detect_flap(observations: &[Self]) -> Self {
        let has_asti = observations.iter().any(|s| *s == Self::Asti);
        let has_nasti = observations.iter().any(|s| *s == Self::Nasti);
        let has_avaktavya = observations.iter().any(|s| *s == Self::Avaktavya);

        if has_avaktavya {
            Self::Avaktavya
        } else if has_asti && has_nasti {
            Self::Ubhaya
        } else if has_nasti {
            Self::Nasti
        } else {
            Self::Asti
        }
    }
}

impl std::fmt::Display for HealthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asti => write!(f, "Asti (Healthy)"),
            Self::Nasti => write!(f, "Nāsti (Faulted)"),
            Self::Ubhaya => write!(f, "Ubhaya (Flapping/Byzantine)"),
            Self::Avaktavya => write!(f, "Avaktavya (Unobservable — LOCKED)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asti_can_mutate() {
        assert!(HealthState::Asti.can_mutate_destructively());
    }

    #[test]
    fn nasti_can_mutate() {
        assert!(HealthState::Nasti.can_mutate_destructively());
    }

    #[test]
    fn avaktavya_blocks_mutation() {
        assert!(!HealthState::Avaktavya.can_mutate_destructively());
    }

    #[test]
    fn ubhaya_blocks_mutation() {
        assert!(!HealthState::Ubhaya.can_mutate_destructively());
    }

    #[test]
    fn flap_detection() {
        let obs = vec![HealthState::Asti, HealthState::Nasti, HealthState::Asti];
        assert_eq!(HealthState::detect_flap(&obs), HealthState::Ubhaya);
    }

    #[test]
    fn join_avaktavya_dominates() {
        let result = HealthState::Asti.join(HealthState::Avaktavya);
        assert_eq!(result, HealthState::Avaktavya);
    }
}
