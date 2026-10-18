//! LLM backend abstraction for `sys_llm_infer`.
//!
//! Supports multiple backends: OpenAI, Anthropic, Ollama (local), and a
//! deterministic Mock backend for offline integration tests.

use crate::WasmHostError;
use serde::{Deserialize, Serialize};

/// LLM request payload passed via the `sys_llm_infer` syscall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInferRequest {
    /// Model tier (0=local/cheap, 1=mid-tier, 2=frontier).
    pub system_tier: u8,
    /// Anuvṛtti-compressed context JSON.
    pub anuvritti_context_json: String,
    /// JSON schema that the response MUST conform to (grammar-constrained).
    pub response_schema_json: String,
}

/// LLM inference response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInferResponse {
    /// The structured JSON response conforming to `response_schema_json`.
    pub content: String,
    /// Actual tokens used (input + output).
    pub tokens_used: u32,
    /// Estimated cost in USD millicents.
    pub millicents_used: u64,
}

/// Trait for LLM backends.
#[async_trait::async_trait]
pub trait LlmBackend: Send + Sync {
    /// Perform a structured inference call.
    async fn infer(&self, req: &LlmInferRequest) -> Result<LlmInferResponse, WasmHostError>;

    /// Return the backend name (for logging).
    fn name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// Mock Backend (deterministic, offline, zero-cost)
// ---------------------------------------------------------------------------

/// Deterministic mock LLM backend for offline tests.
///
/// Always returns a pre-configured `NyayaCommitProof` JSON or echoes the schema.
pub struct MockLlmBackend {
    /// Fixed response to return for every request.
    pub fixed_response: String,
}

impl MockLlmBackend {
    /// Create a mock that always returns the given JSON string.
    pub fn with_response(response: impl Into<String>) -> Self {
        Self { fixed_response: response.into() }
    }
}

#[async_trait::async_trait]
impl LlmBackend for MockLlmBackend {
    async fn infer(&self, _req: &LlmInferRequest) -> Result<LlmInferResponse, WasmHostError> {
        Ok(LlmInferResponse {
            content: self.fixed_response.clone(),
            tokens_used: 42,
            millicents_used: 0,
        })
    }

    fn name(&self) -> &'static str {
        "MockLlmBackend"
    }
}

// ---------------------------------------------------------------------------
// HTTP-based Backend (OpenAI / Anthropic compatible)
// ---------------------------------------------------------------------------

/// HTTP-based LLM backend (OpenAI / Anthropic / Ollama compatible).
pub struct HttpLlmBackend {
    pub base_url: String,
    pub api_key: String,
    pub model_tiers: [String; 3], // tier 0, 1, 2
    pub client: reqwest::Client,
}

impl HttpLlmBackend {
    /// Create a new HTTP backend.
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        models: [impl Into<String>; 3],
    ) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model_tiers: models.map(|m| m.into()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl LlmBackend for HttpLlmBackend {
    async fn infer(&self, req: &LlmInferRequest) -> Result<LlmInferResponse, WasmHostError> {
        let model = self
            .model_tiers
            .get(req.system_tier as usize)
            .cloned()
            .unwrap_or_else(|| self.model_tiers[0].clone());

        let body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": format!(
                        "You are a DevOps root-cause analysis engine. Respond ONLY with valid JSON matching this schema: {}",
                        req.response_schema_json
                    )
                },
                {
                    "role": "user",
                    "content": req.anuvritti_context_json
                }
            ],
            "response_format": { "type": "json_object" }
        });

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| WasmHostError::LlmError(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| WasmHostError::LlmError(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let tokens_used = json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(LlmInferResponse {
            content,
            tokens_used,
            millicents_used: (tokens_used as u64) / 10, // Rough estimate
        })
    }

    fn name(&self) -> &'static str {
        "HttpLlmBackend"
    }
}

// Workaround: async_trait requires the crate
#[allow(unused_imports)]
use async_trait;
