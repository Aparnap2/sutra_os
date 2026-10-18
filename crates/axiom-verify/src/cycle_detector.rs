//! Piṅgala's Binary Trajectory Watchdog — O(1) Cycle Detection.
//!
//! Encodes an agent's sequence of system-call actions into a compact 64-bit
//! rolling register using Piṅgala's *Meru Prastāra* binary enumeration.
//! Detects repeating k-mer cycles (len 1, 2, or 3 repeated ≥ 3 times) in
//! O(1) bitwise operations.
//!
//! Reference: Piṅgala's *Chandaḥśāstra* (c. 200 BCE), binary enumeration of
//! Sanskrit metre patterns — the earliest known formal binary number system.

use crate::error::VerifyError;
use serde::{Deserialize, Serialize};

/// A single action event recorded in the trajectory register.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionEvent {
    /// Category of the action (e.g., "vfs_write", "llm_infer", "http_get").
    pub action_type: String,
    /// Target node or resource identifier.
    pub target_id: String,
    /// Exit status: 0 = success, non-zero = failure code.
    pub exit_status: i32,
}

impl ActionEvent {
    /// Compress the event into a 4-bit Piṅgala symbol using a simple hash.
    fn to_symbol(&self) -> u8 {
        let mut h: u32 = 0x811c_9dc5u32; // FNV-1a offset basis
        for byte in self
            .action_type
            .bytes()
            .chain(self.target_id.bytes())
            .chain(self.exit_status.to_le_bytes())
        {
            h ^= byte as u32;
            h = h.wrapping_mul(0x0100_0193); // FNV prime
        }
        (h & 0x0F) as u8 // 4-bit symbol ∈ [0, 15]
    }
}

/// Rolling 64-bit Piṅgala binary trajectory register.
///
/// Shift in 4-bit symbols from the right; the register holds the last 16
/// action events (64 bits / 4 bits each). Cycle detection checks for
/// repeating k-mers of length 1, 2, and 3.
#[derive(Debug, Default, Clone)]
pub struct CycleDetector {
    /// The 64-bit rolling state register.
    register: u64,
    /// Total number of events recorded so far.
    count: usize,
    /// Maximum allowed repeats of any cycle before signalling termination.
    max_repeats: usize,
}

impl CycleDetector {
    /// Create a new watchdog that signals after `max_repeats` identical cycles.
    pub fn new(max_repeats: usize) -> Self {
        Self { register: 0, count: 0, max_repeats }
    }

    /// Record a new action event and check for cycles.
    ///
    /// Returns `Ok(())` if the trajectory is still healthy, or
    /// `Err(VerifyError::ActionLoopDetected)` if a repeating cycle is found.
    pub fn record(&mut self, event: &ActionEvent) -> Result<(), VerifyError> {
        let symbol = event.to_symbol() as u64;
        // Shift existing register left by 4 bits and OR in new symbol
        self.register = (self.register << 4) | symbol;
        self.count += 1;

        if self.count >= 3 {
            // Check for k-mer cycles of length 1, 2, 3
            for k in 1usize..=3 {
                if self.count >= k * self.max_repeats {
                    if let Some(repeats) = self.count_repeats(k) {
                        if repeats >= self.max_repeats {
                            return Err(VerifyError::ActionLoopDetected {
                                cycle_len: k,
                                repeats,
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Count how many times the last `k` symbols repeat consecutively.
    fn count_repeats(&self, k: usize) -> Option<usize> {
        if k == 0 || k > 4 {
            return None;
        }
        let bits = k * 4;
        let mask: u64 = (1u64 << bits) - 1;
        let pattern = self.register & mask;

        let mut repeats = 1;
        let mut shifted = self.register >> bits;
        while (shifted & mask) == pattern {
            repeats += 1;
            shifted >>= bits;
            if repeats > 10 {
                break; // Avoid infinite loop on pathological registers
            }
        }
        Some(repeats)
    }

    /// Reset the watchdog (e.g., after a successful merge).
    pub fn reset(&mut self) {
        self.register = 0;
        self.count = 0;
    }

    /// Current raw register value (for debugging / audit).
    pub fn register_value(&self) -> u64 {
        self.register
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(action: &str, target: &str, status: i32) -> ActionEvent {
        ActionEvent {
            action_type: action.to_string(),
            target_id: target.to_string(),
            exit_status: status,
        }
    }

    #[test]
    fn no_cycle_in_diverse_actions() {
        let mut watchdog = CycleDetector::new(3);
        let events = [
            make_event("vfs_read", "node-a", 0),
            make_event("llm_infer", "node-b", 0),
            make_event("http_get", "node-c", 0),
            make_event("vfs_write", "node-d", 0),
            make_event("llm_infer", "node-e", 0),
        ];
        for ev in &events {
            assert!(watchdog.record(ev).is_ok(), "diverse actions should not trigger cycle");
        }
    }

    #[test]
    fn cycle_detected_on_repeat() {
        let mut watchdog = CycleDetector::new(3);
        let ev = make_event("llm_infer", "node-a", 1);
        // Record 3+ identical events — should detect cycle
        let mut detected = false;
        for _ in 0..10 {
            if watchdog.record(&ev).is_err() {
                detected = true;
                break;
            }
        }
        assert!(detected, "Identical repeated actions should trigger ActionLoopDetected");
    }
}
