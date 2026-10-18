//! Error types for `sutra-panini`.

use thiserror::Error;

/// All errors from the Pāṇinian manifest & context engine.
#[derive(Debug, Error)]
pub enum PaniniError {
    #[error("Manifest parse error: {0}")]
    ParseError(String),

    #[error("Missing required field '{field}' in manifest")]
    MissingField { field: String },

    #[error("Invalid glob pattern '{pattern}': {reason}")]
    InvalidGlob { pattern: String, reason: String },

    #[error("Capability index {index} out of bounds (max {max})")]
    CapabilityOutOfBounds { index: u8, max: u8 },

    #[error("Context serialization error: {0}")]
    SerializationError(String),
}
