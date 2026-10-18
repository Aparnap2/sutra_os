//! Phantom Credential Proxy.
//!
//! The Wasm guest never sees real credentials. All outbound HTTP calls pass
//! through the Phantom Proxy, which substitutes `$PHANTOM_<KEY>` placeholders
//! with real host-environment secrets in-flight.

use crate::WasmHostError;
use std::collections::HashMap;
use tracing::warn;

/// The Phantom Credential Proxy intercepts HTTP requests from Wasm guests and
/// replaces secret placeholders with real credentials from the host keyring.
pub struct PhantomProxy {
    /// Map from placeholder name (e.g., `PHANTOM_K8S_TOKEN`) to real secret.
    secrets: HashMap<String, String>,
    pub client: reqwest::Client,
}

impl PhantomProxy {
    /// Create a new PhantomProxy, loading secrets from environment variables.
    ///
    /// All environment variables prefixed with `PHANTOM_` are loaded as secrets.
    pub fn from_env() -> Self {
        let secrets: HashMap<String, String> = std::env::vars()
            .filter(|(k, _)| k.starts_with("PHANTOM_"))
            .collect();
        Self {
            secrets,
            client: reqwest::Client::new(),
        }
    }

    /// Create with explicit secrets (for testing).
    pub fn with_secrets(secrets: HashMap<String, String>) -> Self {
        Self { secrets, client: reqwest::Client::new() }
    }

    /// Substitute `$<PLACEHOLDER>` tokens in a string with real secrets.
    ///
    /// Unresolved placeholders are redacted to prevent leaking placeholder
    /// names in responses.
    pub fn substitute(&self, template: &str, placeholder: &str) -> String {
        if let Some(real) = self.secrets.get(placeholder) {
            template.replace(&format!("${placeholder}"), real)
        } else {
            warn!(placeholder = %placeholder, "Phantom secret not found — redacting");
            template.replace(&format!("${placeholder}"), "<REDACTED>")
        }
    }

    /// Perform an HTTP GET, substituting `secret_placeholder` in the Authorization header.
    pub async fn phantom_get(
        &self,
        url: &str,
        secret_placeholder: &str,
    ) -> Result<String, WasmHostError> {
        let token = self
            .secrets
            .get(secret_placeholder)
            .cloned()
            .unwrap_or_else(|| {
                warn!(placeholder = %secret_placeholder, "Phantom secret missing for HTTP call");
                String::new()
            });

        let resp = self
            .client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| WasmHostError::PhantomError(e.to_string()))?;

        resp.text()
            .await
            .map_err(|e| WasmHostError::PhantomError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_known_placeholder() {
        let mut secrets = HashMap::new();
        secrets.insert("PHANTOM_K8S_TOKEN".to_string(), "real-token-xyz".to_string());
        let proxy = PhantomProxy::with_secrets(secrets);
        let result = proxy.substitute("Bearer $PHANTOM_K8S_TOKEN", "PHANTOM_K8S_TOKEN");
        assert_eq!(result, "Bearer real-token-xyz");
    }

    #[test]
    fn redacts_unknown_placeholder() {
        let proxy = PhantomProxy::with_secrets(HashMap::new());
        let result = proxy.substitute("Bearer $PHANTOM_MISSING", "PHANTOM_MISSING");
        assert_eq!(result, "Bearer <REDACTED>");
    }
}
