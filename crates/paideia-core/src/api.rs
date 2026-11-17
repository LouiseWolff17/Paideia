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

/// Body of every error response from `/api`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApiError {
    /// A calm, plain-language message that can be shown to the learner as is.
    pub message: String,
    /// The input field the problem is about, if any.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub field: Option<String>,
}
