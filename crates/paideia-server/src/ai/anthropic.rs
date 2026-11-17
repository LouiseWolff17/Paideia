//! Anthropic Messages API adapter (raw HTTP; there is no official Rust SDK).
//!
//! Uses structured outputs (`output_config.format`) so the reply is JSON that
//! matches [`super::prompt::content_schema`], and the server-side refusal
//! fallback so a declined request is retried on another model within the call.

use std::time::Duration;

use serde::Deserialize;
use serde_json::json;

use super::{parse_content, AiError, Generated, Prompt};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Room for thinking plus a full reply; well within the non-streaming limit.
const MAX_TOKENS: u32 = 16_000;
const TIMEOUT: Duration = Duration::from_secs(180);

pub struct AnthropicTutor {
    http: reqwest::Client,
    api_key: String,
    model: String,
    effort: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    model: String,
    stop_reason: Option<String>,
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(rename = "type")]
    kind: String,
}

impl AnthropicTutor {
    pub fn new(api_key: String, model: String, effort: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .expect("HTTP client with default TLS settings");
        Self { http, api_key, model, effort }
    }

    fn body(&self, prompt: &Prompt) -> serde_json::Value {
        json!({
            "model": self.model,
            "max_tokens": MAX_TOKENS,
            "system": prompt.system,
            "messages": [{ "role": "user", "content": prompt.user }],
            "output_config": {
                "effort": self.effort,
                "format": { "type": "json_schema", "schema": super::prompt::content_schema() },
            },
            "fallbacks": "default",
        })
    }

    pub async fn generate(&self, prompt: &Prompt) -> Result<Generated, AiError> {
        let resp = self
            .http
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("anthropic-beta", FALLBACK_BETA)
            .json(&self.body(prompt))
            .send()
            .await
            .map_err(|e| {
                // reqwest errors name the URL and the failure, never the body.
                AiError::Unavailable(e.without_url().to_string())
            })?;

        let status = resp.status();
        if !status.is_success() {
            // Only the error *type* is kept: messages can quote request content.
            let kind = resp
                .json::<ErrorResponse>()
                .await
                .map(|e| e.error.kind)
                .unwrap_or_else(|_| "unknown".into());
            return Err(match status.as_u16() {
                401 | 403 => AiError::KeyRejected,
                429 | 529 => AiError::Busy,
                s if s >= 500 => AiError::Unavailable(format!("HTTP {s} ({kind})")),
                s => AiError::Request(format!("HTTP {s} ({kind})")),
            });
        }

        let message: MessageResponse = resp
            .json()
            .await
            .map_err(|e| AiError::BadReply(format!("unreadable response: {}", e.without_url())))?;

        match message.stop_reason.as_deref() {
            Some("refusal") => return Err(AiError::Declined),
            Some("max_tokens") => return Err(AiError::BadReply("reply was cut off".into())),
            _ => {}
        }

        // Thinking blocks come first; the structured reply is the text block.
        let text = message
            .content
            .iter()
            .find(|b| b.kind == "text")
            .and_then(|b| b.text.as_deref())
            .ok_or_else(|| AiError::BadReply("no text block in reply".into()))?;

        Ok(Generated { content: parse_content(text)?, model: message.model })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_body_uses_structured_output_and_no_sampling_params() {
        let t = AnthropicTutor::new("k".into(), "claude-opus-5-5".into(), "medium".into());
        let body = t.body(&Prompt { system: "sys".into(), user: "hi".into() });
        assert_eq!(body["model"], "claude-opus-5-5");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["effort"], "medium");
        assert_eq!(body["messages"][0]["content"], "hi");
        // Rejected by API version 2026-10-21.
        assert!(body.get("temperature").is_none());
        // The key goes in a header, never in the body.
        assert!(!body.to_string().contains("\"k\""));
    }
}
