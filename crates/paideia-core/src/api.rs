//! Types shared over HTTP between the local server and the web interface.

/// Response of `GET /api/health`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    /// Whether an Anthropic API key is configured on the server.
    /// The key itself is never sent to the browser.
    pub ai_configured: bool,
}
