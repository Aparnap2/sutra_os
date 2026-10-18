//! *Pratyāhāra* Capability Bitpacker.
//!
//! Models OS permissions after Pāṇini's *Śiva Sūtras* (phoneme groupings with
//! marker boundaries). All kernel capabilities are ordered in a canonical 16-bit
//! sequence; an agent's full permission set is encoded as a compact
//! `(start_idx, stop_idx)` span — identical to how Pāṇini denotes phoneme
//! groups by their first and last phoneme alone.

use crate::PaniniError;

/// The canonical ordered list of all SūtraOS kernel capabilities.
/// Ordered from least-privileged (index 0) to most-privileged (index N-1).
/// Inspired by the 14 *Śiva Sūtra* groups of Sanskrit phonemes.
pub const CAPABILITY_SEQUENCE: &[&str] = &[
    // Group 1: Read capabilities (index 0–3)
    "vfs.read.ontology",        // 0
    "vfs.read.runbooks",        // 1
    "vfs.read.incidents",       // 2
    "vfs.read.ledger",          // 3
    // Group 2: Write capabilities (index 4–7)
    "vfs.write.incidents",      // 4
    "vfs.write.ontology",       // 5
    "vfs.write.runbooks",       // 6
    "vfs.write.ledger",         // 7
    // Group 3: LLM syscall capabilities (index 8–10)
    "sys.llm.tier0",            // 8  — local/cheap model
    "sys.llm.tier1",            // 9  — mid-tier model
    "sys.llm.tier2",            // 10 — frontier model
    // Group 4: Network capabilities (index 11–13)
    "sys.http.read",            // 11
    "sys.http.write",           // 12
    "sys.http.phantom",         // 13 — phantom credential proxy
    // Group 5: Kernel admin capabilities (index 14–15)
    "kernel.merge.main",        // 14 — allowed to merge to main
    "kernel.admin",             // 15 — full kernel control
];

/// A compact 16-bit bitmask encoding all granted capabilities.
///
/// Bit `i` set ⟺ `CAPABILITY_SEQUENCE[i]` is granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PratyaharaBitmask(pub u16);

impl PratyaharaBitmask {
    /// Grant a single capability by name.
    pub fn grant_by_name(&mut self, name: &str) -> Result<(), PaniniError> {
        let idx = Self::index_of(name)?;
        self.0 |= 1u16 << idx;
        Ok(())
    }

    /// Revoke a single capability by name.
    pub fn revoke_by_name(&mut self, name: &str) -> Result<(), PaniniError> {
        let idx = Self::index_of(name)?;
        self.0 &= !(1u16 << idx);
        Ok(())
    }

    /// Check if a capability is granted.
    pub fn has(&self, name: &str) -> bool {
        Self::index_of(name)
            .map(|idx| (self.0 >> idx) & 1 == 1)
            .unwrap_or(false)
    }

    /// Encode a contiguous span `[start, stop]` (inclusive) of capabilities.
    /// This is the *Pratyāhāra* encoding: a single pair encodes a whole group.
    pub fn from_span(start: u8, stop: u8) -> Result<Self, PaniniError> {
        let max = CAPABILITY_SEQUENCE.len() as u8 - 1;
        if start > max || stop > max || start > stop {
            return Err(PaniniError::CapabilityOutOfBounds { index: stop, max });
        }
        let mut mask = 0u16;
        for i in start..=stop {
            mask |= 1u16 << i;
        }
        Ok(Self(mask))
    }

    /// Return the human-readable list of granted capabilities.
    pub fn granted_names(&self) -> Vec<&'static str> {
        CAPABILITY_SEQUENCE
            .iter()
            .enumerate()
            .filter(|(i, _)| (self.0 >> i) & 1 == 1)
            .map(|(_, name)| *name)
            .collect()
    }

    fn index_of(name: &str) -> Result<usize, PaniniError> {
        CAPABILITY_SEQUENCE
            .iter()
            .position(|&n| n == name)
            .ok_or_else(|| PaniniError::CapabilityOutOfBounds {
                index: 255,
                max: CAPABILITY_SEQUENCE.len() as u8 - 1,
            })
    }
}

impl std::fmt::Display for PratyaharaBitmask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PratyaharaBitmask(0b{:016b})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_and_check() {
        let mut mask = PratyaharaBitmask::default();
        mask.grant_by_name("vfs.read.ontology").unwrap();
        mask.grant_by_name("sys.llm.tier1").unwrap();
        assert!(mask.has("vfs.read.ontology"));
        assert!(mask.has("sys.llm.tier1"));
        assert!(!mask.has("kernel.admin"));
    }

    #[test]
    fn span_encoding() {
        // Read-only span: capabilities 0–3
        let mask = PratyaharaBitmask::from_span(0, 3).unwrap();
        assert!(mask.has("vfs.read.ontology"));
        assert!(mask.has("vfs.read.ledger"));
        assert!(!mask.has("vfs.write.incidents"));
    }

    #[test]
    fn granted_names_list() {
        let mut mask = PratyaharaBitmask::default();
        mask.grant_by_name("vfs.read.runbooks").unwrap();
        mask.grant_by_name("sys.http.phantom").unwrap();
        let names = mask.granted_names();
        assert!(names.contains(&"vfs.read.runbooks"));
        assert!(names.contains(&"sys.http.phantom"));
        assert_eq!(names.len(), 2);
    }
}
